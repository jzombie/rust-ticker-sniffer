use crate::constants::TLD_LIST;

// TODO: Provide optional STOP_WORD filtering
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
    token.chars().map(|c| c as u32).collect()
}

pub fn charcode_vector_to_token(charcodes: &[u32]) -> String {
    charcodes
        .iter()
        .map(|&code| char::from_u32(code).unwrap_or('\u{FFFD}')) // Convert code to char, using '�' as a fallback
        .collect()
}

pub fn tokenize_to_charcode_vectors(text: &str) -> Vec<Vec<u32>> {
    tokenize(text)
        .into_iter() // Use the existing `tokenize` function to get tokens
        .map(|token| token_to_charcode_vector(&token)) // Convert each token to char code vectors
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
