use std::collections::HashMap;

/// Core function to extract tickers from text.
pub fn extract_tickers_from_text(
    text: &str,
    symbols_map: &HashMap<String, Option<String>>,
) -> Vec<String> {
    let mut matches = vec![];
    let tokens: Vec<&str> = text.split_whitespace().collect();

    for token in tokens {
        // Only process tokens that are already upper-case
        if token == token.to_uppercase() {
            let normalized = token
                .trim_matches(|c: char| !c.is_alphanumeric())
                .to_uppercase();

            if symbols_map.contains_key(&normalized) {
                matches.push(normalized);
            }
        }
    }

    matches
}
