use std::collections::HashMap;
use std::fs::read_dir;
use ticker_sniffer::{ResultBiasAdjuster, Weights, DEFAULT_WEIGHTS};

#[path = "../../test_utils/lib.rs"]
mod test_utils;
use test_utils::constants::{TEST_FILES_DIRECTORY, TEST_SYMBOLS_CSV_PATH};
use test_utils::{load_symbols_from_file, run_test_for_file, EvaluationResult};

#[path = "../../bin_utils/lib.rs"]
mod bin_utils;
use bin_utils::suppress_output;

const SHOULD_SUPPRESS_OUTPUT: bool = true;

struct StructuredQueryContext {
    query: String,
    context: Vec<String>,
}

fn train_result_bias_adjuster() {
    let test_dir = TEST_FILES_DIRECTORY;
    let symbols_file = TEST_SYMBOLS_CSV_PATH;

    // Load symbols
    let symbols_map: HashMap<String, Option<String>> =
        load_symbols_from_file(TEST_SYMBOLS_CSV_PATH).expect("Failed to load symbols");

    // Initialize ResultBiasAdjuster
    let mut result_bias_adjuster = ResultBiasAdjuster::new();

    // Hyperparameters
    let learning_rate = 0.01;
    let max_epochs = 1000;
    let patience = 10;
    let tolerance = 1e-5;

    // Initialize weights and best scores
    let weights = DEFAULT_WEIGHTS;
    let mut best_loss = f32::MAX;
    let mut no_improvement_count = 0;

    println!("Starting Bias Adjuster training...");

    for epoch in 1..=max_epochs {
        println!("Epoch {}/{}", epoch, max_epochs);

        // Evaluate current performance
        let (current_loss, all_evaluation_results) = evaluate_loss(
            &result_bias_adjuster,
            &weights,
            &symbols_map,
            test_dir,
            SHOULD_SUPPRESS_OUTPUT,
        );

        // Stop training if no more loss
        if current_loss == 0.0 {
            best_loss = 0.0;
            break;
        }

        println!("Epoch {} - Loss: {:.8}", epoch, current_loss);

        // Track best weights and early stopping
        if current_loss < best_loss - tolerance {
            best_loss = current_loss;
            no_improvement_count = 0;
            println!("New best loss: {:.8}", best_loss);
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

        let mut false_positive_query_contexts = HashMap::new();
        let mut false_negative_query_contexts = HashMap::new();

        // First, collect grouped query contexts to ensure they are unique
        for evaluation_result in all_evaluation_results {
            // Collect false positive query contexts
            for company_name_token_rating in evaluation_result.false_positive_rankings {
                let query_hash = result_bias_adjuster.hash_query_context(
                    &company_name_token_rating.context_query_string,
                    &company_name_token_rating.context_company_tokens,
                );

                false_positive_query_contexts.insert(
                    query_hash,
                    StructuredQueryContext {
                        query: company_name_token_rating.context_query_string.clone(),
                        context: company_name_token_rating.context_company_tokens.clone(),
                    },
                );
            }

            // Collect false negative query contexts
            for company_name_token_rating in evaluation_result.false_negative_rankings {
                let query_hash = result_bias_adjuster.hash_query_context(
                    &company_name_token_rating.context_query_string,
                    &company_name_token_rating.context_company_tokens,
                );

                false_negative_query_contexts.insert(
                    query_hash,
                    StructuredQueryContext {
                        query: company_name_token_rating.context_query_string.clone(),
                        context: company_name_token_rating.context_company_tokens.clone(),
                    },
                );
            }
        }

        // Process false positive contexts
        for (_, structured_context) in &false_positive_query_contexts {
            eprintln!(
                "Context - False Positive - {}, {:?}",
                structured_context.query, structured_context.context
            );

            result_bias_adjuster.update_weights(
                &structured_context.query,
                &structured_context.context,
                0.0, // TODO: Make configurable
                learning_rate,
            );
        }

        // Process false negative contexts
        for (_, structured_context) in &false_negative_query_contexts {
            eprintln!(
                "Context - False Negative - {}, {:?}",
                structured_context.query, structured_context.context
            );

            result_bias_adjuster.update_weights(
                &structured_context.query,
                &structured_context.context,
                0.0, // TODO: Make configurable
                learning_rate,
            );
        }
    }

    println!(
        "Weights: {:?}",
        result_bias_adjuster.to_weight_slice_format()
    );

    println!("Training completed. Best loss: {:.6}", best_loss);
}

/// Evaluate the current loss for ResultBiasAdjuster
fn evaluate_loss(
    result_bias_adjuster: &ResultBiasAdjuster,
    weights: &Weights,
    _symbols_map: &HashMap<String, Option<String>>,
    test_dir: &str,
    should_suppress_output: bool,
) -> (f32, Vec<EvaluationResult>) {
    let mut total_loss = 0.0;
    let mut file_count = 0;

    let mut all_evaluation_results: Vec<EvaluationResult> = Vec::new();

    for file in read_dir(test_dir).expect("Failed to read test directory") {
        let file = file.expect("Failed to read file");
        let file_path = file.path();

        if file_path.is_file() {
            // Declare evaluation_result outside the block
            let evaluation_result: EvaluationResult;

            // Run test and calculate MSE
            evaluation_result = suppress_output(
                || {
                    let (_, _, result) = run_test_for_file(
                        file_path.to_str().expect("Invalid UTF-8 in file path"),
                        false, // Disable assertions during training
                        weights.clone(),
                        result_bias_adjuster,
                    );
                    result
                },
                should_suppress_output,
            );

            total_loss += evaluation_result.mse;
            file_count += 1;

            all_evaluation_results.push(evaluation_result);
        }
    }

    // Avoid division by zero
    let average_loss = if file_count > 0 {
        total_loss / file_count as f32
    } else {
        0.0
    };

    (average_loss, all_evaluation_results)
}

// TODO: Remove?
// Compute gradients for global weights
// fn compute_gradient(
//     result_bias_adjuster: &ResultBiasAdjuster,
//     weights: &Weights,
//     symbols_map: &HashMap<String, Option<String>>,
//     test_dir: &str,
// ) -> Vec<f32> {
//     let delta = 1e-3;
//     let mut gradient = vec![0.0; result_bias_adjuster.global_weights.len()];

//     let (loss_original, _) = evaluate_loss(result_bias_adjuster, weights, symbols_map, test_dir);

//     // Clone the context once
//     let mut perturbed_attention = result_bias_adjuster.clone();

//     // Perturb all weights at once
//     for i in 0..result_bias_adjuster.global_weights.len() {
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
    train_result_bias_adjuster();
}
