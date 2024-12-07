use std::collections::HashMap;
use std::fs::{read_dir, File};
use std::io;
use std::os::unix::io::AsRawFd;
use test_utils::{load_symbols_from_file, run_test_for_file};
use ticker_sniffer::{ContextAttention, Weights, DEFAULT_WEIGHTS};

#[path = "../../test_utils/lib.rs"]
mod test_utils;

fn train_context_attention() {
    let test_dir = "tests/test_files";
    let symbols_file = "tests/test_symbols.csv";

    // Load symbols
    let symbols_map: HashMap<String, Option<String>> =
        load_symbols_from_file(symbols_file).expect("Failed to load symbols");

    // Initialize ContextAttention
    let mut context_attention = ContextAttention::new(256);

    // Hyperparameters
    let learning_rate = 0.01;
    let max_epochs = 1000;
    let patience = 10;
    let tolerance = 1e-5;

    // Initialize weights and best scores
    let mut weights = DEFAULT_WEIGHTS;
    let mut best_loss = f32::MAX;
    let mut no_improvement_count = 0;

    println!("Starting ContextAttention training...");

    for epoch in 1..=max_epochs {
        println!("Epoch {}/{}", epoch, max_epochs);

        // Evaluate current performance
        let current_loss = evaluate_loss(&context_attention, &weights, &symbols_map, test_dir);

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
            println!("No improvement for {} epochs. Stopping training.", patience);
            break;
        }

        // Compute gradients for global weights
        let gradient = compute_gradient(&context_attention, &weights, &symbols_map, test_dir);

        // Update global weights using gradient descent
        for i in 0..context_attention.global_weights.len() {
            context_attention.global_weights[i] -= learning_rate * gradient[i];
        }

        eprint!("Context weights: {:?}", context_attention.global_weights);
    }

    println!("Training completed. Best loss: {:.6}", best_loss);
}

/// Evaluate the current loss for ContextAttention
fn evaluate_loss(
    context_attention: &ContextAttention,
    weights: &Weights,
    symbols_map: &HashMap<String, Option<String>>,
    test_dir: &str,
) -> f32 {
    let mut total_loss = 0.0;
    let mut file_count = 0;

    for file in read_dir(test_dir).expect("Failed to read test directory") {
        let file = file.expect("Failed to read file");
        let file_path = file.path();

        if file_path.is_file() {
            // Run test and calculate MSE

            let (_, _, mse) = suppress_output(|| {
                run_test_for_file(
                    file_path.to_str().unwrap(),
                    false, // Disable assertions during training
                    weights.clone(),
                    context_attention,
                )
            });
            total_loss += mse;
            file_count += 1;
        }
    }

    total_loss / file_count as f32 // Return average loss
}

/// Compute gradients for global weights
fn compute_gradient(
    context_attention: &ContextAttention,
    weights: &Weights,
    symbols_map: &HashMap<String, Option<String>>,
    test_dir: &str,
) -> Vec<f32> {
    let delta = 1e-3;
    let mut gradient = vec![0.0; context_attention.global_weights.len()];

    let loss_original = evaluate_loss(context_attention, weights, symbols_map, test_dir);

    // Clone the context once
    let mut perturbed_attention = context_attention.clone();

    // Perturb all weights at once
    for i in 0..context_attention.global_weights.len() {
        perturbed_attention.global_weights[i] += delta;
    }

    // Evaluate perturbed loss
    let loss_perturbed = evaluate_loss(&perturbed_attention, weights, symbols_map, test_dir);

    // Compute gradients for all weights in one pass
    for i in 0..gradient.len() {
        gradient[i] = (loss_perturbed - loss_original) / delta;
    }

    gradient
}

fn main() {
    train_context_attention();
}

// TODO: Refactor and don't repeat (another instance is in `tune.rs`)
//
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
