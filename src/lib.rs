use std::collections::{HashMap, HashSet};
mod constants;
use crate::constants::STOP_WORDS;

pub type SymbolsMap<'a> = &'a HashMap<String, Option<String>>;

#[derive(Debug, Clone, Copy)]
pub struct Weights {
    pub continuity: f32,
    pub mismatched_letter_penalty: f32,
    pub mismatched_word_penalty: f32,
    pub match_score_threshold: f32,
    pub symbol_abbr_threshold: f32,
}

impl Weights {
    pub fn new(
        continuity: f32,
        mismatched_letter_penalty: f32,
        mismatched_word_penalty: f32,
        match_score_threshold: f32,
        symbol_abbr_threshold: f32,
    ) -> Self {
        Self {
            continuity,
            mismatched_letter_penalty,
            mismatched_word_penalty,
            match_score_threshold,
            symbol_abbr_threshold,
        }
    }
}

/// Tokenizer function to split the text into individual tokens.
///
/// Note: This explcitly does not modify the case of the text.
pub fn tokenize(text: &str) -> Vec<&str> {
    text.split_whitespace()
        .map(|word| word.trim_end_matches(|c: char| !c.is_alphanumeric()))
        .collect()
}

pub fn extract_tickers_from_text(
    text: &str,
    symbols_map: SymbolsMap,
    weights: Weights,
) -> (Vec<String>, f32) {
    let mut matches = HashSet::new();

    // Extract tickers by company name
    let (company_name_matches, total_score, tokenized_filter) =
        extract_tickers_from_company_names(text, symbols_map, weights);
    matches.extend(company_name_matches);

    let filtered_text: String = text
        .split_whitespace()
        .filter(|word| !tokenized_filter.contains(&word.to_string()))
        .collect::<Vec<&str>>()
        .join(" ");

    // Extract tickers by symbol
    let symbol_matches = extract_tickers_from_symbols(&filtered_text, symbols_map);
    matches.extend(symbol_matches);

    let abbreviation_matches =
        extract_tickers_from_abbreviations(&filtered_text, symbols_map, weights);
    matches.extend(abbreviation_matches);

    // Convert HashSet to Vec and return sorted for consistency
    let mut results: Vec<String> = matches.into_iter().collect();
    results.sort();

    (results, total_score)
}

fn extract_tickers_from_symbols(text: &str, symbols_map: SymbolsMap) -> Vec<String> {
    let mut matches = HashSet::new();
    let tokens = tokenize(text);

    for token in tokens {
        // Normalize token to match symbol patterns
        if token == token.to_uppercase() {
            let normalized = token.to_uppercase();

            // Check if the normalized token directly matches any symbol
            if symbols_map.contains_key(&normalized) {
                matches.insert(normalized.clone());
            } else {
                // Generate alternative symbols and check matches
                let alternatives = generate_alternative_symbols(&normalized);
                for alt in alternatives {
                    if symbols_map.contains_key(&alt) {
                        matches.insert(alt);
                        break; // Stop checking alternatives once matched
                    }
                }
            }
        }
    }

    matches.into_iter().collect()
}

fn extract_tickers_from_abbreviations(
    text: &str,
    symbols_map: SymbolsMap,
    weights: Weights,
) -> Vec<String> {
    let mut matches = HashSet::new();
    let input_tokens = tokenize(text);

    let input_tokens_capitalized: Vec<&str> = input_tokens
        .iter()
        .filter(|token| token.chars().next().map_or(false, |c| c.is_uppercase()) && token.len() > 1) // Min length > 1
        .filter(|token| {
            let lowercased = token.to_lowercase();
            !STOP_WORDS.contains(&lowercased.as_str())
        })
        .cloned()
        .collect();

    for token in input_tokens_capitalized {
        // Normalize the token to lowercase
        let lc_token = token.to_lowercase();

        for (symbol, _company_name) in symbols_map {
            // let lc_company_name = company_name.to_lowercase();
            let lc_symbol = symbol.to_lowercase();

            // Check if the token starts with part of the company name
            if lc_token.starts_with(&lc_symbol) {
                let token_length = token.len();
                let symbol_length = symbol.len();

                let abbr_perc = symbol_length as f32 / token_length as f32;

                if abbr_perc > weights.symbol_abbr_threshold {
                    matches.insert(symbol.to_string());
                }
            }
        }
    }

    matches.into_iter().collect()
}

fn extract_tickers_from_company_names(
    text: &str,
    symbols_map: SymbolsMap,
    weights: Weights,
) -> (Vec<String>, f32, HashSet<String>) {
    let normalized_text = text
        // .to_lowercase()
        .replace(|c: char| !c.is_alphanumeric() && c != ' ', " "); // Normalize input

    let input_tokens = tokenize(&normalized_text);

    let mut scored_results: HashMap<String, f32> = HashMap::new();
    let mut tokenized_filter: HashSet<String> = HashSet::new();

    if !input_tokens.is_empty() {
        // Filter input tokens: Only consider tokens starting with a capital letter and of sufficient length, then remove stop words
        let input_tokens_capitalized: Vec<&str> = input_tokens
            .iter()
            .filter(|token| {
                token.chars().next().map_or(false, |c| c.is_uppercase()) && token.len() > 1
            }) // Min length > 1
            .filter(|token| {
                let lowercased = token.to_lowercase();
                !STOP_WORDS.contains(&lowercased.as_str())
            })
            .cloned()
            .collect();

        for (symbol, company_name) in symbols_map {
            // Skip entries without a valid company name
            if let Some(company_name) = company_name {
                if company_name.is_empty() {
                    continue;
                }

                let company_name_char_count = company_name.len();

                // Normalize, filter stop words, and tokenize the company name
                let company_tokens: Vec<String> = company_name
                    .to_lowercase()
                    .replace(|c: char| !c.is_alphanumeric() && c != ' ', " ")
                    .split_whitespace()
                    .filter(|word| !STOP_WORDS.contains(word))
                    .map(String::from)
                    .collect();

                if company_tokens.is_empty() {
                    continue;
                }

                let total_company_words = company_tokens.len();
                let mut consecutive_match_count = 0;
                let mut top_consecutive_match_count = 0;

                let mut consecutive_input_token_char_count = 0;
                let mut top_consecutive_input_token_char_count = 0;

                let mut match_score = 0.0;

                // Single pass through input tokens
                let mut company_index = 0;

                // let mut seen_tokens: HashSet<String> = HashSet::new();

                for input_token in &input_tokens_capitalized {
                    // eprintln!(
                    //     "input token: {}, company index: {}, company name: {}",
                    //     input_token, company_index, company_name
                    // );

                    let lc_input_token = input_token.to_lowercase();
                    // let input_token_char_count = input_token.len();
                    //

                    // eprintln!("Input token: {}", input_token);

                    if &lc_input_token != &company_tokens[company_index] {
                        // Note: This reset is perfomrmed before the following `if` statement to fix an issue
                        // where a phrase with `Apple Apple Hopitality REIT` are identified as separate companies.
                        // Previously, the consecutive match mechanism would get out of sync and identify
                        // `Apple Hospitality REIT` with a low score.
                        consecutive_match_count = 0;
                        consecutive_input_token_char_count = 0;

                        company_index = 0;
                    }

                    if &lc_input_token == &company_tokens[company_index] {
                        // consecutive_input_token_char_count += input_token_char_count;

                        // Match found, increment the company pointer
                        consecutive_match_count += 1;
                        consecutive_input_token_char_count += 1;

                        company_index += 1;

                        if consecutive_match_count > top_consecutive_match_count {
                            top_consecutive_match_count = consecutive_match_count;
                        }

                        if consecutive_input_token_char_count
                            > top_consecutive_input_token_char_count
                        {
                            top_consecutive_input_token_char_count =
                                consecutive_input_token_char_count;
                        }

                        // If we've matched the entire company_tokens, score it
                        if company_index == total_company_words {
                            // match_score += consecutive_score;

                            // Reset for further potential matches in input tokens
                            consecutive_match_count = 0;
                            company_index = 0;
                        }
                    }
                }

                if top_consecutive_match_count > 0 {
                    match_score += top_consecutive_match_count as f32 * weights.continuity;

                    match_score += (consecutive_input_token_char_count as f32
                        / company_name_char_count as f32)
                        * (1.0 - weights.mismatched_letter_penalty);

                    match_score += (top_consecutive_match_count as f32
                        / total_company_words as f32)
                        * (1.0 - weights.mismatched_word_penalty);
                }

                // Skip if the match score is insignificant
                if match_score > weights.match_score_threshold {
                    // Add company name tokens to the filter to prevent basic symbol queries from considering them.
                    // For example, if a company match is for "Apple Hospitality REIT, Inc.," the token "REIT"
                    // should not be treated as a standalone symbol.
                    let tokenized_company_name = tokenize(&company_name);
                    for word in tokenized_company_name {
                        tokenized_filter.insert(word.to_string());
                    }

                    scored_results
                        .entry(symbol.to_string())
                        .and_modify(|e| *e += match_score)
                        .or_insert(match_score);
                } else if match_score > 0.0 {
                    eprintln!(
                        "Discarded symbol: {}; Match Score: {:.4}, Consecutive Matches: {}",
                        symbol, match_score, top_consecutive_match_count
                    );
                }
            }
        }
    }

    // Sort scored_results by score
    let mut sorted_results: Vec<_> = scored_results.into_iter().collect();
    sorted_results.sort_by(|(_, score_a), (_, score_b)| {
        score_b
            .partial_cmp(score_a)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let results: Vec<(String, f32)> = sorted_results
        .clone()
        .into_iter()
        .map(|(symbol, score)| (symbol, score))
        .collect();

    // Iterate over each result and print them
    for (symbol, score) in &results {
        eprintln!(
            "Matched Symbol: {}, Score: {:.4}, Company Name: {:?}",
            symbol, score, symbols_map[symbol]
        );
    }

    // Compute result keys and total weight in a single iteration
    let (result_keys, total_score): (Vec<String>, f32) = sorted_results
        .into_iter()
        .map(|(symbol, score)| (symbol, score))
        .fold((vec![], 0.0), |(mut keys, total), (symbol, score)| {
            keys.push(symbol);
            (keys, total + score)
        });

    eprintln!("Total score: {:.2}", total_score);

    // Return only the keys and the total score
    (result_keys, total_score, tokenized_filter)
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
