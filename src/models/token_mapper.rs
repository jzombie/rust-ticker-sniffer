use crate::types::TokenizerVectorToken;
use crate::Tokenizer;
use std::collections::HashMap;

pub struct TokenMapper {
    token_map: HashMap<TokenizerVectorToken, usize>,
    reverse_token_map: HashMap<usize, TokenizerVectorToken>,
    next_id: usize,
}

impl TokenMapper {
    /// Creates a new TokenMapper
    pub fn new() -> Self {
        TokenMapper {
            token_map: HashMap::new(),
            reverse_token_map: HashMap::new(),
            next_id: 0,
        }
    }

    /// Adds a token (as a string) to the map if it doesn't exist,
    /// and returns its unique ID
    pub fn upsert_token(&mut self, token: &str) -> usize {
        let token_vector = Tokenizer::token_to_charcode_vector(token);

        if let Some(&id) = self.token_map.get(&token_vector) {
            id
        } else {
            let id = self.next_id;
            self.token_map.insert(token_vector.clone(), id);
            self.reverse_token_map.insert(id, token_vector.clone());
            self.next_id += 1;
            id
        }
    }

    /// Gets the ID for a token (as a string), or None if the token is not present
    pub fn get_token_id(&self, token: &str) -> Option<usize> {
        let token_vector = Tokenizer::token_to_charcode_vector(token);

        self.token_map.get(&token_vector).copied()
    }

    pub fn get_filtered_tokens<'a>(&'a self, tokens: Vec<&'a str>) -> Vec<&str> {
        tokens
            .into_iter()
            .filter(|token| self.get_token_id(token).is_some())
            .collect()
    }

    pub fn get_filtered_token_ids<'a>(&'a self, tokens: Vec<&'a str>) -> Vec<usize> {
        tokens
            .into_iter()
            .filter_map(|token| self.get_token_id(token))
            .collect()
    }

    pub fn get_token_by_id(&self, id: usize) -> Option<String> {
        self.reverse_token_map
            .get(&id)
            .map(|token_vector| Tokenizer::charcode_vector_to_token(token_vector))
    }

    pub fn get_tokens_by_ids(&self, ids: &[usize]) -> Vec<Option<String>> {
        ids.iter()
            .map(|&id| self.get_token_by_id(id)) // Call the wrapped `get_token_by_id`
            .collect()
    }

    /// Gets the total number of unique tokens
    pub fn token_count(&self) -> usize {
        self.token_map.len()
    }
}
