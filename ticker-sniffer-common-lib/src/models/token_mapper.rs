use crate::types::{TokenId, TokenRef, TokenVector};
use crate::Tokenizer;
use std::collections::HashMap;

/// Maps "global" tokens without including function specific to token types.
pub struct TokenMapper {
    pub token_map: HashMap<TokenVector, TokenId>,
    pub reverse_token_map: HashMap<TokenId, TokenVector>,
    next_id: TokenId,
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
    pub fn upsert_token(&mut self, token: &str) -> TokenId {
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
    pub fn get_token_id(&self, token: &TokenRef) -> Option<TokenId> {
        let token_vector = Tokenizer::token_to_charcode_vector(token);

        self.token_map.get(&token_vector).copied()
    }

    pub fn get_filtered_tokens<'a>(&'a self, tokens: Vec<&'a TokenRef>) -> Vec<&TokenRef> {
        tokens
            .into_iter()
            .filter(|token| self.get_token_id(token).is_some())
            .collect()
    }

    pub fn get_filtered_token_ids<'a>(&'a self, tokens: Vec<&'a TokenRef>) -> Vec<TokenId> {
        tokens
            .into_iter()
            .filter_map(|token| self.get_token_id(token))
            .collect()
    }

    pub fn get_token_by_id(&self, token_id: TokenId) -> Option<String> {
        self.reverse_token_map
            .get(&token_id)
            .map(|token_vector| Tokenizer::charcode_vector_to_token(token_vector))
    }

    pub fn get_tokens_by_ids(&self, token_ids: &[TokenId]) -> Vec<Option<String>> {
        token_ids
            .iter()
            .map(|&token_id| self.get_token_by_id(token_id))
            .collect()
    }

    /// Gets the total number of unique tokens
    pub fn get_token_count(&self) -> usize {
        self.token_map.len()
    }
}
