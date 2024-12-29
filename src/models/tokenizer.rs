use crate::types::TokenizerVectorToken;
use std::char;

pub struct Tokenizer {}

// TODO: Company name and alternate names may not match the predefined rules (some may
// be lowercase due to branding, etc.). The tokenizer should include a configuration which
// uses those [potentially special-case] names as a corpus of items to tokenize against.
impl Tokenizer {
    pub fn new() -> Self {
        Self {}
    }

    /// Tokenizer function to split the text into individual tokens.
    pub fn tokenize(&self, text: &str) -> Vec<String> {
        // Helper function to calculate uppercase ratio
        // fn uppercase_ratio(word: &str) -> f32 {
        //     let total_chars = word.chars().count() as f32;
        //     if total_chars == 0.0 {
        //         return 0.0;
        //     }
        //     let uppercase_chars = word.chars().filter(|c| c.is_uppercase()).count() as f32;
        //     uppercase_chars / total_chars
        // }

        // Preprocess and tokenize the text
        text.replace("-\n", "") // Merge hyphenated words across lines
            .replace('\n', " ") // Normalize line breaks to spaces
            .replace('\r', " ") // Handle potential carriage returns
            .replace("--", " ") // Replace standalone double hyphens
            .replace(",", " ") // Normalize commas to spaces
            .split_whitespace() // Split into words
            // Remove possessive endings
            .map(|word| {
                let stripped = word.replace("'s", "").replace("s'", "");

                stripped
                    .chars()
                    .filter(|c| c.is_alphanumeric())
                    .collect::<String>()
            })
            // Split hyphenated words into multiple words
            .flat_map(|word| {
                let parts: Vec<String> = word
                    .split('-')
                    .map(|part| {
                        part.chars()
                            .filter(|c| c.is_alphanumeric())
                            .collect::<String>()
                    })
                    .collect();

                if parts.len() > 1 {
                    parts.into_iter() // Only split into parts if there are multiple segments
                } else {
                    vec![word.replace('-', "")].into_iter() // Otherwise, use the whole word
                }
            })
            // Filter to alphanumeric and uppercase
            .map(|word| {
                word.chars()
                    .filter(|c| c.is_alphanumeric())
                    .collect::<String>() // Collect filtered characters into a String
                    .to_uppercase() // Convert to uppercase
            })
            // Skip empty words
            .filter(|word| !word.is_empty())
            .collect()
    }

    pub fn tokenize_to_charcode_vectors(&self, text: &str) -> Vec<TokenizerVectorToken> {
        self.tokenize(text)
            .iter() // Use the existing `tokenize` function to get tokens
            .map(|token| Tokenizer::token_to_charcode_vector(&token))
            .collect()
    }

    pub fn token_to_charcode_vector(token: &str) -> TokenizerVectorToken {
        token.chars().map(|c| c as u32).collect()
    }

    pub fn tokens_to_charcode_vectors(tokens: &Vec<&str>) -> Vec<TokenizerVectorToken> {
        tokens
            .iter()
            .map(|token| Tokenizer::token_to_charcode_vector(token))
            .collect()
    }

    pub fn charcode_vector_to_token(charcodes: &TokenizerVectorToken) -> String {
        charcodes
            .iter()
            .map(|&code| char::from_u32(code).unwrap_or('\u{FFFD}')) // Convert code to char, using '�' as a fallback
            .collect()
    }

    pub fn charcode_vectors_to_tokens(charcode_vectors: &Vec<TokenizerVectorToken>) -> Vec<String> {
        charcode_vectors
            .iter()
            .map(|charcodes| Tokenizer::charcode_vector_to_token(charcodes))
            .collect() // Collect the resulting strings into a Vec<String>
    }
}
