use std::collections::{HashMap, HashSet};

/// Tokenizer function to split the text into individual tokens.
pub fn tokenize(text: &str) -> Vec<&str> {
    text.split_whitespace().collect()
}

/// Core function to extract tickers from text.
pub fn extract_tickers_from_text(
    text: &str,
    symbols_map: &HashMap<String, Option<String>>,
) -> Vec<String> {
    let mut matches = HashSet::new(); // Use a HashSet to eliminate duplicates
    let tokens = tokenize(text); // Use the tokenizer function

    for token in tokens {
        // Only process tokens that are already upper-case
        if token == token.to_uppercase() {
            let normalized = token
                .trim_matches(|c: char| !c.is_alphanumeric())
                .to_uppercase();

            // Generate alternative symbols and check if any of them match
            let alternatives = generate_alternative_symbols(&normalized);
            for alt in alternatives {
                if symbols_map.contains_key(&alt) {
                    matches.insert(alt); // Use `insert` to ensure uniqueness
                    break; // No need to check other alternatives once a match is found
                }
            }
        }
    }

    // Convert the HashSet to a Vec to return the result
    matches.into_iter().collect()
}

pub fn generate_alternative_symbols(query: &str) -> Vec<String> {
    let mut alternatives: Vec<String> = vec![query.to_uppercase()];
    if query.contains('.') {
        alternatives.push(query.replace('.', "-").to_uppercase());
    } else if query.contains('-') {
        alternatives.push(query.replace('-', ".").to_uppercase());
    }
    alternatives
}
