use crate::types::TokenizerVectorToken;
use crate::Tokenizer;
use std::collections::HashMap;

pub struct TokenMapper {
    pub token_map: HashMap<TokenizerVectorToken, usize>,
    next_id: usize,
}

impl TokenMapper {
    /// Creates a new TokenMapper
    pub fn new() -> Self {
        TokenMapper {
            token_map: HashMap::new(),
            next_id: 0,
        }
    }

    /// Converts a string slice into a vector of char codes (u32)
    // fn to_char_codes(token: &str) -> TokenizerVectorToken {
    //     token.chars().map(|c| c as u32).collect()
    // }

    /// Adds a token (as a string) to the map if it doesn't exist,
    /// and returns its unique ID
    pub fn upsert_token(&mut self, token: &str) -> usize {
        // TODO: Use Tokenizer for this
        // let char_codes = Self::to_char_codes(token);
        let token_vector = Tokenizer::token_to_charcode_vector(token);

        if let Some(&id) = self.token_map.get(&token_vector) {
            id
        } else {
            let id = self.next_id;
            self.token_map.insert(token_vector, id);
            self.next_id += 1;
            id
        }
    }

    /// Gets the ID for a token (as a string), or None if the token is not present
    pub fn get_token_id(&self, token: &str) -> Option<usize> {
        let token_vector = Tokenizer::token_to_charcode_vector(token);

        self.token_map.get(&token_vector).copied()
    }

    /// Gets the total number of unique tokens
    pub fn token_count(&self) -> usize {
        self.token_map.len()
    }
}
