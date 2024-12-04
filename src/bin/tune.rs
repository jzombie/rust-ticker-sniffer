use std::fs;
use std::path::Path;

#[path = "../../test_utils/lib.rs"] // Adjusted to match your directory structure
mod test_utils;

use test_utils::{load_symbols_from_file, run_test_for_file};

fn tune_weights() {
    let test_dir = "tests/test_files";

    println!("Initializing tuning process...");

    // Load symbols from a test CSV file
    let symbols_map =
        load_symbols_from_file("tests/test_symbols.csv").expect("Failed to load symbols from CSV");

    // Define weight ranges
    let weight_range = 0.0..=1.0; // Adjust range as needed
    let step = 0.1; // Step size for weight increments

    let mut best_weights = (0.0, 0.0, 0.0);
    let mut best_score = f32::MIN;

    // Iterate over epochs
    let epochs = 10; // Number of epochs to tune
    for epoch in 1..=epochs {
        println!("Epoch {}/{}", epoch, epochs);

        // Iterate through weight combinations
        for weight1 in (weight_range.start() * 10.0_f32).ceil() as i32
            ..=(weight_range.end() * 10.0_f32).floor() as i32
        {
            for weight2 in (weight_range.start() * 10.0_f32).ceil() as i32
                ..=(weight_range.end() * 10.0_f32).floor() as i32
            {
                for weight3 in (weight_range.start() * 10.0_f32).ceil() as i32
                    ..=(weight_range.end() * 10.0_f32).floor() as i32
                {
                    let w1 = weight1 as f32 * step;
                    let w2 = weight2 as f32 * step;
                    let w3 = weight3 as f32 * step;

                    // Placeholder: Replace with actual weight-adjusted function
                    let score = evaluate_weights(w1, w2, w3, &symbols_map, test_dir);

                    if score > best_score {
                        best_score = score;
                        best_weights = (w1, w2, w3);
                        println!(
                            "New best weights: ({:.2}, {:.2}, {:.2}) with score {:.4}",
                            w1, w2, w3, score
                        );
                    }
                }
            }
        }
    }

    println!(
        "Tuning process completed. Best weights: ({:.2}, {:.2}, {:.2}) with score {:.4}",
        best_weights.0, best_weights.1, best_weights.2, best_score
    );
}

/// Evaluate weights by applying them and scoring performance
fn evaluate_weights(
    weight1: f32,
    weight2: f32,
    weight3: f32,
    symbols_map: &std::collections::HashMap<String, Option<String>>,
    test_dir: &str,
) -> f32 {
    // Temporary score accumulator
    let mut total_score = 0.0;

    // Read test files
    let files = fs::read_dir(test_dir).expect("Failed to read test files directory");
    for file in files {
        let file = file.expect("Failed to read file");
        let file_path = file.path();

        if file_path.is_file() {
            // Simulate running with the weights (replace this with your actual logic)
            let _baseline_results = run_test_for_file(file_path.to_str().unwrap());
            let score = simulate_performance_with_weights(weight1, weight2, weight3);
            total_score += score;
        }
    }

    // Average score across all test files
    total_score
}

/// Simulate performance with the given weights (replace with actual implementation)
fn simulate_performance_with_weights(weight1: f32, weight2: f32, weight3: f32) -> f32 {
    // Placeholder logic: return a dummy score based on weights
    // Replace this with actual evaluation logic (e.g., MSE, accuracy, etc.)
    weight1 + weight2 + weight3 // Example: sum of weights
}

fn main() {
    tune_weights();
}
