use regex::Regex;
use std::collections::{HashMap, HashSet};

const MATCH_SCORE_THRESHOLD: f32 = 0.9;

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
    symbols_map: &HashMap<String, Option<String>>, // Same map for both symbols and company names
) -> Vec<String> {
    let mut matches = HashSet::new();

    // Extract tickers by symbol
    let symbol_matches = extract_tickers_from_symbols(text, symbols_map);
    matches.extend(symbol_matches);

    // Extract tickers by company name
    let company_name_matches = extract_tickers_from_company_names(text, symbols_map);
    matches.extend(company_name_matches);

    // Convert HashSet to Vec and return sorted for consistency
    let mut results: Vec<String> = matches.into_iter().collect();
    results.sort();
    results
}

pub fn extract_tickers_from_symbols(
    text: &str,
    symbols_map: &HashMap<String, Option<String>>,
) -> Vec<String> {
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

pub fn extract_tickers_from_company_names(
    text: &str,
    symbols_map: &HashMap<String, Option<String>>, // Same map for symbols and names
) -> Vec<String> {
    // Step 1: Use regex to split text into sentences based on sentence-ending punctuation
    let sentence_terminator = Regex::new(r"[.!?]\s+").unwrap(); // Match sentence-ending punctuation followed by whitespace
    let sentences: Vec<&str> = sentence_terminator
        .split(text) // Split based on the regex
        .map(str::trim)
        .filter(|s| !s.is_empty()) // Remove empty sentences
        .collect();

    let mut matches: HashMap<String, f32> = HashMap::new();

    for sentence in sentences {
        // Step 2: Normalize tokens within the sentence
        let cleaned_sentence: String = sentence
            .split_whitespace()
            .map(|token| {
                // Remove punctuation only from the ends of tokens, not mid-word
                token.trim_end_matches(|c: char| !c.is_alphanumeric())
            })
            .collect::<Vec<_>>()
            .join(" ");

        // Step 3: Extract uppercase tokens for matching
        let input_tokens: Vec<&str> = cleaned_sentence
            .split_whitespace()
            .filter(|token| token.chars().next().map_or(false, |c| c.is_uppercase()))
            .collect();

        for (symbol, company_name) in symbols_map {
            if let Some(company_name) = company_name {
                // Normalize the company name
                let normalized_company = company_name
                    .replace(|c: char| !c.is_alphanumeric() && c != ' ', " ")
                    .to_lowercase();
                let company_tokens: Vec<&str> = normalized_company.split_whitespace().collect();

                if company_tokens.is_empty() {
                    continue;
                }

                // Step 4: Calculate match score
                let match_score = calculate_match_score(&input_tokens, &company_tokens);

                if match_score >= MATCH_SCORE_THRESHOLD {
                    matches
                        .entry(symbol.clone())
                        .and_modify(|existing_score| {
                            *existing_score = existing_score.max(match_score)
                        })
                        .or_insert(match_score);
                }
            }
        }
    }

    // Step 5: Collect, sort, and return matches
    let mut result: Vec<_> = matches.into_iter().collect();
    result.sort_by(|(sym_a, score_a), (sym_b, score_b)| {
        score_b
            .partial_cmp(score_a)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| sym_a.cmp(sym_b))
    });

    result.into_iter().map(|(symbol, _)| symbol).collect()
}

/// Match scoring algorithm based on token overlap and continuity.
/// Calculate the match score based on token overlap and continuity.
fn calculate_match_score(input_tokens: &[&str], company_tokens: &[&str]) -> f32 {
    let total_input_tokens = input_tokens.len() as f32;
    let total_company_tokens = company_tokens.len() as f32;

    let mut total_matches = 0;
    let mut max_continuous_matches = 0;

    // TODO: Weigh these instead
    // // Filter out the common words from company_tokens
    // let filtered_company_tokens: Vec<&str> = company_tokens
    //     .iter()
    //     .filter(|&&token| !COMMON_WORDS.contains(&token))
    //     .cloned()
    //     .collect();

    let mut i = 0;
    while i < input_tokens.len() {
        let mut current_match = 0;
        for (j, company_token) in company_tokens.iter().enumerate() {
            if i + j < input_tokens.len() {
                let input_token = input_tokens[i + j].to_lowercase();
                let company_token = company_token.to_lowercase();

                // Exact or partial match (relaxed for company names)
                if input_token == company_token {
                    current_match += 2; // Exact match
                } else if company_token.starts_with(&input_token) && input_token.len() > 2 {
                    current_match += 1; // Partial match with prefix
                } else {
                    break;
                }
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

    // Calculate coverage for both input tokens and company tokens
    let coverage_input = total_matches as f32 / total_input_tokens;
    let coverage_company = total_matches as f32 / total_company_tokens;

    // Weighted continuity score
    let continuity_score = max_continuous_matches as f32 / total_company_tokens;

    // Combine scores with balanced weighting
    let match_score =
        (0.5 * continuity_score) + (0.25 * coverage_input) + (0.25 * coverage_company);

    if match_score > 0.0 {
        // Log all metrics for debugging
        eprintln!(
            "Input Tokens: {:?}, Company Tokens: {:?}",
            input_tokens, company_tokens
        );
        eprintln!(
            "Coverage Input: {:.2}, Coverage Company: {:.2}, Continuity Score: {:.2}, Match Score: {:.2}",
            coverage_input, coverage_company, continuity_score, match_score
        );
        eprintln!("------");
    }

    match_score
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
