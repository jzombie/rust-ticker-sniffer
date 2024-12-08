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
    let query = "Apple";
    let query_words: Vec<&str> = query.split_whitespace().collect();
    let query_vectors: Vec<Vec<u32>> = query_words
        .iter()
        .map(|word| string_to_charcode_vector(word))
        .collect();

    println!("Query: {}, Word Vectors: {:?}", query, query_vectors);

    // Step 3: Filter candidates by length and pad query vectors
    let length_tolerance = 2; // Tolerance for length difference
    let mut similarities = Vec::new();

    for (symbol, company_name, vectors, word_lengths) in &word_vectors {
        // Define the sliding window: indices for the range of words to consider
        let start_index = 0;
        let end_index = 2; // Exclusive (does not include the end index)

        // Extract the vectors and word lengths within the sliding window
        let limited_vectors: Vec<&Vec<u32>> = vectors
            .iter()
            .skip(start_index)
            .take(end_index - start_index)
            .collect();
        let limited_word_lengths: Vec<usize> = word_lengths
            .iter()
            .skip(start_index)
            .take(end_index - start_index)
            .cloned()
            .collect();

        let mut total_similarity = 0.0;

        for (query_vector, query_length) in query_vectors
            .iter()
            .zip(query_words.iter().map(|w| w.len()))
        {
            // Check if any word in the sliding window matches the query length tolerance
            if limited_word_lengths
                .iter()
                .any(|&length| (length as isize - query_length as isize).abs() <= length_tolerance)
            {
                // Determine the maximum length for padding
                let max_length = limited_vectors
                    .iter()
                    .map(|v| v.len())
                    .chain(std::iter::once(query_vector.len())) // Include query vector length
                    .max()
                    .unwrap_or(0);

                // Pad the query vector and the candidate vectors to the same length
                let padded_query_vector = pad_vector(query_vector, max_length);
                let padded_limited_vectors: Vec<Vec<u32>> = limited_vectors
                    .iter()
                    .map(|v| pad_vector(v, max_length))
                    .collect();

                // Compute cosine similarity
                let max_similarity = padded_limited_vectors
                    .iter()
                    .map(|candidate| cosine_similarity(&padded_query_vector, candidate))
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

fn string_to_charcode_vector(input: &str) -> Vec<u32> {
    input
        .chars()
        .filter(|c| c.is_alphanumeric()) // Remove punctuation
        .map(|c| c.to_ascii_lowercase() as u32) // Convert to lowercase and get char code
        .collect()
}

/// Convert a string into a vector of character codes, removing punctuation and making it lowercase
// fn string_to_charcode_vector(input: &str) -> Vec<u32> {
//     // Normalize the input: remove punctuation and convert to lowercase
//     let normalized: String = input
//         .chars()
//         .filter(|c| c.is_alphanumeric()) // Remove punctuation
//         .map(|c| c.to_ascii_lowercase()) // Convert to lowercase
//         .collect();

//     // Generate trigrams and expand each character into the vector
//     let mut trigram_vector = Vec::new();
//     let chars: Vec<char> = normalized.chars().collect();
//     for i in 0..chars.len().saturating_sub(2) {
//         let trigram: &[char] = &chars[i..i + 3]; // Take a trigram slice
//                                                  // Add each character's code to the vector
//         trigram_vector.extend(trigram.iter().map(|&c| c as u32));
//     }

//     // Remove the last element if there are 5 or more entries
//     // if trigram_vector.len() >= 5 {
//     //     trigram_vector.pop();
//     // }

//     trigram_vector
// }

/// Pad a vector with zeros to the desired length
fn pad_vector(vector: &[u32], length: usize) -> Vec<u32> {
    let mut padded = vector.to_vec();
    if padded.len() < length {
        padded.resize(length, 0);
    }
    padded
}

/// Calculate cosine similarity between two vectors
fn cosine_similarity(v1: &[u32], v2: &[u32]) -> f64 {
    assert_eq!(
        v1.len(),
        v2.len(),
        "Vectors must have the same length for cosine similarity"
    );

    let dot_product: u64 = v1.iter().zip(v2).map(|(a, b)| *a as u64 * *b as u64).sum();
    let magnitude_v1: f64 = (v1.iter().map(|x| (*x as f64).powi(2)).sum::<f64>()).sqrt();
    let magnitude_v2: f64 = (v2.iter().map(|x| (*x as f64).powi(2)).sum::<f64>()).sqrt();

    if magnitude_v1 == 0.0 || magnitude_v2 == 0.0 {
        return 0.0;
    }

    dot_product as f64 / (magnitude_v1 * magnitude_v2)
}
