use std::collections::HashMap;
use std::fs::{read_dir, File};
use std::io;
use std::os::unix::io::AsRawFd;
use test_utils::{load_symbols_from_file, run_test_for_file};
use ticker_sniffer::models::CompanyNameTokenRanking;
use ticker_sniffer::{ContextAttention, Weights, DEFAULT_WEIGHTS};

#[path = "utils/suppress_output.rs"]
mod suppress_output;
use suppress_output::suppress_output;

#[path = "../../test_utils/lib.rs"]
mod test_utils;

fn train_context_attention() {
    let test_dir = "tests/test_files";
    let symbols_file = "tests/test_symbols.csv";

    // Load symbols
    let symbols_map: HashMap<String, Option<String>> =
        load_symbols_from_file(symbols_file).expect("Failed to load symbols");

    // TODO: Start w/ pre-trained weights?
    // Initialize ContextAttention
    let mut context_attention = ContextAttention::new(256);

    // Hyperparameters
    let learning_rate = 0.01;
    let max_epochs = 1000;
    let patience = 10;
    let tolerance = 1e-5;

    // Initialize weights and best scores
    let weights = DEFAULT_WEIGHTS;
    let mut best_loss = f32::MAX;
    let mut no_improvement_count = 0;

    println!("Starting ContextAttention training...");

    for epoch in 1..=max_epochs {
        println!("Epoch {}/{}", epoch, max_epochs);

        // Evaluate current performance
        let (current_loss, all_company_rankings) =
            evaluate_loss(&context_attention, &weights, &symbols_map, test_dir);

        println!("Epoch {} - Loss: {:.6}", epoch, current_loss);

        // Track best weights and early stopping
        if current_loss < best_loss - tolerance {
            best_loss = current_loss;
            no_improvement_count = 0;
            println!("New best loss: {:.6}", best_loss);
        } else {
            no_improvement_count += 1;
        }

        if no_improvement_count >= patience {
            println!(
                "No improvement for {} epochs. Stopping training.",
                no_improvement_count
            );
            break;
        }

        for company_ranking in all_company_rankings {
            context_attention.update_weights(
                &company_ranking.context_query_string,
                &company_ranking.context_company_tokens,
                // target,
                1.0, // TODO: Dynamically determine: 1.0 for true positive, 0.0 for false positive
                learning_rate,
            );
        }

        // Update weights using the update_weights method
        // for file in read_dir(test_dir).expect("Failed to read test directory") {
        //     let file = file.expect("Failed to read file");
        //     let file_path = file.path();

        //     if file_path.is_file() {
        //         // Read the file content
        //         let raw_text =
        //             std::fs::read_to_string(file_path).expect("Failed to read test file");

        //         // TODO: Remove
        //         // Filter out lines starting with 'EXPECTED:', 'EXPECTED_FAILURE:', or 'COMMENT:'
        //         let filtered_text: String = raw_text
        //             .lines()
        //             .filter(|line| {
        //                 !line.trim_start().starts_with("EXPECTED:")
        //                     && !line.trim_start().starts_with("EXPECTED_FAILURE:")
        //                     && !line.trim_start().starts_with("COMMENT:")
        //             })
        //             .collect::<Vec<&str>>()
        //             .join("\n");

        //         // TODO: Replace
        //         // Extract context from the filtered text
        //         let context: Vec<String> =
        //             filtered_text.split_whitespace().map(String::from).collect();

        //         // TODO: Replace
        //         // Simulate target and ticker for training (modify as needed)
        //         let ticker = "EXAMPLE"; // Replace with the actual ticker from the file
        //         let target = 1.0; // Example: Set to 1.0 for true positive

        //         // Update weights
        //         context_attention.update_weights(&ticker, &context, target, learning_rate);
        //     }
        // }

        eprintln!("Context weights: {:?}", context_attention.global_weights);
    }

    println!("Training completed. Best loss: {:.6}", best_loss);
}

/// Evaluate the current loss for ContextAttention
fn evaluate_loss(
    context_attention: &ContextAttention,
    weights: &Weights,
    _symbols_map: &HashMap<String, Option<String>>,
    test_dir: &str,
) -> (f32, Vec<CompanyNameTokenRanking>) {
    let mut total_loss = 0.0;
    let mut file_count = 0;

    let mut all_company_rankings: Vec<CompanyNameTokenRanking> = Vec::new();

    for file in read_dir(test_dir).expect("Failed to read test directory") {
        let file = file.expect("Failed to read file");
        let file_path = file.path();

        if file_path.is_file() {
            // Run test and calculate MSE

            let (_, _, mse, company_rankings, _evaluation_results) = suppress_output(|| {
                run_test_for_file(
                    file_path.to_str().unwrap(),
                    false, // Disable assertions during training
                    weights.clone(),
                    context_attention,
                )
            });
            total_loss += mse;
            file_count += 1;

            // TODO: Collect these and use to influence weight updates
            // eprintln!(
            //     "expected: {:?}, results: {:?}, company_rankings: {:?}",
            //     expected_tickers, results, company_rankings
            // );
            for company_ranking in company_rankings {
                all_company_rankings.push(company_ranking);
            }
        }
    }

    let average_loss = total_loss / file_count as f32;

    (average_loss, all_company_rankings)
}

// TODO: Remove?
// Compute gradients for global weights
// fn compute_gradient(
//     context_attention: &ContextAttention,
//     weights: &Weights,
//     symbols_map: &HashMap<String, Option<String>>,
//     test_dir: &str,
// ) -> Vec<f32> {
//     let delta = 1e-3;
//     let mut gradient = vec![0.0; context_attention.global_weights.len()];

//     let (loss_original, _) = evaluate_loss(context_attention, weights, symbols_map, test_dir);

//     // Clone the context once
//     let mut perturbed_attention = context_attention.clone();

//     // Perturb all weights at once
//     for i in 0..context_attention.global_weights.len() {
//         perturbed_attention.global_weights[i] += delta;
//     }

//     // Evaluate perturbed loss
//     let (loss_perturbed, _) = evaluate_loss(&perturbed_attention, weights, symbols_map, test_dir);

//     // Compute gradients for all weights in one pass
//     for i in 0..gradient.len() {
//         gradient[i] = (loss_perturbed - loss_original) / delta;
//     }

//     gradient
// }

fn main() {
    train_context_attention();
}
