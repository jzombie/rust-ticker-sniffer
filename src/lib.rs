use regex::Regex;
use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
};

pub type SymbolsMap<'a> = &'a HashMap<String, Option<String>>;

#[derive(Debug, Clone, Copy)]
pub struct Weights {
    pub mismatched_letter_penalty: f32,
    pub mismatched_word_penalty: f32,
    pub match_score_threshold: f32,
    // pub continuity: f32,
    // pub coverage_input: f32,
    // pub coverage_company: f32,
    // pub match_score_threshold: f32,
    // pub common_word_penalty: f32,
}

impl Weights {
    /// Creates a new `Weights` instance with specified values.
    pub fn new(
        mismatched_letter_penalty: f32,
        mismatched_word_penalty: f32,
        match_score_threshold: f32,
        // continuity: f32,
        // coverage_input: f32,
        // coverage_company: f32,
        // match_score_threshold: f32,
        // common_word_penalty: f32,
    ) -> Self {
        Self {
            mismatched_letter_penalty,
            mismatched_word_penalty,
            match_score_threshold,
            // continuity,
            // coverage_input,
            // coverage_company,
            // match_score_threshold,
            // common_word_penalty,
        }
    }

    // Creates a new `Weights` instance with random values around a given base.
    // pub fn random(base: f32, range: f32) -> Self {
    //     use rand::Rng;
    //     let mut rng = rand::thread_rng();
    //     Self {
    //         continuity: base + rng.gen_range(-range..range),
    //         coverage_input: base + rng.gen_range(-range..range),
    //         coverage_company: base + rng.gen_range(-range..range),
    //     }
    // }

    // Normalizes the weights so that they sum up to the specified `target_sum`.
    // pub fn normalize(&mut self, target_sum: f32) {
    //     // Calculate the current sum of weights
    //     let sum = self.continuity
    //         + self.coverage_input
    //         + self.coverage_company
    //         + self.match_score_threshold
    //         + self.common_word_penalty;

    //     // Scale weights to achieve the target sum
    //     if sum > 0.0 {
    //         let scale = target_sum / sum;
    //         self.continuity *= scale;
    //         self.coverage_input *= scale;
    //         self.coverage_company *= scale;
    //         self.match_score_threshold *= scale;
    //         self.common_word_penalty *= scale;
    //     }
    // }

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

const STOP_WORDS: &[&str] = &[
    "a",
    "about",
    "above",
    "after",
    "again",
    "against",
    "all",
    "am",
    "an",
    "and",
    "any",
    "are",
    "aren't",
    "as",
    "at",
    "be",
    "because",
    "been",
    "before",
    "being",
    "below",
    "between",
    "both",
    "but",
    "by",
    "can",
    "can't",
    "cannot",
    "could",
    "couldn't",
    "did",
    "didn't",
    "do",
    "does",
    "doesn't",
    "doing",
    "don't",
    "down",
    "during",
    "each",
    "few",
    "for",
    "from",
    "further",
    "had",
    "hadn't",
    "has",
    "hasn't",
    "have",
    "haven't",
    "having",
    "he",
    "he'd",
    "he'll",
    "he's",
    "her",
    "here",
    "here's",
    "hers",
    "herself",
    "him",
    "himself",
    "his",
    "how",
    "how's",
    "i",
    "i'd",
    "i'll",
    "i'm",
    "i've",
    "if",
    "in",
    "into",
    "is",
    "isn't",
    "it",
    "it's",
    "its",
    "itself",
    "let's",
    "me",
    "more",
    "most",
    "mustn't",
    "my",
    "myself",
    "no",
    "nor",
    "not",
    "of",
    "off",
    "on",
    "once",
    "only",
    "or",
    "other",
    "ought",
    "our",
    "ours",
    "ourselves",
    "out",
    "over",
    "own",
    "same",
    "shan't",
    "she",
    "she'd",
    "she'll",
    "she's",
    "should",
    "shouldn't",
    "so",
    "some",
    "such",
    "than",
    "that",
    "that's",
    "the",
    "their",
    "theirs",
    "them",
    "themselves",
    "then",
    "there",
    "there's",
    "these",
    "they",
    "they'd",
    "they'll",
    "they're",
    "they've",
    "this",
    "those",
    "through",
    "to",
    "too",
    "under",
    "until",
    "up",
    "very",
    "was",
    "wasn't",
    "we",
    "we'd",
    "we'll",
    "we're",
    "we've",
    "were",
    "weren't",
    "what",
    "what's",
    "when",
    "when's",
    "where",
    "where's",
    "which",
    "while",
    "who",
    "who's",
    "whom",
    "why",
    "why's",
    "with",
    "won't",
    "would",
    "wouldn't",
    "you",
    "you'd",
    "you'll",
    "you're",
    "you've",
    "your",
    "yours",
    "yourself",
    "yourselves",
];

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

// fn calculate_slope(scores: &[f32]) -> f32 {
//     if scores.len() < 2 {
//         return 0.0; // Not enough points to calculate a slope
//     }

//     let first_score = scores[0];
//     let last_score = scores[scores.len() - 1];
//     let n = scores.len() as f32;

//     (last_score - first_score) / (n - 1.0)
// }

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
