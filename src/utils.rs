// TODO: Clean up

pub mod cosine_similarity;
pub use cosine_similarity::cosine_similarity;

// pub mod generate_alternative_symbols;
// pub mod jaccard_similarity_chars;
pub mod tokenize;

// pub use generate_alternative_symbols::generate_alternative_symbols;
//pub use jaccard_similarity_chars::jaccard_similarity_chars;
pub use tokenize::{
    charcode_vector_to_token, token_to_charcode_vector, tokenize, tokenize_to_charcode_vectors,
};

pub mod pad_vector;
pub use pad_vector::{pad_vector, pad_vectors_to_match};
