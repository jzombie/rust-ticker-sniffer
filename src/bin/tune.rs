use std::collections::HashMap;
use std::fs;

#[path = "../../test_utils/lib.rs"] // Adjusted to match your directory structure
mod test_utils;

use test_utils::{load_symbols_from_file, run_test_for_file};

fn tune_weights() {
    let test_dir = "tests/test_files";

    println!("Initializing tuning process...");

    let symbols_map =
        load_symbols_from_file("tests/test_symbols.csv").expect("Failed to load symbols from CSV");

    let mut weights = (0.5, 0.5, 0.5);
    let mut velocity = (0.0, 0.0, 0.0);
    let mut best_weights = weights;
    let mut best_loss = f32::MAX;

    let learning_rate = 0.1;
    let momentum = 0.9;
    let regularization_lambda = 0.01;
    let tolerance = 1e-5; // Minimum loss improvement to reset patience
    let max_epochs = 100; // Maximum number of epochs
    let patience = 5; // Number of epochs to wait for improvement

    let mut no_improvement_count = 0; // Tracks consecutive epochs without improvement

    for epoch in 1..=max_epochs {
        println!("Epoch {}/{}", epoch, max_epochs);

        let current_loss = evaluate_loss_with_regularization(
            weights.0,
            weights.1,
            weights.2,
            symbols_map.clone(),
            test_dir,
            regularization_lambda,
        );

        // Update best weights if current loss is lower
        if current_loss < best_loss - tolerance {
            best_loss = current_loss;
            best_weights = weights;
            no_improvement_count = 0; // Reset patience counter
            println!(
                "New best weights: ({:.4}, {:.4}, {:.4}), Loss: {:.4}",
                best_weights.0, best_weights.1, best_weights.2, best_loss
            );
        } else {
            no_improvement_count += 1; // Increment patience counter
        }

        let grad_w1 = compute_gradient_with_regularization(
            weights.0,
            weights.1,
            weights.2,
            &symbols_map,
            test_dir,
            0,
            regularization_lambda,
        );
        let grad_w2 = compute_gradient_with_regularization(
            weights.0,
            weights.1,
            weights.2,
            &symbols_map,
            test_dir,
            1,
            regularization_lambda,
        );
        let grad_w3 = compute_gradient_with_regularization(
            weights.0,
            weights.1,
            weights.2,
            &symbols_map,
            test_dir,
            2,
            regularization_lambda,
        );

        velocity.0 = momentum * velocity.0 + learning_rate * grad_w1;
        velocity.1 = momentum * velocity.1 + learning_rate * grad_w2;
        velocity.2 = momentum * velocity.2 + learning_rate * grad_w3;

        weights.0 -= velocity.0;
        weights.1 -= velocity.1;
        weights.2 -= velocity.2;

        println!(
            "Weights: ({:.4}, {:.4}, {:.4}), Loss: {:.4}",
            weights.0, weights.1, weights.2, current_loss
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
        "Tuning process completed. Best weights: ({:.4}, {:.4}, {:.4}), Best loss: {:.4}",
        best_weights.0, best_weights.1, best_weights.2, best_loss
    );
}

/// Compute the gradient for a specific weight with regularization
fn compute_gradient_with_regularization(
    w1: f32,
    w2: f32,
    w3: f32,
    symbols_map: &HashMap<String, Option<String>>,
    test_dir: &str,
    weight_index: usize,
    regularization_lambda: f32,
) -> f32 {
    let delta = 1e-5; // Small perturbation for finite differences
    let mut perturbed_weights = (w1, w2, w3);

    // Perturb the specific weight
    match weight_index {
        0 => perturbed_weights.0 += delta,
        1 => perturbed_weights.1 += delta,
        2 => perturbed_weights.2 += delta,
        _ => unreachable!(),
    }

    // Calculate the loss difference with regularization
    let loss_original = evaluate_loss_with_regularization(
        w1,
        w2,
        w3,
        symbols_map.clone(),
        test_dir,
        regularization_lambda,
    );
    let loss_perturbed = evaluate_loss_with_regularization(
        perturbed_weights.0,
        perturbed_weights.1,
        perturbed_weights.2,
        symbols_map.clone(),
        test_dir,
        regularization_lambda,
    );

    // Compute gradient as finite difference
    (loss_perturbed - loss_original) / delta
}

/// Evaluate the loss with L2 regularization
fn evaluate_loss_with_regularization(
    weight1: f32,
    weight2: f32,
    weight3: f32,
    symbols_map: HashMap<String, Option<String>>,
    test_dir: &str,
    regularization_lambda: f32,
) -> f32 {
    // Evaluate the original loss
    let base_loss = evaluate_loss(weight1, weight2, weight3, &symbols_map, test_dir);

    // Add L2 regularization penalty
    let l2_penalty = regularization_lambda * (weight1.powi(2) + weight2.powi(2) + weight3.powi(2));
    base_loss + l2_penalty
}

/// Evaluate the loss for given weights
fn evaluate_loss(
    weight1: f32,
    weight2: f32,
    weight3: f32,
    symbols_map: &HashMap<String, Option<String>>,
    test_dir: &str,
) -> f32 {
    let mut total_errors = 0;

    // Read test files
    let files = fs::read_dir(test_dir).expect("Failed to read test files directory");
    for file in files {
        let file = file.expect("Failed to read file");
        let file_path = file.path();

        if file_path.is_file() {
            total_errors += run_test_for_file(file_path.to_str().unwrap(), false);
        }
    }

    // Return total errors as the loss
    total_errors as f32
}

fn main() {
    tune_weights();
}
