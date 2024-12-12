use crate::constants::STOP_WORDS;
use crate::constants::TLD_LIST;
use crate::types::TokenizerVectorTokenType;
use std::char;
use std::collections::HashSet;

pub struct Tokenizer {
    pub require_first_letter_caps: bool,
    pub min_uppercase_ratio: Option<f32>,
    pub hyphens_as_potential_multiple_words: bool,
    pub filter_stop_words: bool,
    pre_processed_stop_words: Option<HashSet<String>>,
}

impl Tokenizer {
    /// Configuration specifically for ticker symbol parsing
    pub fn ticker_symbol_parser() -> Self {
        let filter_stop_words = true;
        Self {
            require_first_letter_caps: false,
            min_uppercase_ratio: Some(0.9),
            hyphens_as_potential_multiple_words: false,
            filter_stop_words,
            pre_processed_stop_words: if filter_stop_words {
                Some(Self::preprocess_stop_words())
            } else {
                None
            },
        }
    }

    /// Configuration for arbitrary text doc parsing
    pub fn text_doc_parser() -> Self {
        let filter_stop_words = true;
        Self {
            require_first_letter_caps: true,
            min_uppercase_ratio: None,
            hyphens_as_potential_multiple_words: true,
            filter_stop_words,
            pre_processed_stop_words: if filter_stop_words {
                Some(Self::preprocess_stop_words())
            } else {
                None
            },
        }
    }

    /// Tokenizer function to split the text into individual tokens.
    pub fn tokenize(&self, text: &str) -> Vec<String> {
        // Helper function to calculate uppercase ratio
        fn uppercase_ratio(word: &str) -> f32 {
            let total_chars = word.chars().count() as f32;
            if total_chars == 0.0 {
                return 0.0;
            }
            let uppercase_chars = word.chars().filter(|c| c.is_uppercase()).count() as f32;
            uppercase_chars / total_chars
        }

        // Preprocess and tokenize the text
        text.replace("-\n", "") // Merge hyphenated words across lines
            .replace('\n', " ") // Normalize line breaks to spaces
            .replace('\r', " ") // Handle potential carriage returns
            .replace("--", " ") // Replace standalone double hyphens
            .replace(",", " ") // Normalize commas to spaces
            .split_whitespace() // Split into words
            // Filter by original capitalization
            .filter(|word| {
                // Apply uppercase ratio filter and first letter caps requirement
                let passes_uppercase_ratio = self
                    .min_uppercase_ratio
                    .map_or(true, |ratio| uppercase_ratio(word) >= ratio);

                let passes_first_letter_caps = if self.require_first_letter_caps {
                    word.chars()
                        .find(|c| c.is_alphanumeric()) // Find the first alphanumeric character
                        .map_or(false, |c| c.is_uppercase()) // Ensure it is uppercase
                } else {
                    true
                };

                passes_uppercase_ratio && passes_first_letter_caps
            })
            // Remove TLD extensions (i.e. so `Amazon` and `Amazon.com` are treated the same)
            .map(|word| {
                // Normalize TLDs and strip them if found
                word.rsplit_once('.')
                    .filter(|(_, tld)| TLD_LIST.contains(&tld.to_lowercase().as_str()))
                    .map_or_else(|| word.to_string(), |(base, _)| base.to_string())
            })
            // Remove possessive endings
            .map(|word| {
                let stripped = word.replace("'s", "").replace("s'", "");

                stripped
                    .chars()
                    .filter(|c| c.is_alphanumeric())
                    .collect::<String>()
            })
            // Handle hyphenated words
            .flat_map(|word| {
                word.split('-')
                    .map(|part| {
                        part.chars()
                            .filter(|c| c.is_alphanumeric())
                            .collect::<String>()
                    })
                    .collect::<Vec<_>>()
                    .into_iter()
                    .chain(if self.hyphens_as_potential_multiple_words {
                        Vec::new().into_iter()
                    } else {
                        vec![word.replace('-', "")].into_iter()
                    })
            })
            // Filter to alphanumeric and uppercase
            .map(|word| {
                word.chars()
                    .filter(|c| c.is_alphanumeric())
                    .collect::<String>() // Collect filtered characters into a String
                    .to_uppercase() // Convert to uppercase
            })
            .filter(|word| {
                // Use preprocessed stop words for filtering
                !self.filter_stop_words
                    || !self
                        .pre_processed_stop_words
                        .as_ref()
                        .map_or(false, |stop_words| stop_words.contains(word))
            })
            .collect()
    }

    pub fn tokenize_to_charcode_vectors(&self, text: &str) -> Vec<TokenizerVectorTokenType> {
        self.tokenize(text)
            .iter() // Use the existing `tokenize` function to get tokens
            .map(|token| Tokenizer::token_to_charcode_vector(&token))
            .collect()
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

    pub fn token_to_charcode_vector(token: &str) -> TokenizerVectorTokenType {
        token.chars().map(|c| c as u32).collect()
    }

    pub fn charcode_vector_to_token(charcodes: &TokenizerVectorTokenType) -> String {
        charcodes
            .iter()
            .map(|&code| char::from_u32(code).unwrap_or('\u{FFFD}')) // Convert code to char, using '�' as a fallback
            .collect()
    }

    pub fn charcode_vectors_to_tokens(
        charcode_vectors: &Vec<TokenizerVectorTokenType>,
    ) -> Vec<String> {
        charcode_vectors
            .iter()
            .map(|charcodes| Tokenizer::charcode_vector_to_token(charcodes))
            .collect() // Collect the resulting strings into a Vec<String>
    }
}
