use std::collections::{HashMap, HashSet};
mod constants;
use crate::constants::STOP_WORDS;
pub mod models;
pub use models::Weights;

pub type SymbolsMap<'a> = &'a HashMap<String, Option<String>>;

/// Tokenizer function to split the text into individual tokens.
///
/// Note: This explcitly does not modify the case of the text.
pub fn tokenize(text: &str) -> Vec<&str> {
    text.split_whitespace()
        .map(|word| word.trim_end_matches(|c: char| !c.is_alphanumeric()))
        .collect()
}

pub fn tokenize_company_name_query(text: &str) -> Vec<&str> {
    tokenize(text)
        .iter()
        // Only accept first letters that are capitalized
        .filter(|token| token.chars().next().map_or(false, |c| c.is_uppercase()) && token.len() > 1) // Min length > 1
        // Remove stop words
        .filter(|token| {
            let lowercased = token.to_lowercase();
            !STOP_WORDS.contains(&lowercased.as_str())
        })
        .cloned()
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

    let input_tokens_capitalized: Vec<&str> = tokenize_company_name_query(text);

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

#[derive(Clone)]
struct CompanyNameTokenRanking {
    ticker_symbol: String,
    company_name: String,
    input_token_indices: Vec<usize>,
    consecutive_match_count: usize,
    consecutive_jaccard_similarity: f32,
    match_score: f32,
}

fn extract_tickers_from_company_names(
    text: &str,
    symbols_map: SymbolsMap,
    weights: Weights,
) -> (Vec<String>, f32, HashSet<String>) {
    let mut scored_results: HashMap<String, f32> = HashMap::new();
    let mut tokenized_filter: HashSet<String> = HashSet::new();

    let input_tokens_capitalized: Vec<&str> = tokenize_company_name_query(text);
    let mut company_rankings: Vec<CompanyNameTokenRanking> = Vec::new();

    let mut input_token_index_to_top_company_ranking_map: HashMap<
        usize,
        Vec<CompanyNameTokenRanking>,
    > = HashMap::new();

    if !input_tokens_capitalized.is_empty() {
        // Filter input tokens: Only consider tokens starting with a capital letter and of sufficient length, then remove stop words

        for (symbol, company_name) in symbols_map {
            // Skip entries without a valid company name
            if let Some(company_name) = company_name {
                if company_name.is_empty() {
                    continue;
                }

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

                let mut match_score = 0.0;

                // Single pass through input tokens
                let mut company_index = 0;

                // let mut seen_tokens: HashSet<String> = HashSet::new();

                // let mut input_token_indices: Vec<usize> = Vec::new();
                let mut company_index_token_index_map: HashMap<usize, usize> = HashMap::new();
                let mut top_company_index_token_index_map: HashMap<usize, usize> = HashMap::new();

                for (input_token_position, input_token) in
                    input_tokens_capitalized.iter().enumerate()
                {
                    let lc_input_token = input_token.to_lowercase();

                    if &lc_input_token != &company_tokens[company_index] {
                        // Note: This reset is perfomrmed before the following `if` statement to fix an issue
                        // where a phrase with `Apple Apple Hopitality REIT` are identified as separate companies.
                        // Previously, the consecutive match mechanism would get out of sync and identify
                        // `Apple Hospitality REIT` with a low score.
                        consecutive_match_count = 0;

                        company_index = 0;

                        company_index_token_index_map.remove(&company_index);
                    }

                    if &lc_input_token == &company_tokens[company_index] {
                        // input_token_indices.push(input_token_position);
                        company_index_token_index_map.insert(company_index, input_token_position);

                        // Match found, increment the company pointer
                        consecutive_match_count += 1;

                        company_index += 1;

                        if consecutive_match_count > top_consecutive_match_count {
                            top_consecutive_match_count = consecutive_match_count;

                            top_company_index_token_index_map =
                                company_index_token_index_map.clone();
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

                let mut consecutive_jaccard_similarity: f32 = 0.0;

                if top_consecutive_match_count > 0 {
                    match_score += top_consecutive_match_count as f32 * weights.continuity;

                    let lc_norm_input_string: String = top_company_index_token_index_map
                        .values()
                        .map(|&index| input_tokens_capitalized[index])
                        .collect::<Vec<&str>>()
                        .join(" ")
                        .to_lowercase();

                    let lc_norm_company_string: String = company_tokens.join(" ");

                    consecutive_jaccard_similarity =
                        jaccard_similarity_chars(&lc_norm_input_string, &lc_norm_company_string);

                    match_score +=
                        consecutive_jaccard_similarity * (1.0 - weights.mismatched_letter_penalty);

                    match_score += (top_consecutive_match_count as f32
                        / total_company_words as f32)
                        * (1.0 - weights.mismatched_word_penalty);
                }

                if match_score > weights.match_score_threshold {
                    let company_ranking: CompanyNameTokenRanking = CompanyNameTokenRanking {
                        ticker_symbol: symbol.to_string(),
                        company_name: company_name.to_string(),
                        input_token_indices: top_company_index_token_index_map
                            .values()
                            .cloned()
                            .collect(),
                        consecutive_match_count: top_consecutive_match_count,
                        consecutive_jaccard_similarity,
                        match_score,
                    };

                    company_rankings.push(company_ranking);
                } else if match_score > 0.0 {
                    eprintln!(
                        "Discarded symbol: {}; Match Score: {:.4}, Consecutive Matches: {}, Jaccard: {}",
                        symbol, match_score, top_consecutive_match_count, consecutive_jaccard_similarity
                    );
                }
            }
        }
    }

    for company_ranking in company_rankings {
        if company_ranking.match_score > 0.0 {
            eprintln!(
                "Company name: {}; Match Score: {}; Input Token Positions: {:?}; Jaccard: {}",
                company_ranking.company_name,
                company_ranking.match_score,
                company_ranking.input_token_indices,
                company_ranking.consecutive_jaccard_similarity
            );

            for input_token_index in company_ranking.input_token_indices.iter() {
                // Check if this token index already has an entry
                if let Some(existing_rankings) =
                    input_token_index_to_top_company_ranking_map.get_mut(input_token_index)
                {
                    // Find the highest score in the current list
                    let max_score = existing_rankings
                        .iter()
                        .map(|ranking| ranking.match_score)
                        .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
                        .unwrap_or(0.0);

                    if company_ranking.match_score > max_score {
                        // New higher score, replace the existing vector
                        *existing_rankings = vec![company_ranking.clone()];
                    } else if (company_ranking.match_score - max_score).abs() < f32::EPSILON {
                        // Scores are equal, append the new ranking
                        existing_rankings.push(company_ranking.clone());
                    }
                } else {
                    // No entry exists, insert this company ranking as a new vector
                    input_token_index_to_top_company_ranking_map
                        .insert(*input_token_index, vec![company_ranking.clone()]);
                }
            }
        }
    }

    for (_, company_rankings) in input_token_index_to_top_company_ranking_map {
        for company_ranking in company_rankings {
            // Tokenize the company name and add tokens to the filter
            let tokenized_company_name = tokenize(&company_ranking.company_name);
            for word in tokenized_company_name {
                tokenized_filter.insert(word.to_string());
            }

            // Update the scored_results with the match score
            scored_results
                .entry(company_ranking.ticker_symbol.to_string())
                .and_modify(|e| *e += company_ranking.match_score)
                .or_insert(company_ranking.match_score);
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

/// Compute the Jaccard similarity between two strings by treating characters as sets.
pub fn jaccard_similarity_chars(s1: &str, s2: &str) -> f32 {
    let set1: HashSet<_> = s1.chars().collect();
    let set2: HashSet<_> = s2.chars().collect();

    let intersection_size = set1.intersection(&set2).count();
    let union_size = set1.union(&set2).count();

    if union_size == 0 {
        0.0 // Avoid division by zero if both sets are empty
    } else {
        intersection_size as f32 / union_size as f32
    }
}
