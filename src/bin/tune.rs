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
        // continuity: 0.025 + rng.gen_range(-0.1..0.1),
        // mismatched_letter_penalty: 1.0 + rng.gen_range(-0.1..0.1),
        // mismatched_word_penalty: 0.3 + rng.gen_range(-0.1..0.1),
        // match_score_threshold: 0.25 + rng.gen_range(-0.1..0.1),
        // symbol_abbr_threshold: 0.8 + rng.gen_range(-0.1..0.1),
        // continuity: 0.05939734,
        // mismatched_letter_penalty: 0.7206682,
        // mismatched_word_penalty: 0.24114594,
        // match_score_threshold: 0.30506316,
        // symbol_abbr_threshold: 0.88860726,
        continuity: 0.025,
        mismatched_letter_penalty: 1.0,
        mismatched_word_penalty: 0.3,
        match_score_threshold: 0.25,
        symbol_abbr_threshold: 0.8,
    };

    let mut velocity = (0.0, 0.0, 0.0, 0.0, 0.0);
    let mut best_weights = weights.clone();
    let mut best_loss = f32::MAX;

    let learning_rate = 0.005;
    let momentum = 0.05;
    let regularization_lambda = 0.01;
    let tolerance = 1e-5; // Minimum loss improvement to reset patience
    let max_epochs = 2000; // Maximum number of epochs
    let patience = 5; // Number of epochs to wait for improvement

    let max_gradient_norm = 4.0; // Define a maximum gradient norm to prevent explosion

    let mut no_improvement_count = 0; // Tracks consecutive epochs without improvement

    let mut initial_loss = None; // To track the loss in the first epoch

    for epoch in 1..=max_epochs {
        println!("Epoch {}/{}", epoch, max_epochs);

        // weights.normalize(1.0);

        let current_loss = evaluate_loss_with_regularization(
            weights,
            symbols_map.clone(),
            test_dir,
            regularization_lambda,
        );

        if initial_loss.is_none() {
            initial_loss = Some(current_loss); // Store the first loss
        }

        // Calculate the percentage improvement over the initial loss
        if let Some(init_loss) = initial_loss {
            let improvement = ((init_loss - current_loss) / init_loss) * 100.0;
            println!(
                "Epoch {}/{} - Current Loss: {:.4}, Improvement: {:.2}%",
                epoch, max_epochs, current_loss, improvement
            );
        } else {
            println!(
                "Epoch {}/{} - Current Loss: {:.4}",
                epoch, max_epochs, current_loss
            );
        }

        // Update best weights if current loss is lower
        if current_loss < best_loss - tolerance {
            best_loss = current_loss;
            best_weights = weights.clone();
            no_improvement_count = 0; // Reset patience counter
            println!(
                "New best weights: ({}), Loss: {:.4}",
                best_weights, best_loss
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
        let grad_w5 = compute_gradient_with_regularization(
            weights,
            &symbols_map,
            test_dir,
            4,
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
        velocity.4 = (momentum * velocity.4 + learning_rate * grad_w5)
            .clamp(-max_gradient_norm, max_gradient_norm);

        weights.continuity -= velocity.0;
        weights.mismatched_letter_penalty -= velocity.1;
        weights.mismatched_word_penalty -= velocity.2;
        weights.match_score_threshold -= velocity.3;
        weights.symbol_abbr_threshold -= velocity.4;

        println!("Weights: ({}), Loss: {:.4}", weights, current_loss);

        println!(
            "Gradients: grad_w1 = {:.5}, grad_w2 = {:.5}, grad_w3 = {:.5}, grad_w4 = {:.5}, grad_w5 = {:.5}",
            grad_w1, grad_w2, grad_w3, grad_w4, grad_w5
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
        "Tuning process completed. Best weights: ({}), Best loss: {:.4}",
        best_weights, best_loss
    );

    if let Some(init_loss) = initial_loss {
        let total_improvement = ((init_loss - best_loss) / init_loss) * 100.0;
        println!(
            "Tuning completed. Best weights: ({}), Best loss: {:.4} (Improvement: {:.2}%)",
            best_weights, best_loss, total_improvement
        );
    }
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
    // let delta = 1e-3;
    let delta = 0.005;

    let mut perturbed_weights = weights.clone();

    // Perturb the specific weight
    match weight_index {
        0 => perturbed_weights.continuity += delta,
        1 => perturbed_weights.mismatched_letter_penalty += delta,
        2 => perturbed_weights.mismatched_word_penalty += delta,
        3 => perturbed_weights.match_score_threshold += delta,
        4 => perturbed_weights.symbol_abbr_threshold += delta,
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
        * (weights.continuity.powi(2)
            + weights.mismatched_letter_penalty.powi(2)
            + weights.mismatched_word_penalty.powi(2)
            + weights.match_score_threshold.powi(2)
            + weights.symbol_abbr_threshold.powi(2));
    base_loss + l2_penalty
}

/// Evaluate the loss for given weights
fn evaluate_loss(
    weights: Weights,
    _symbols_map: &HashMap<String, Option<String>>,
    test_dir: &str,
) -> f32 {
    let mut total_mse: f32 = 0.0;
    let mut test_file_count = 0;

    // Read test files
    let files = read_dir(test_dir).expect("Failed to read test files directory");
    for file in files {
        let file = file.expect("Failed to read file");
        let file_path = file.path();

        if file_path.is_file() {
            // Wrap `run_test_for_file` to suppress output
            let (_, _, mse) = suppress_output(|| {
                run_test_for_file(file_path.to_str().unwrap(), false, weights.clone())
            });

            total_mse += mse;
            test_file_count += 1;
        }
    }

    // Calculate the average MSE
    let average_mse = if test_file_count > 0 {
        total_mse / test_file_count as f32
    } else {
        0.0 // Handle edge case where no test files are found
    };

    // println!("Average MSE for weights ({}): {:.4}", weights, average_mse);

    average_mse
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
