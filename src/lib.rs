use std::collections::{HashMap, HashSet};
mod constants;
use crate::constants::STOP_WORDS;
pub mod models;
pub use constants::{
    DEFAULT_BIAS_ADJUSTER_SCORE, DEFAULT_RESULT_BIAS_ADJUSTER_WEIGHTS, DEFAULT_WEIGHTS,
};
pub use models::{CompanyNameTokenRanking, ResultBiasAdjuster, Weights};
pub mod utils;
pub use utils::{
    generate_alternative_symbols, jaccard_similarity_chars, tokenize, tokenize_company_name_query,
};
pub mod types;
pub use types::{SymbolsMap, TickerSymbol};

pub fn extract_tickers_from_text(
    text: &str,
    symbols_map: &SymbolsMap,
) -> (Vec<TickerSymbol>, f32, Vec<CompanyNameTokenRanking>) {
    let result_bias_adjuster =
        ResultBiasAdjuster::from_weights(DEFAULT_RESULT_BIAS_ADJUSTER_WEIGHTS);

    extract_tickers_from_text_with_custom_weights(
        &text,
        &symbols_map,
        DEFAULT_WEIGHTS,
        &result_bias_adjuster,
    )
}

pub fn extract_tickers_from_text_with_custom_weights(
    text: &str,
    symbols_map: &SymbolsMap,
    weights: Weights,
    result_bias_adjuster: &ResultBiasAdjuster,
) -> (Vec<TickerSymbol>, f32, Vec<CompanyNameTokenRanking>) {
    let mut matches = HashSet::new();

    // Extract tickers by company name
    let (company_name_matches, total_score, tokenized_filter, company_rankings) =
        extract_tickers_from_company_names(text, symbols_map, weights, result_bias_adjuster);
    let company_name_match_count = company_name_matches.len();

    matches.extend(company_name_matches);

    let filtered_text: String = text
        .split_whitespace()
        .filter(|word| !tokenized_filter.contains(&word.to_string()))
        .collect::<Vec<&str>>()
        .join(" ");

    // Extract tickers by symbol
    let mut symbol_matches = extract_tickers_from_symbols(&filtered_text, symbols_map);
    let symbol_match_count = symbol_matches.len();

    // Calculate the ratio of symbol matches to company name matches
    let match_ratio = if company_name_match_count > 0 {
        symbol_match_count as f32 / company_name_match_count as f32
    } else {
        f32::MAX
    };

    // eprintln!(
    //     "Symbol match ratio: {:4} {:4}, Symbol match count: {}, Company name match count: {}",
    //     match_ratio, weights.stop_word_filter_ratio, symbol_match_count, company_name_match_count
    // );

    // Decide whether to prune symbol matches based on the ratio and weight
    if match_ratio < weights.stop_word_filter_ratio {
        symbol_matches.retain(|symbol| {
            if STOP_WORDS.contains(&symbol.to_lowercase().as_str()) {
                false // Remove stop words entirely
            } else {
                true // Keep non-stop words
            }
        });
    }

    matches.extend(symbol_matches.clone());

    let abbreviation_matches =
        extract_tickers_from_abbreviations(&filtered_text, symbols_map, weights);
    matches.extend(abbreviation_matches);

    // Convert HashSet to Vec and return sorted for consistency
    let mut results: Vec<TickerSymbol> = matches.into_iter().collect();
    results.sort();

    (results, total_score, company_rankings)
}

fn extract_tickers_from_symbols(text: &str, symbols_map: &SymbolsMap) -> Vec<TickerSymbol> {
    let mut matches = HashSet::new();
    let tokens = tokenize(text);

    for token in tokens {
        // Only match on tokens that are fully upper-case
        if token == token.to_uppercase() {
            let normalized = token.to_string();

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
    symbols_map: &SymbolsMap,
    weights: Weights,
) -> Vec<TickerSymbol> {
    let mut matches = HashSet::new();

    let input_tokens_capitalized: Vec<&str> = tokenize_company_name_query(text);

    for token in input_tokens_capitalized {
        // Normalize the token to lowercase
        let lc_token = token.to_lowercase();

        let token_length = token.len();

        for (symbol, _company_name) in symbols_map {
            let symbol_length = symbol.len();

            // let lc_company_name = company_name.to_lowercase();
            let lc_symbol = symbol.to_lowercase();

            // Check if the token starts with part of the company name
            if lc_token.starts_with(&lc_symbol) {
                let abbr_perc = symbol_length as f32 / token_length as f32;

                if abbr_perc > weights.abbreviation_match_threshold {
                    matches.insert(symbol.to_string());
                }
            }
        }
    }

    matches.into_iter().collect()
}

fn extract_tickers_from_company_names(
    text: &str,
    symbols_map: &SymbolsMap,
    weights: Weights,
    result_bias_adjuster: &ResultBiasAdjuster,
) -> (
    Vec<TickerSymbol>,
    f32,
    HashSet<String>,
    Vec<CompanyNameTokenRanking>,
) {
    let mut scored_results: HashMap<TickerSymbol, f32> = HashMap::new();

    // Note: This is not a vector of symbols; maybe explicit type defining could make this more apparent
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

                // TODO: This could run through "tokenize" as well if it doesn't incur a perf penalty for some reason
                //
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

                let mut result_bias_adjuster_score: f32 = 0.0;

                // TODO: Ideally, this should be set inide the following block,
                // but I'm fighting with the borrower-checker to try to retain this value
                let lc_norm_input_string: String = top_company_index_token_index_map
                    .values()
                    .map(|&index| input_tokens_capitalized[index])
                    .collect::<Vec<&str>>()
                    .join(" ")
                    .to_lowercase();

                if top_consecutive_match_count > 0 {
                    match_score +=
                        top_consecutive_match_count as f32 * weights.consecutive_match_weight;

                    let lc_norm_company_string: String = company_tokens.join(" ");

                    consecutive_jaccard_similarity =
                        jaccard_similarity_chars(&lc_norm_input_string, &lc_norm_company_string);

                    match_score +=
                        consecutive_jaccard_similarity * (1.0 - weights.letter_mismatch_penalty);

                    match_score += (top_consecutive_match_count as f32
                        / total_company_words as f32)
                        * (1.0 - weights.word_mismatch_penalty);

                    // TODO: Apply configurable weighting?
                    result_bias_adjuster_score =
                        result_bias_adjuster.score(&lc_norm_input_string, &company_tokens);

                    // Scale match_score by result_bias_adjuster_score
                    match_score *= result_bias_adjuster_score * (1.0 / DEFAULT_BIAS_ADJUSTER_SCORE);
                }

                if lc_norm_input_string.len() > 0 {
                    // if match_score > weights.minimum_match_score {
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
                        result_bias_adjuster_score,
                        context_query_string: lc_norm_input_string,
                        context_company_tokens: company_tokens,
                    };

                    company_rankings.push(company_ranking);

                    eprintln!(
                        "Company name: {}, Context attention score: {}",
                        company_name, result_bias_adjuster_score
                    );
                }

                // } else if match_score > 0.0 {
                //     eprintln!(
                //         "Discarded symbol: {}; Match Score: {:.4}, Consecutive Matches: {}, Jaccard: {}",
                //         symbol, match_score, top_consecutive_match_count, consecutive_jaccard_similarity
                //     );
                // }
            }
        }
    }

    for company_ranking in &company_rankings {
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

    let mut result_ticker_symbols = Vec::new();
    let mut total_score = 0.0;

    for (symbol, score) in scored_results.clone() {
        // Print the result
        eprintln!(
            "Matched Symbol: {}, Score: {:.4}, Company Name: {:?}",
            symbol, score, symbols_map[&symbol]
        );

        // Update the result keys and total score
        result_ticker_symbols.push(symbol);
        total_score += score;
    }

    eprintln!("Total score: {:.2}", total_score);

    (
        result_ticker_symbols,
        total_score,
        tokenized_filter,
        company_rankings,
    )
}
