#[path = "../../test_utils/lib.rs"]
mod test_utils;
use test_utils::constants::TEST_SYMBOLS_CSV_PATH;
use test_utils::load_symbols_from_file;

use std::collections::HashMap;

fn main() {
    let symbols_map =
        load_symbols_from_file(TEST_SYMBOLS_CSV_PATH).expect("Failed to load symbols from CSV");

    // Step 1: Collect vectors for each word in company names
    let mut word_vectors: Vec<(String, String, Vec<Vec<u32>>, Vec<usize>)> = Vec::new();

    for (symbol, company_name_option) in symbols_map.iter() {
        if let Some(company_name) = company_name_option {
            let words: Vec<&str> = company_name.split_whitespace().collect();
            let vectors: Vec<Vec<u32>> = words
                .iter()
                .map(|word| string_to_charcode_vector(word))
                .collect();
            let word_lengths: Vec<usize> = words.iter().map(|word| word.len()).collect();

            word_vectors.push((symbol.clone(), company_name.clone(), vectors, word_lengths));
        }
    }

    // Step 2: Query a test string
    let query = "Berkshire";
    let query_words: Vec<&str> = query.split_whitespace().collect();
    let query_vectors: Vec<Vec<u32>> = query_words
        .iter()
        .map(|word| string_to_charcode_vector(word))
        .collect();

    println!("Query: {}, Word Vectors: {:?}", query, query_vectors);

    // Step 3: Filter candidates by length and truncate vectors
    let length_tolerance = 2;
    let mut similarities = Vec::new();

    for (symbol, company_name, vectors, word_lengths) in &word_vectors {
        let mut total_similarity = 0.0;

        for (query_vector, query_length) in query_vectors
            .iter()
            .zip(query_words.iter().map(|w| w.len()))
        {
            // Check if any word in the company name matches the query length tolerance
            if word_lengths
                .iter()
                .any(|&length| (length as isize - query_length as isize).abs() <= length_tolerance)
            {
                // Truncate each candidate vector to the query vector length
                let truncated_vectors: Vec<Vec<u32>> = vectors
                    .iter()
                    .map(|v| truncate_vector(v, query_vector.len()))
                    .collect();

                // Compute cosine similarity
                let max_similarity = truncated_vectors
                    .iter()
                    .map(|v| cosine_similarity(query_vector, v))
                    .fold(0.0, f64::max);

                total_similarity += max_similarity;
            }
        }

        if total_similarity > 0.0 {
            similarities.push((
                symbol.clone(),
                company_name.clone(),
                total_similarity / query_words.len() as f64, // Average similarity across query words
            ));
        }
    }

    // Step 4: Sort by cosine similarity (descending)
    similarities.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));

    // Step 5: Print top results
    println!("\nTop Matches:");
    for (symbol, company_name, similarity) in similarities.iter().take(5) {
        println!(
            "Symbol: {}, Company Name: {}, Cosine Similarity: {:.4}",
            symbol, company_name, similarity
        );
    }
}

/// Convert a string into a vector of character codes, removing punctuation and making it lowercase
fn string_to_charcode_vector(input: &str) -> Vec<u32> {
    input
        .chars()
        .filter(|c| c.is_alphanumeric()) // Remove punctuation
        .map(|c| c.to_ascii_lowercase() as u32) // Convert to lowercase and get char code
        .collect()
}

/// Truncate a vector to the desired length
fn truncate_vector(vector: &[u32], length: usize) -> Vec<u32> {
    vector.iter().cloned().take(length).collect()
}

/// Calculate cosine similarity between two vectors
fn cosine_similarity(v1: &[u32], v2: &[u32]) -> f64 {
    let dot_product: u64 = v1.iter().zip(v2).map(|(a, b)| *a as u64 * *b as u64).sum();
    let magnitude_v1: f64 = (v1.iter().map(|x| (*x as f64).powi(2)).sum::<f64>()).sqrt();
    let magnitude_v2: f64 = (v2.iter().map(|x| (*x as f64).powi(2)).sum::<f64>()).sqrt();

    if magnitude_v1 == 0.0 || magnitude_v2 == 0.0 {
        return 0.0;
    }

    dot_product as f64 / (magnitude_v1 * magnitude_v2)
}
