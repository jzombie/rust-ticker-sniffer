// use crate::constants::STOP_WORDS;

/// Tokenizer function to split the text into individual tokens.
///
/// Note: This explcitly does not modify the case of the text.
pub fn tokenize(text: &str) -> Vec<String> {
    // List of common TLDs for robust handling
    const TLD_LIST: [&str; 5] = ["com", "org", "net", "edu", "gov"];

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

    // Tokenize, process possessives, handle web addresses, and uppercase
    cleaned_text
        .split_whitespace()
        .filter(|word| word.chars().any(|c| c.is_uppercase())) // Keep only words with at least one capital letter
        .flat_map(|word| {
            let mut tokens = Vec::new();

            // Handle web addresses
            if let Some((base, tld)) = word.rsplit_once('.') {
                if TLD_LIST.contains(&tld.to_lowercase().as_str()) {
                    let cleaned_word: String = format!("{}{}", base, tld); // Concatenate base and TLD
                    tokens.push(cleaned_word.to_uppercase());

                    // Add the base word as another token
                    tokens.push(
                        base.chars()
                            .filter(|c| c.is_alphanumeric())
                            .collect::<String>()
                            .to_uppercase(),
                    );

                    return tokens;
                }
            }

            // Handle possessive endings `'s` or `s'`
            let clean_word: String = word.chars().filter(|c| c.is_alphanumeric()).collect();
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

pub fn token_to_charcode_vector(token: &str) -> Vec<u32> {
    token
        .chars()
        .map(|c| c.to_ascii_lowercase() as u32) // Convert to lowercase and get char code
        .collect()
}

// TODO: Remove
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
