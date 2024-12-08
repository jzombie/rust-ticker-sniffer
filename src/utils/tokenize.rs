use crate::constants::STOP_WORDS;

/// Tokenizer function to split the text into individual tokens.
///
/// Note: This explcitly does not modify the case of the text.
pub fn tokenize(text: &str) -> Vec<String> {
    // Preprocess text: handle hyphenation, line breaks, and cleanup
    let cleaned_text = text
        .replace("-\n", "") // Merge hyphenated words across lines
        .replace('\n', " ") // Normalize line breaks to spaces
        .replace('\r', " ") // Handle potential carriage returns
        .replace("--", " ") // Replace standalone double hyphens
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ") // Ensure proper spacing
        .trim()
        .to_string();

    // Tokenize, process possessives, and uppercase
    cleaned_text
        .split_whitespace()
        .filter(|word| word.chars().any(|c| c.is_uppercase())) // Keep only words with at least one capital letter
        .flat_map(|word| {
            let mut tokens = Vec::new();
            let clean_word: String = word.chars().filter(|c| c.is_alphanumeric()).collect();

            // Check for possessive endings `'s` or `s'` in the original word
            if word.ends_with("'s") || word.ends_with("s'") {
                let base_word: String = word
                    .trim_end_matches("'s")
                    .trim_end_matches("s'")
                    .chars()
                    .filter(|c| c.is_alphanumeric())
                    .collect();
                tokens.push(base_word.to_uppercase()); // Add the base word
            }

            tokens.push(clean_word.to_uppercase()); // Add the original word
            tokens
        })
        .collect()
}

// pub fn tokenize_company_name_query(text: &str) -> Vec<&str> {
//     tokenize(text)
//         .iter()
//         // Only accept first letters that are capitalized
//         .filter(|token| token.chars().next().map_or(false, |c| c.is_uppercase()) && token.len() > 1) // Min length > 1
//         // Remove stop words
//         .filter(|token| {
//             let lowercased = token.to_lowercase();
//             !STOP_WORDS.contains(&lowercased.as_str())
//         })
//         .cloned()
//         .collect()
// }
