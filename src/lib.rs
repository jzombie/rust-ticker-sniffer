use regex::Regex;
use std::collections::{HashMap, HashSet};

pub type SymbolsMap<'a> = &'a HashMap<String, Option<String>>;

#[derive(Debug, Clone, Copy)]
pub struct Weights {
    pub continuity: f32,
    pub coverage_input: f32,
    pub coverage_company: f32,
    pub match_score_threshold: f32,
    pub common_word_penalty: f32,
}

impl Weights {
    /// Creates a new `Weights` instance with specified values.
    pub fn new(
        continuity: f32,
        coverage_input: f32,
        coverage_company: f32,
        match_score_threshold: f32,
        common_word_penalty: f32,
    ) -> Self {
        Self {
            continuity,
            coverage_input,
            coverage_company,
            match_score_threshold,
            common_word_penalty,
        }
    }

    /// Creates a new `Weights` instance with random values around a given base.
    // pub fn random(base: f32, range: f32) -> Self {
    //     use rand::Rng;
    //     let mut rng = rand::thread_rng();
    //     Self {
    //         continuity: base + rng.gen_range(-range..range),
    //         coverage_input: base + rng.gen_range(-range..range),
    //         coverage_company: base + rng.gen_range(-range..range),
    //     }
    // }

    /// Normalizes the weights so that they sum up to the specified `target_sum`.
    pub fn normalize(&mut self, target_sum: f32) {
        // Calculate the current sum of weights
        let sum = self.continuity
            + self.coverage_input
            + self.coverage_company
            + self.match_score_threshold
            + self.common_word_penalty;

        // Scale weights to achieve the target sum
        if sum > 0.0 {
            let scale = target_sum / sum;
            self.continuity *= scale;
            self.coverage_input *= scale;
            self.coverage_company *= scale;
            self.match_score_threshold *= scale;
            self.common_word_penalty *= scale;
        }
    }

    // Applies the weights to the scoring formula.
    // pub fn calculate_score(
    //     &self,
    //     continuity_score: f32,
    //     coverage_input: f32,
    //     coverage_company: f32,
    // ) -> f32 {
    //     (self.continuity * continuity_score)
    //         + (self.coverage_input * coverage_input)
    //         + (self.coverage_company * coverage_company)
    // }
}

const COMMON_WORDS: &[&str] = &[
    "the",
    "corporation",
    "enterprise",
    "inc",
    "company",
    "limited",
    "llc",
    "group",
    "technologies",
];

/// Tokenizer function to split the text into individual tokens.
pub fn tokenize(text: &str) -> Vec<&str> {
    text.split_whitespace().collect()
}

pub fn extract_tickers_from_text(
    text: &str,
    symbols_map: SymbolsMap,
    weights: Weights,
) -> (Vec<String>, f32) {
    let mut matches = HashSet::new();

    // Extract tickers by symbol
    let symbol_matches = extract_tickers_from_symbols(text, symbols_map);
    matches.extend(symbol_matches);

    // Extract tickers by company name
    let (company_name_matches, total_score) =
        extract_tickers_from_company_names(text, symbols_map, weights);
    matches.extend(company_name_matches);

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
            let normalized = token
                .trim_end_matches(|c: char| !c.is_alphanumeric())
                .to_uppercase();

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

fn extract_tickers_from_company_names(
    text: &str,
    symbols_map: SymbolsMap,
    weights: Weights,
) -> (Vec<String>, f32) {
    let normalized_text = text
        // .to_lowercase()
        .replace(|c: char| !c.is_alphanumeric() && c != ' ', " "); // Normalize input

    let input_tokens: Vec<&str> = normalized_text
        .split_whitespace()
        .filter(|token| !COMMON_WORDS.contains(token))
        .collect();

    let mut scored_results: HashMap<String, f32> = HashMap::new();

    if !input_tokens.is_empty() {
        // Filter input tokens: Only consider tokens starting with a capital letter and of sufficient length
        let input_tokens_capitalized: Vec<String> = input_tokens
            .iter()
            .filter(|token| {
                token.chars().next().map_or(false, |c| c.is_uppercase()) && token.len() > 1
            }) // Min length > 1
            .map(|token| token.to_lowercase()) // Normalize to lowercase for matching
            .collect();

        for (symbol, company_name) in symbols_map {
            // Skip already-matched results
            // if seen_ticker_ids.contains(&raw_result.ticker_id) {
            //     continue;
            // }

            // Ensure the company name exists
            if company_name.is_none() || company_name.clone().unwrap().is_empty() {
                continue; // Skip results with no company name
            }

            // Normalize and tokenize the company name
            let company_lower = company_name
                .clone()
                .unwrap()
                .to_lowercase()
                .replace(|c: char| !c.is_alphanumeric() && c != ' ', " ");
            let company_tokens: Vec<String> =
                company_lower.split_whitespace().map(String::from).collect();
            let total_company_words = company_tokens.len();

            if company_tokens.is_empty() {
                continue; // Skip empty names
            }

            let mut match_score = 0.0;
            let mut token_index = 0;

            // Attempt to match consecutive tokens
            while token_index < input_tokens_capitalized.len() {
                let mut consecutive_match_count = 0;
                let mut start_index = None;

                for (company_index, company_token) in company_tokens.iter().enumerate() {
                    if input_tokens_capitalized[token_index] == *company_token {
                        if start_index.is_none() {
                            start_index = Some(company_index);
                        }
                        consecutive_match_count += 1;
                        token_index += 1;
                        if token_index == input_tokens_capitalized.len() {
                            break; // No more tokens to match
                        }
                    } else if start_index.is_some() {
                        break; // End of consecutive match
                    }
                }

                if consecutive_match_count > 0 {
                    let consecutive_score =
                        consecutive_match_count as f32 / total_company_words as f32;
                    match_score += consecutive_score;
                } else {
                    token_index += 1; // Move to the next token if no match was found
                }
            }

            // Penalize results with extra unrelated words
            match_score /= total_company_words as f32;

            // Skip if no meaningful match
            if match_score == 0.0 {
                continue;
            }

            // Aggregate score for this company
            scored_results
                .entry(symbol.to_string())
                .and_modify(|e| *e += match_score)
                .or_insert(match_score);
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
        eprintln!("Matched Symbol: {}, Score: {:.2}", symbol, score);
    }

    // Extract just the keys (symbols) and collect them into a Vec<String>
    let result_keys: Vec<String> = sorted_results
        .into_iter()
        .map(|(symbol, _)| symbol)
        .collect();

    // Return only the keys and the total score
    (result_keys, 0.0)
}

// fn extract_tickers_from_company_names(
//     text: &str,
//     symbols_map: SymbolsMap,
//     weights: Weights,
// ) -> (Vec<String>, f32) {
//     let mut total_score: f32 = 0.0;

//     // Step 1: Use regex to split text into sentences based on sentence-ending punctuation
//     let sentence_terminator = Regex::new(r"[.!?]\s+").unwrap(); // Match sentence-ending punctuation followed by whitespace
//     let sentences: Vec<&str> = sentence_terminator
//         .split(text) // Split based on the regex
//         .map(str::trim)
//         .filter(|s| !s.is_empty()) // Remove empty sentences
//         .collect();

//     let mut matches: HashMap<String, f32> = HashMap::new();

//     for sentence in sentences {
//         // Step 2: Normalize tokens within the sentence
//         let cleaned_sentence: String = sentence
//             .split_whitespace()
//             .map(|token| {
//                 // Remove punctuation only from the ends of tokens, not mid-word
//                 token.trim_end_matches(|c: char| !c.is_alphanumeric())
//             })
//             .collect::<Vec<_>>()
//             .join(" ");

//         // Step 3: Extract uppercase tokens for matching
//         let input_tokens: Vec<&str> = cleaned_sentence
//             .split_whitespace()
//             .filter(|token| token.chars().next().map_or(false, |c| c.is_uppercase()))
//             .collect();

//         for (symbol, company_name) in symbols_map {
//             if let Some(company_name) = company_name {
//                 // Normalize the company name
//                 let normalized_company = company_name
//                     .replace(|c: char| !c.is_alphanumeric() && c != ' ', " ")
//                     .to_lowercase();
//                 let company_tokens: Vec<&str> = normalized_company.split_whitespace().collect();

//                 if company_tokens.is_empty() {
//                     continue;
//                 }

//                 // Step 4: Calculate match score
//                 let match_score = calculate_match_score(&input_tokens, &company_tokens, &weights);

//                 if match_score >= weights.match_score_threshold {
//                     total_score += match_score;

//                     matches
//                         .entry(symbol.clone())
//                         .and_modify(|existing_score| {
//                             *existing_score = existing_score.max(match_score)
//                         })
//                         .or_insert(match_score);
//                 }
//             }
//         }
//     }

//     // Step 5: Collect, sort, and return matches
//     let mut result: Vec<_> = matches.into_iter().collect();
//     result.sort_by(|(sym_a, score_a), (sym_b, score_b)| {
//         score_b
//             .partial_cmp(score_a)
//             .unwrap_or(std::cmp::Ordering::Equal)
//             .then_with(|| sym_a.cmp(sym_b))
//     });

//     (
//         result.into_iter().map(|(symbol, _)| symbol).collect(),
//         total_score,
//     )
// }

/// Match scoring algorithm based on token overlap and continuity.
/// Calculate the match score based on token overlap and continuity.
// fn calculate_match_score(input_tokens: &[&str], company_tokens: &[&str], weights: &Weights) -> f32 {
//     let total_input_tokens = input_tokens.len() as f32;
//     let total_company_tokens = company_tokens.len() as f32;

//     let mut total_matches = 0;
//     let mut max_continuous_matches = 0;
//     let mut common_word_matches = 0;

//     let mut i = 0;
//     while i < input_tokens.len() {
//         let mut current_match = 0;
//         for (j, company_token) in company_tokens.iter().enumerate() {
//             if i + j < input_tokens.len() {
//                 let input_token = input_tokens[i + j].to_lowercase();
//                 let company_token = company_token.to_lowercase();

//                 // Check if the token is a common word and count it
//                 if COMMON_WORDS.contains(&company_token.as_str()) {
//                     common_word_matches += 1;
//                 }

//                 if input_token == company_token {
//                     current_match += 2; // Exact match
//                 } else if company_token.starts_with(&input_token) && input_token.len() > 2 {
//                     current_match += 1; // Partial match with prefix
//                 } else {
//                     break;
//                 }
//             }
//         }
//         if current_match > 0 {
//             total_matches += current_match;
//             max_continuous_matches = max_continuous_matches.max(current_match);
//             i += current_match; // Skip matched tokens
//         } else {
//             i += 1;
//         }
//     }

//     // Calculate coverage for both input tokens and company tokens
//     let coverage_input = total_matches as f32 / total_input_tokens;
//     let coverage_company = total_matches as f32 / total_company_tokens;

//     // Weighted continuity score
//     let continuity_score = max_continuous_matches as f32 / total_company_tokens;

//     // Calculate penalty for common word matches
//     let common_word_penalty = weights.common_word_penalty * common_word_matches as f32;

//     // Combine scores
//     let match_score = (weights.continuity * continuity_score)
//         + (weights.coverage_input * coverage_input)
//         + (weights.coverage_company * coverage_company)
//         - common_word_penalty;

//     // if match_score > 0.0 {
//     // Log all metrics for debugging
//     eprintln!(
//         "Input Tokens: {:?}, Company Tokens: {:?}",
//         input_tokens, company_tokens
//     );
//     eprintln!(
//             "Total Matches: {}, Coverage Input: {:.2}, Coverage Company: {:.2}, Continuity Score: {:.2}, Common Word Penalty: {:.2}, Match Score: {:.2}, Match Score Threshold: {:.2}",
//             total_matches, coverage_input, coverage_company, continuity_score, common_word_penalty, match_score, weights.match_score_threshold
//         );
//     eprintln!("------");
//     // }

//     match_score
// }

pub fn generate_alternative_symbols(query: &str) -> Vec<String> {
    let mut alternatives: Vec<String> = vec![query.to_uppercase()];
    if query.contains('.') {
        alternatives.push(query.replace('.', "-").to_uppercase());
    } else if query.contains('-') {
        alternatives.push(query.replace('-', ".").to_uppercase());
    }
    alternatives
}
