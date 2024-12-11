use crate::constants::STOP_WORDS;
use crate::constants::TLD_LIST;
use crate::types::TokenizerVectorTokenType;

#[derive(Copy, Clone)]
pub struct Tokenizer {
    pub min_uppercase_ratio: Option<f32>,
    pub hyphens_as_potential_multiple_words: bool,
    pub require_first_letter_caps: bool,
    pub filter_stop_words: bool,
}

impl Tokenizer {
    /// Configuration specifically for ticker symbol parsing
    pub fn ticker_symbol_parser() -> Self {
        Self {
            min_uppercase_ratio: Some(0.9),
            hyphens_as_potential_multiple_words: false,
            require_first_letter_caps: false,
            filter_stop_words: false,
        }
    }

    /// Configuration for arbitrary text doc parsing
    pub fn text_doc_parser() -> Self {
        Self {
            min_uppercase_ratio: None,
            hyphens_as_potential_multiple_words: true,
            require_first_letter_caps: true,
            filter_stop_words: true,
        }
    }

    // TODO: Provide optional STOP_WORD filtering?
    /// Tokenizer function to split the text into individual tokens.
    ///
    /// Note: This explcitly does not modify the case of the text.

    pub fn tokenize(self, text: &str) -> Vec<String> {
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
            .replace(",", " ") // Normalize commas to periods
            .split_whitespace() // Split into words
            .filter(|word| {
                if self.require_first_letter_caps {
                    word.chars().next().map_or(false, |c| c.is_uppercase())
                } else {
                    true
                }
            })
            .map(|word| {
                // Remove possessive endings ('s or s') and normalize
                let stripped = word.replace("'s", "").replace("s'", "");

                stripped
                    .chars()
                    .filter(|c| c.is_alphanumeric())
                    .collect::<String>()
            })
            .filter(|word| {
                // Apply uppercase ratio filter
                self.min_uppercase_ratio
                    .map_or(true, |ratio| uppercase_ratio(word) >= ratio)
            })
            .filter(|word| {
                !self.filter_stop_words || !STOP_WORDS.contains(&word.to_lowercase().as_str())
            })
            .map(|word| {
                // Normalize TLDs and lowercase the base word
                word.rsplit_once('.')
                    .filter(|(_, tld)| TLD_LIST.contains(&tld.to_lowercase().as_str()))
                    .map_or_else(|| word.to_lowercase(), |(base, _)| base.to_lowercase())
            })
            .flat_map(|word| {
                // Handle hyphenated words
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
            .map(|word| word.to_uppercase()) // Convert to uppercase
            // TODO: Remove
            // .map(|word| {
            //     println!("{}", word);
            //     word
            // })
            .collect()
    }

    pub fn token_to_charcode_vector(self, token: &str) -> TokenizerVectorTokenType {
        token.chars().map(|c| c as u32).collect()
    }

    pub fn charcode_vector_to_token(self, charcodes: &TokenizerVectorTokenType) -> String {
        charcodes
            .iter()
            .map(|&code| char::from_u32(code).unwrap_or('\u{FFFD}')) // Convert code to char, using '�' as a fallback
            .collect()
    }

    pub fn tokenize_to_charcode_vectors(self, text: &str) -> Vec<TokenizerVectorTokenType> {
        self.tokenize(text)
            .into_iter() // Use the existing `tokenize` function to get tokens
            .map(|token| self.token_to_charcode_vector(&token)) // Convert each token to char code vectors
            .collect()
    }
}
