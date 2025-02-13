use crate::constants::STOP_WORDS;
use crate::types::{Token, TokenCharCode, TokenRef, TokenVector};
use std::char;
use std::collections::HashSet;

/// A utility struct for tokenizing text, with configurable options for
/// processing text documents, ticker symbols, and verbatim parsing.
pub struct Tokenizer {
    /// Whether to process text verbatim without normalization or filtering.
    as_verbatim: bool,

    /// The minimum ratio of uppercase letters required in a token, if applicable.
    min_uppercase_ratio: Option<f32>,

    /// Determines whether the input string should be considered case sensitive.
    /// Disabling this may cause a loss of precision (e.g., the noun "apple" will
    /// match the company name "Apple"). This may cause some tests to fail but
    /// could improve handling of certain input fields.
    case_sensitive: bool,

    /// Preprocessed stop words for filtering tokens.
    pre_processed_stop_words: Option<HashSet<String>>,
}

impl Tokenizer {
    /// Creates a tokenizer configured for parsing ticker symbols.
    ///
    /// Enforces uppercase ratios and does not filter stop words.
    pub fn ticker_symbol_parser() -> Self {
        Self {
            as_verbatim: false,
            min_uppercase_ratio: Some(0.9),
            case_sensitive: false,
            pre_processed_stop_words: None,
        }
    }

    /// Creates a tokenizer configured for parsing arbitrary text documents.
    ///
    /// Normalizes text, filters stop words, and allows tokens with mixed case.
    pub fn text_doc_parser(case_sensitive: bool) -> Self {
        Self {
            as_verbatim: false,
            min_uppercase_ratio: None,
            case_sensitive,
            pre_processed_stop_words: Some(Self::preprocess_stop_words()),
            // TODO: Make configurable
        }
    }

    /// Creates a tokenizer configured for minimal processing.
    ///
    /// Splits text into tokens without normalization or filtering.
    pub fn verbatim_doc_parser() -> Self {
        Self {
            as_verbatim: true,
            min_uppercase_ratio: None,
            case_sensitive: false,
            pre_processed_stop_words: None,
        }
    }

    /// Splits the input text into tokens based on the tokenizer's configuration.
    ///
    /// # Arguments
    /// * `text` - The input text to tokenize.
    ///
    /// # Returns
    /// * A vector of tokens as strings.
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
            .replace(['\n', '\r'], " ") // Normalize line breaks to spaces
            .replace('\r', " ") // Handle potential carriage returns
            .replace("--", " ") // Replace standalone double hyphens
            .replace(',', " ") // Normalize commas to spaces
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

                let passes_any_caps_or_is_number = if self.case_sensitive {
                    word.chars().any(|c| c.is_uppercase()) || word.chars().all(|c| c.is_numeric())
                } else {
                    true
                };

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

    /// Calculates the ratio of uppercase letters in a word.
    ///
    /// # Arguments
    /// * `word` - A reference to the token to analyze.
    ///
    /// # Returns
    /// * The ratio of uppercase letters in the word as a float.
    fn calc_uppercase_ratio(&self, word: &TokenRef) -> f32 {
        let total_chars = word.chars().count() as f32;
        if total_chars == 0.0 {
            return 0.0;
        }
        let uppercase_chars = word.chars().filter(|c| c.is_uppercase()).count() as f32;
        uppercase_chars / total_chars
    }

    /// Preprocesses the stop words by converting them to uppercase.
    ///
    /// # Returns
    /// * A `HashSet` containing the preprocessed stop words.
    fn preprocess_stop_words() -> HashSet<String> {
        STOP_WORDS.iter().map(|word| word.to_uppercase()).collect()
    }

    /// Converts tokens to character code vectors.
    ///
    /// # Arguments
    /// * `text` - A reference to the text to tokenize.
    ///
    /// # Returns
    /// * A vector of character code vectors.
    pub fn tokenize_to_charcode_vectors(&self, text: &TokenRef) -> Vec<TokenVector> {
        self.tokenize(text)
            .iter() // Use the existing `tokenize` function to get tokens
            .map(|token| Tokenizer::token_to_charcode_vector(token))
            .collect()
    }

    /// Converts a token to a vector of character codes.
    ///
    /// # Arguments
    /// * `token` - A reference to the token to convert.
    ///
    /// # Returns
    /// * A vector of character codes representing the token.
    pub fn token_to_charcode_vector(token: &TokenRef) -> TokenVector {
        token.chars().map(|c| c as TokenCharCode).collect()
    }

    /// Converts multiple tokens to vectors of character codes.
    ///
    /// # Arguments
    /// * `tokens` - A slice of token references to convert.
    ///
    /// # Returns
    /// * A vector of character code vectors.
    pub fn tokens_to_charcode_vectors(tokens: &[&TokenRef]) -> Vec<TokenVector> {
        tokens
            .iter()
            .map(|token| Tokenizer::token_to_charcode_vector(token))
            .collect()
    }

    /// Converts a vector of character codes to a token.
    ///
    /// # Arguments
    /// * `charcodes` - A reference to the vector of character codes.
    ///
    /// # Returns
    /// * A token reconstructed from the character codes.
    pub fn charcode_vector_to_token(charcodes: &TokenVector) -> Token {
        charcodes
            .iter()
            .map(|&code| char::from_u32(code).unwrap_or('\u{FFFD}')) // Convert code to char, using '�' as a fallback
            .collect()
    }

    /// Converts multiple vectors of character codes to tokens.
    ///
    /// # Arguments
    /// * `charcode_vectors` - A slice of character code vectors.
    ///
    /// # Returns
    /// * A vector of tokens reconstructed from the character code vectors.
    pub fn charcode_vectors_to_tokens(charcode_vectors: &[TokenVector]) -> Vec<Token> {
        charcode_vectors
            .iter()
            .map(Tokenizer::charcode_vector_to_token)
            .collect() // Collect the resulting strings into a Vec<String>
    }
}
