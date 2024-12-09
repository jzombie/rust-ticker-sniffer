use crate::constants::TLD_LIST;

#[derive(Copy, Clone)]
pub struct Tokenizer {
    pub min_uppercase_ratio: Option<f32>,
    pub hyphens_as_potential_multiple_words: bool,
}

impl Tokenizer {
    pub fn ticker_symbol_parser() -> Self {
        Self {
            min_uppercase_ratio: Some(0.9),
            hyphens_as_potential_multiple_words: false,
        }
    }

    pub fn text_doc_parser() -> Self {
        Self {
            min_uppercase_ratio: None,
            hyphens_as_potential_multiple_words: true,
        }
    }

    // TODO: Provide optional STOP_WORD filtering
    /// Tokenizer function to split the text into individual tokens.
    ///
    /// Note: This explcitly does not modify the case of the text.
    pub fn tokenize(self, text: &str) -> Vec<String> {
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

        // Helper function to calculate uppercase ratio
        fn uppercase_ratio(word: &str) -> f32 {
            let total_chars = word.chars().count() as f32;
            if total_chars == 0.0 {
                return 0.0;
            }
            let uppercase_chars = word.chars().filter(|c| c.is_uppercase()).count() as f32;
            uppercase_chars / total_chars
        }

        // Tokenize, process possessives, handle web addresses, and uppercase
        cleaned_text
            .split_whitespace()
            // .filter(|word| word.chars().any(|c| c.is_uppercase())) // Keep only words with at least one capital letter
            .filter(|word| {
                if let Some(ratio) = self.min_uppercase_ratio {
                    uppercase_ratio(word) >= ratio
                } else {
                    word.chars().any(|c| c.is_uppercase())
                }
            })
            .flat_map(|word| {
                let mut tokens = Vec::new();

                if self.hyphens_as_potential_multiple_words {
                    // Handle hyphenated words
                    if word.contains('-') {
                        let split_words: Vec<&str> = word.split('-').collect();
                        tokens.push(word.replace('-', "").to_uppercase()); // Concatenated version
                        for part in &split_words {
                            tokens.push(
                                part.chars()
                                    .filter(|c| c.is_alphanumeric())
                                    .collect::<String>()
                                    .to_uppercase(),
                            ); // Add each part separately
                        }
                        return tokens;
                    }
                }

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

    pub fn token_to_charcode_vector(self, token: &str) -> Vec<u32> {
        token.chars().map(|c| c as u32).collect()
    }

    pub fn charcode_vector_to_token(self, charcodes: &[u32]) -> String {
        charcodes
            .iter()
            .map(|&code| char::from_u32(code).unwrap_or('\u{FFFD}')) // Convert code to char, using '�' as a fallback
            .collect()
    }

    pub fn tokenize_to_charcode_vectors(self, text: &str) -> Vec<Vec<u32>> {
        self.tokenize(text)
            .into_iter() // Use the existing `tokenize` function to get tokens
            .map(|token| self.token_to_charcode_vector(&token)) // Convert each token to char code vectors
            .collect()
    }
}
