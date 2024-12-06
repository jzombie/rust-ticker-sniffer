use crate::constants::STOP_WORDS;

/// Tokenizer function to split the text into individual tokens.
///
/// Note: This explcitly does not modify the case of the text.
pub fn tokenize(text: &str) -> Vec<&str> {
    text.split_whitespace()
        // Remove non-alphanumeric characters from end of string (and not in any other position)
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
