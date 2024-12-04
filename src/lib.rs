use std::collections::{HashMap, HashSet};

/// Tokenizer function to split the text into individual tokens.
pub fn tokenize(text: &str) -> Vec<&str> {
    text.split_whitespace().collect()
}

pub fn extract_tickers_from_text(
    text: &str,
    symbols_map: &HashMap<String, Option<String>>, // Same map for both symbols and company names
) -> Vec<String> {
    let mut matches = HashSet::new();

    // Extract tickers by symbol
    let symbol_matches = extract_tickers_from_symbols(text, symbols_map);
    matches.extend(symbol_matches);

    // Extract tickers by company name
    let company_name_matches = extract_tickers_from_company_names(text, symbols_map);
    matches.extend(company_name_matches);

    // Convert HashSet to Vec and return
    matches.into_iter().collect()
}

pub fn extract_tickers_from_symbols(
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

/// Extract tickers from company names using a scoring mechanism.
pub fn extract_tickers_from_company_names(
    text: &str,
    symbols_map: &HashMap<String, Option<String>>, // Same map for symbols and names
) -> Vec<String> {
    let mut matches: HashMap<String, f32> = HashMap::new(); // Explicit type for scores

    // Tokenize the input text, keeping only tokens starting with an uppercase letter
    let input_tokens: Vec<&str> = text
        .split_whitespace()
        .filter(|token| token.chars().next().map_or(false, |c| c.is_uppercase()))
        .collect();

    // eprintln!("Debug: Input tokens: {:?}", input_tokens);

    for (symbol, company_name) in symbols_map {
        if let Some(company_name) = company_name {
            // Normalize and tokenize the company name
            let normalized_company = company_name
                .replace(|c: char| !c.is_alphanumeric() && c != ' ', " ")
                .to_lowercase();
            let company_tokens: Vec<&str> = normalized_company.split_whitespace().collect();

            if company_tokens.is_empty() {
                // eprintln!("Debug: Skipping empty company name for symbol {}", symbol);
                continue; // Skip if the company name is empty
            }

            // eprintln!(
            //     "Debug: Checking company: {} (tokens: {:?}) against input tokens: {:?}",
            //     company_name, company_tokens, input_tokens
            // );

            // Calculate the match score
            let match_score = calculate_match_score(&input_tokens, &company_tokens);

            // Debugging output
            // eprintln!(
            //     "Debug: Input tokens: {:?}, Company tokens: {:?}, Match score: {}, Symbol: {}",
            //     input_tokens, company_tokens, match_score, symbol
            // );

            // Add or update the score for this symbol if it meets the threshold
            if match_score >= 0.4 {
                matches
                    .entry(symbol.clone())
                    .and_modify(|existing_score| *existing_score = existing_score.max(match_score))
                    .or_insert(match_score);
            }
        }
    }

    // Collect all symbols that passed the threshold
    let mut result: Vec<_> = matches.into_iter().collect();

    // Sort by score (descending) and break ties lexicographically
    result.sort_by(|(sym_a, score_a), (sym_b, score_b)| {
        score_b
            .partial_cmp(score_a)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| sym_a.cmp(sym_b))
    });

    // Debugging output for final matches
    eprintln!("Debug: Final matches: {:?}", result);

    // Return the sorted symbols
    result.into_iter().map(|(symbol, _)| symbol).collect()
}

/// Calculate the match score based on token overlap and continuity.
fn calculate_match_score(input_tokens: &[&str], company_tokens: &[&str]) -> f32 {
    let total_company_tokens = company_tokens.len() as f32;
    let mut total_matches = 0;
    let mut max_continuous_matches = 0;

    let mut i = 0;
    while i < input_tokens.len() {
        let mut current_match = 0;
        for (j, company_token) in company_tokens.iter().enumerate() {
            if i + j < input_tokens.len()
                && input_tokens[i + j].to_lowercase() == company_token.to_lowercase()
            {
                current_match += 1;
            } else {
                break;
            }
        }
        if current_match > 0 {
            total_matches += current_match;
            max_continuous_matches = max_continuous_matches.max(current_match);
            i += current_match; // Skip matched tokens
        } else {
            i += 1;
        }
    }

    let coverage_score = total_matches as f32 / total_company_tokens;
    let continuity_score = max_continuous_matches as f32 / total_company_tokens;

    // Weighted combination
    let final_score = 0.7 * continuity_score + 0.3 * coverage_score;

    // Debugging
    // eprintln!(
    //     "Debug: Input tokens: {:?}, Company tokens: {:?}, Continuity score: {:.2}, Coverage score: {:.2}, Final score: {:.2}",
    //     input_tokens, company_tokens, continuity_score, coverage_score, final_score
    // );

    final_score
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
