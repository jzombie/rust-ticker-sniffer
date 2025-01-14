use crate::constants::STOP_WORDS;
use crate::types::{Token, TokenRef, TokenVector};
use std::char;
use std::collections::HashSet;

pub struct Tokenizer {
    as_verbatim: bool,
    min_uppercase_ratio: Option<f32>,
    pre_processed_stop_words: Option<HashSet<String>>,
}

impl Tokenizer {
    /// Configuration specifically for ticker symbol parsing
    pub fn ticker_symbol_parser() -> Self {
        Self {
            as_verbatim: false,
            min_uppercase_ratio: Some(0.9),
            pre_processed_stop_words: None,
        }
    }
    /// Configuration for arbitrary text doc parsing
    pub fn text_doc_parser() -> Self {
        Self {
            as_verbatim: false,
            min_uppercase_ratio: None,
            pre_processed_stop_words: Some(Self::preprocess_stop_words()),
        }
    }

    /// Configuration for minimal processing
    pub fn verbatim_doc_parser() -> Self {
        Self {
            as_verbatim: true,
            min_uppercase_ratio: None,
            pre_processed_stop_words: None,
        }
    }

    /// Tokenizer function to split the text into individual tokens.
    pub fn tokenize(&self, text: &str) -> Vec<Token> {
        if self.as_verbatim {
            return text
                .split_whitespace() // Split into words
                .map(|word| word.to_string()) // Convert each word into a Token object
                .collect();
        }

        let stop_words = self.pre_processed_stop_words.as_ref();

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
            .filter(|word| {
                // Apply uppercase ratio filter and any capital letter requirement
                let passes_uppercase_ratio = self
                    .min_uppercase_ratio
                    .map_or(true, |ratio| self.calc_uppercase_ratio(word) >= ratio);

                let passes_any_caps_or_is_number =
                    word.chars().any(|c| c.is_uppercase()) || word.chars().all(|c| c.is_numeric());

                passes_uppercase_ratio && passes_any_caps_or_is_number
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
            // Skip empty words and stop words
            .filter(|word| !word.is_empty() && stop_words.map_or(true, |sw| !sw.contains(word)))
            .collect()
    }

    fn calc_uppercase_ratio(&self, word: &TokenRef) -> f32 {
        let total_chars = word.chars().count() as f32;
        if total_chars == 0.0 {
            return 0.0;
        }
        let uppercase_chars = word.chars().filter(|c| c.is_uppercase()).count() as f32;
        uppercase_chars / total_chars
    }

    /// Pre-process the stop words by removing non-alphanumeric characters and converting to uppercase
    fn preprocess_stop_words() -> HashSet<String> {
        STOP_WORDS
            .iter()
            .map(|word| {
                word.chars()
                    .filter(|c| c.is_alphanumeric())
                    .collect::<String>()
                    .to_uppercase()
            })
            .collect()
    }

    pub fn tokenize_to_charcode_vectors(&self, text: &TokenRef) -> Vec<TokenVector> {
        self.tokenize(text)
            .iter() // Use the existing `tokenize` function to get tokens
            .map(|token| Tokenizer::token_to_charcode_vector(&token))
            .collect()
    }

    pub fn token_to_charcode_vector(token: &TokenRef) -> TokenVector {
        token.chars().map(|c| c as u32).collect()
    }

    pub fn tokens_to_charcode_vectors(tokens: &Vec<&TokenRef>) -> Vec<TokenVector> {
        tokens
            .iter()
            .map(|token| Tokenizer::token_to_charcode_vector(token))
            .collect()
    }

    pub fn charcode_vector_to_token(charcodes: &TokenVector) -> Token {
        charcodes
            .iter()
            .map(|&code| char::from_u32(code).unwrap_or('\u{FFFD}')) // Convert code to char, using '�' as a fallback
            .collect()
    }

    pub fn charcode_vectors_to_tokens(charcode_vectors: &Vec<TokenVector>) -> Vec<Token> {
        charcode_vectors
            .iter()
            .map(|charcodes| Tokenizer::charcode_vector_to_token(charcodes))
            .collect() // Collect the resulting strings into a Vec<String>
    }
}
