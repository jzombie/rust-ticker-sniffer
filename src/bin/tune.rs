use rand::Rng;
use std::fs;

#[path = "../../test_utils/lib.rs"] // Adjusted to match your directory structure
mod test_utils;

use test_utils::{load_symbols_from_file, run_test_for_file};

fn tune_weights() {
    let test_dir = "tests/test_files";

    println!("Initializing tuning process...");

    // Load symbols from a test CSV file
    let symbols_map =
        load_symbols_from_file("tests/test_symbols.csv").expect("Failed to load symbols from CSV");

    // Define weight range and step size
    let weight_range = 0.0_f32..=1.0_f32; // Explicit range
    let mut best_weights = (0.0, 0.0, 0.0);

    // Goal is to minimize the score; so start with MAX potential score
    let mut best_score = usize::MAX;

    let epochs = 10; // Number of epochs to tune
    for epoch in 1..=epochs {
        println!("Epoch {}/{}", epoch, epochs);

        // Randomly generate weights for this epoch
        let mut rng = rand::thread_rng();
        let w1 = rng.gen_range(weight_range.clone());
        let w2 = rng.gen_range(weight_range.clone());
        let w3 = rng.gen_range(weight_range.clone());

        // Evaluate the generated weights
        let score = evaluate_weights(w1, w2, w3, &symbols_map, test_dir);

        // Goal is to minimize the score
        if score < best_score {
            best_score = score;
            best_weights = (w1, w2, w3);
            println!(
                "New best weights: ({:.2}, {:.2}, {:.2}) with score {}",
                w1, w2, w3, score
            );
        }
    }

    println!(
        "Tuning process completed. Best weights: ({:.2}, {:.2}, {:.2}) with score {}",
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
) -> usize {
    let mut total_errors = 0;

    // Read test files
    let files = fs::read_dir(test_dir).expect("Failed to read test files directory");
    for file in files {
        let file = file.expect("Failed to read file");
        let file_path = file.path();

        if file_path.is_file() {
            total_errors += run_test_for_file(file_path.to_str().unwrap());
        }
    }

    total_errors
}

fn main() {
    tune_weights();
}
