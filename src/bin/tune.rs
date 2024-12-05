use rand::Rng;
use std::collections::HashMap;
use std::fs::{read_dir, File};
use std::io;
use std::os::unix::io::AsRawFd;
use ticker_sniffer::Weights;

#[path = "../../test_utils/lib.rs"] // Adjusted to match your directory structure
mod test_utils;

use test_utils::{load_symbols_from_file, run_test_for_file};

fn tune_weights() {
    let test_dir = "tests/test_files";

    println!("Initializing tuning process...");

    let symbols_map =
        load_symbols_from_file("tests/test_symbols.csv").expect("Failed to load symbols from CSV");

    let mut rng = rand::thread_rng();

    // Initialize randomly
    let mut weights = Weights {
        // mismatched_letter_penalty: 0.5 + rng.gen_range(-0.1..0.1),
        // mismatched_word_penalty: 0.5 + rng.gen_range(-0.1..0.1),
        // match_score_threshold: 0.5 + rng.gen_range(-0.1..0.1),
        // bias: 0.5 + rng.gen_range(-0.1..0.1),
        mismatched_letter_penalty: 0.0,
        mismatched_word_penalty: 1.0,
        match_score_threshold: 0.5,
        bias: 1.0,
    };

    let mut velocity = (0.0, 0.0, 0.0, 0.0, 0.0);
    let mut best_weights = weights.clone();
    let mut best_loss = f32::MAX;

    let learning_rate = 0.01;
    let momentum = 0.05;
    let regularization_lambda = 0.01;
    let tolerance = 1e-5; // Minimum loss improvement to reset patience
    let max_epochs = 2000; // Maximum number of epochs
    let patience = 5; // Number of epochs to wait for improvement

    let max_gradient_norm = 4.0; // Define a maximum gradient norm to prevent explosion

    let mut no_improvement_count = 0; // Tracks consecutive epochs without improvement

    for epoch in 1..=max_epochs {
        println!("Epoch {}/{}", epoch, max_epochs);

        // weights.normalize(1.0);

        let current_loss = evaluate_loss_with_regularization(
            weights,
            symbols_map.clone(),
            test_dir,
            regularization_lambda,
        );

        // Update best weights if current loss is lower
        if current_loss < best_loss - tolerance {
            best_loss = current_loss;
            best_weights = weights.clone();
            no_improvement_count = 0; // Reset patience counter
            println!(
                "New best weights: ({:.4}, {:.4}, {:.4}, {:.4}), Loss: {:.4}",
                best_weights.mismatched_letter_penalty,
                best_weights.mismatched_word_penalty,
                best_weights.match_score_threshold,
                best_weights.bias,
                best_loss
            );
        } else {
            no_improvement_count += 1; // Increment patience counter
        }

        let grad_w1 = compute_gradient_with_regularization(
            weights,
            &symbols_map,
            test_dir,
            0,
            regularization_lambda,
            max_gradient_norm,
        );
        let grad_w2 = compute_gradient_with_regularization(
            weights,
            &symbols_map,
            test_dir,
            1,
            regularization_lambda,
            max_gradient_norm,
        );
        let grad_w3 = compute_gradient_with_regularization(
            weights,
            &symbols_map,
            test_dir,
            2,
            regularization_lambda,
            max_gradient_norm,
        );
        let grad_w4 = compute_gradient_with_regularization(
            weights,
            &symbols_map,
            test_dir,
            3,
            regularization_lambda,
            max_gradient_norm,
        );

        velocity.0 = (momentum * velocity.0 + learning_rate * grad_w1)
            .clamp(-max_gradient_norm, max_gradient_norm);
        velocity.1 = (momentum * velocity.1 + learning_rate * grad_w2)
            .clamp(-max_gradient_norm, max_gradient_norm);
        velocity.2 = (momentum * velocity.2 + learning_rate * grad_w3)
            .clamp(-max_gradient_norm, max_gradient_norm);
        velocity.3 = (momentum * velocity.3 + learning_rate * grad_w4)
            .clamp(-max_gradient_norm, max_gradient_norm);

        weights.mismatched_letter_penalty -= velocity.0;
        weights.mismatched_word_penalty -= velocity.1;
        weights.match_score_threshold -= velocity.2;
        weights.bias -= velocity.3;

        println!(
            "Weights: ({:.4}, {:.4}, {:.4}, {:.4}), Loss: {:.4}",
            weights.mismatched_letter_penalty,
            weights.mismatched_word_penalty,
            weights.match_score_threshold,
            weights.bias,
            current_loss
        );

        println!(
            "Gradients: grad_w1 = {:.5}, grad_w2 = {:.5}, grad_w3 = {:.5}, grad_w4 = {:.5}",
            grad_w1, grad_w2, grad_w3, grad_w4
        );

        // Check for convergence with patience
        if no_improvement_count >= patience {
            println!(
                "Stopping early due to lack of improvement for {} epochs.",
                no_improvement_count
            );
            break;
        }
    }

    println!(
        "Tuning process completed. Best weights: ({:.4}, {:.4}, {:.4}, {:.4}), Best loss: {:.4}",
        best_weights.mismatched_letter_penalty,
        best_weights.mismatched_word_penalty,
        best_weights.match_score_threshold,
        best_weights.bias,
        best_loss
    );
}

/// Compute the gradient for a specific weight with regularization
fn compute_gradient_with_regularization(
    weights: Weights,
    symbols_map: &HashMap<String, Option<String>>,
    test_dir: &str,
    weight_index: usize,
    regularization_lambda: f32,
    max_gradient_norm: f32,
) -> f32 {
    let delta = 1e-5; // Small perturbation for finite differences

    let mut perturbed_weights = weights.clone();

    // Perturb the specific weight
    match weight_index {
        0 => perturbed_weights.mismatched_letter_penalty += delta,
        1 => perturbed_weights.mismatched_word_penalty += delta,
        2 => perturbed_weights.match_score_threshold += delta,
        3 => perturbed_weights.bias += delta,
        _ => unreachable!(),
    }

    // Calculate the loss difference with regularization
    let loss_original = evaluate_loss_with_regularization(
        weights,
        symbols_map.clone(),
        test_dir,
        regularization_lambda,
    );
    let loss_perturbed = evaluate_loss_with_regularization(
        perturbed_weights,
        symbols_map.clone(),
        test_dir,
        regularization_lambda,
    );

    // Compute gradient as finite difference
    let mut gradient = (loss_perturbed - loss_original) / delta;

    // Clip the gradient to prevent explosion
    if gradient.abs() > max_gradient_norm {
        gradient = gradient.signum() * max_gradient_norm;
    }

    // Log the difference between the losses
    println!(
        "Weight index: {}, Loss Original: {:.6}, Loss Perturbed: {:.6}, Difference: {:.6}, Gradient: {:.6}",
        weight_index,
        loss_original,
        loss_perturbed,
        loss_perturbed - loss_original,
        gradient
    );

    gradient
}

/// Evaluate the loss with L2 regularization
fn evaluate_loss_with_regularization(
    weights: Weights,
    symbols_map: HashMap<String, Option<String>>,
    test_dir: &str,
    regularization_lambda: f32,
) -> f32 {
    // Evaluate the original loss
    let base_loss = evaluate_loss(weights.clone(), &symbols_map, test_dir);

    // Add L2 regularization penalty
    let l2_penalty = regularization_lambda
        * (weights.mismatched_letter_penalty.powi(2)
            + weights.mismatched_word_penalty.powi(2)
            + weights.match_score_threshold.powi(2)
            + weights.bias.powi(2));
    base_loss + l2_penalty
}

/// Evaluate the loss for given weights
fn evaluate_loss(
    weights: Weights,
    _symbols_map: &HashMap<String, Option<String>>,
    test_dir: &str,
) -> f32 {
    let mut total_errors = 0;
    let mut total_score: f32 = 0.0;

    // Read test files
    let files = read_dir(test_dir).expect("Failed to read test files directory");
    for file in files {
        let file = file.expect("Failed to read file");
        let file_path = file.path();

        if file_path.is_file() {
            // Wrap `run_test_for_file` to suppress output
            let (next_errors, next_score) = suppress_output(|| {
                run_test_for_file(file_path.to_str().unwrap(), false, weights.clone())
            });

            total_errors += next_errors;
            total_score += next_score;
        }
    }

    // Log the total loss in the console
    // let total_loss = total_errors as f32 - total_score;

    // let weight_sum = weights.mismatched_letter_penalty
    //     + weights.mismatched_word_penalty
    //     + weights.match_score_threshold
    //     + weights.bias;

    // // Regularization term to penalize large weights
    // let weight_penalty = 0.1 * weight_sum.powi(2); // Adjust coefficient as needed

    // Updated loss function
    // let total_loss = (total_errors.pow(2)) as f32 - total_score + weight_penalty;
    let total_loss = (total_errors.pow(2)) as f32 - total_score;
    // let total_loss = (1.0 - total_score) + weight_penalty;
    // let total_loss = 1.0 - total_score;

    println!(
        "Loss for weights ({:.4}, {:.4}, {:.4}, {:.4}): {:.4} (errors: {})",
        weights.mismatched_letter_penalty,
        weights.mismatched_word_penalty,
        weights.match_score_threshold,
        weights.bias,
        total_loss,
        total_errors
    );

    total_loss
}

/// Suppress output of a given closure
fn suppress_output<F, T>(f: F) -> T
where
    F: FnOnce() -> T,
{
    let dev_null = File::open("/dev/null").expect("Failed to open /dev/null");
    let null_fd = dev_null.as_raw_fd();

    // Backup stdout and stderr using `dup`
    let stdout_backup = unsafe { libc::dup(io::stdout().as_raw_fd()) };
    let stderr_backup = unsafe { libc::dup(io::stderr().as_raw_fd()) };

    if stdout_backup < 0 || stderr_backup < 0 {
        panic!("Failed to backup stdout or stderr");
    }

    // Redirect stdout and stderr to /dev/null
    unsafe {
        libc::dup2(null_fd, io::stdout().as_raw_fd());
        libc::dup2(null_fd, io::stderr().as_raw_fd());
    }

    let result = f(); // Run the closure

    // Restore original stdout and stderr
    unsafe {
        libc::dup2(stdout_backup, io::stdout().as_raw_fd());
        libc::dup2(stderr_backup, io::stderr().as_raw_fd());
        libc::close(stdout_backup); // Close the backup descriptors
        libc::close(stderr_backup);
    }

    result
}

fn main() {
    tune_weights();
}
