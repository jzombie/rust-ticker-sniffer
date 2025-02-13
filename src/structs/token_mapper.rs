use crate::types::{TokenId, TokenRef, TokenVector};
use crate::Tokenizer;
use std::collections::HashMap;

/// A struct to map tokens to unique identifiers and vice versa.
///
/// This structure is responsible for maintaining a bidirectional mapping
/// between tokens (represented as character vectors) and their unique IDs.
/// It also provides utility methods to query and manage these mappings.
pub struct TokenMapper {
    /// A map of token character vectors to their unique IDs.
    pub token_map: HashMap<TokenVector, TokenId>,

    /// A reverse map of unique IDs back to their token character vectors.
    pub reverse_token_map: HashMap<TokenId, TokenVector>,

    /// Tracks the next available unique ID for new tokens.
    next_id: TokenId,
}

impl TokenMapper {
    /// Creates a new instance of `TokenMapper`.
    ///
    /// Initializes empty maps for tokens and reverse lookups, and sets the
    /// starting `next_id` to 0.
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        TokenMapper {
            token_map: HashMap::new(),
            reverse_token_map: HashMap::new(),
            next_id: 0,
        }
    }

    /// Adds a token to the map if it doesn't already exist, and returns its unique ID.
    ///
    /// If the token is already present in the `token_map`, its existing ID is returned.
    /// Otherwise, a new ID is generated, stored, and returned.
    ///
    /// # Arguments
    /// * `token` - A reference to the token string to add or look up.
    ///
    /// # Returns
    /// * A unique ID for the token.
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

    /// Gets the unique ID for a token if it exists in the map.
    ///
    /// # Arguments
    /// * `token` - A reference to the token string to look up.
    ///
    /// # Returns
    /// * `Some(TokenId)` if the token is present, or `None` if it is not found.
    pub fn get_token_id(&self, token: &TokenRef) -> Option<TokenId> {
        let token_vector = Tokenizer::token_to_charcode_vector(token);

        self.token_map.get(&token_vector).copied()
    }

    /// Filters and returns tokens that are present in the map.
    ///
    /// # Arguments
    /// * `tokens` - A vector of borrowed token references.
    ///
    /// # Returns
    /// * A vector of borrowed token references that exist in the map.
    pub fn get_filtered_tokens<'a>(&'a self, tokens: Vec<&'a TokenRef>) -> Vec<&'a TokenRef> {
        tokens
            .into_iter()
            .filter(|token| self.get_token_id(token).is_some())
            .collect()
    }

    /// Filters and returns token IDs for tokens that exist in the map.
    ///
    /// # Arguments
    /// * `tokens` - A vector of borrowed token references.
    ///
    /// # Returns
    /// * A vector of token IDs corresponding to the tokens found in the map.
    pub fn get_filtered_token_ids<'a>(&'a self, tokens: Vec<&'a TokenRef>) -> Vec<TokenId> {
        tokens
            .into_iter()
            .filter_map(|token| self.get_token_id(token))
            .collect()
    }

    /// Retrieves the token string for a given unique ID.
    ///
    /// # Arguments
    /// * `token_id` - The unique ID of the token to look up.
    ///
    /// # Returns
    /// * `Some(String)` containing the token if the ID is found, or `None` otherwise.
    pub fn get_token_by_id(&self, token_id: TokenId) -> Option<String> {
        self.reverse_token_map
            .get(&token_id)
            .map(Tokenizer::charcode_vector_to_token)
    }

    /// Retrieves token strings for a list of token IDs.
    ///
    /// # Arguments
    /// * `token_ids` - A slice of token IDs to look up.
    ///
    /// # Returns
    /// * A vector of `Option<String>` where each entry corresponds to the token
    ///   string for the given ID, or `None` if the ID is not found.
    pub fn get_tokens_by_ids(&self, token_ids: &[TokenId]) -> Vec<Option<String>> {
        token_ids
            .iter()
            .map(|&token_id| self.get_token_by_id(token_id))
            .collect()
    }

    /// Gets the total number of unique tokens in the map.
    ///
    /// # Returns
    /// * The number of unique tokens as a `usize`.
    pub fn get_token_count(&self) -> usize {
        self.token_map.len()
    }
}
