extern crate fxhash;
use crate::constants::DEFAULT_BIAS_ADJUSTER_SCORE;
use fxhash::hash64;
use std::collections::HashMap;

#[derive(Clone)]
pub struct ResultBiasAdjuster {
    pub weights: HashMap<u64, f32>, // Weights for query-context pairs
}

impl ResultBiasAdjuster {
    /// Initialize the model
    pub fn new() -> Self {
        Self {
            weights: HashMap::new(),
        }
    }

    /// Initialize from weights in the form of a slice of tuples
    pub fn from_weights(weights: &[(u64, f32)]) -> Self {
        let weights_map: HashMap<u64, f32> = weights.iter().cloned().collect();
        Self {
            weights: weights_map,
        }
    }

    /// Display weights in the format of a slice of tuples
    pub fn to_weight_slice_format(&self) -> Vec<(u64, f32)> {
        self.weights.iter().map(|(&k, &v)| (k, v)).collect()
    }

    /// Compute a hash for a query-context pair
    pub fn hash_query_context(&self, query: &str, context: &[String]) -> u64 {
        let mut hash = hash64(query);

        for word in context {
            hash ^= hash64(word); // Combine hashes with XOR for deterministic result
        }

        hash
    }

    /// Compute the score for a query-context pair
    pub fn score(&self, query: &str, context: &[String]) -> f32 {
        let key = self.hash_query_context(query, context);
        *self
            .weights
            .get(&key)
            .unwrap_or(&DEFAULT_BIAS_ADJUSTER_SCORE)
    }

    /// Update the weights for a query-context pair using a simple gradient
    pub fn update_weights(
        &mut self,
        query: &str,
        context: &[String],
        target: f32, // 1.0 for similar, 0.0 for dissimilar
        learning_rate: f32,
    ) {
        let key = self.hash_query_context(query, context);

        // Clone weight for calculation, avoiding mutable borrow during `score` call
        let weight = *self.weights.get(&key).unwrap_or(&0.01); // Default small random value

        // Compute similarity (using the score function)
        let similarity = self.score(query, context);

        // Compute gradient based on target
        let gradient = if target == 1.0 {
            // Positive pair: Push similarity towards 1.0
            1.0 - similarity
        } else {
            // Negative pair: Push similarity away from 1.0
            similarity - 1.0
        };

        // Regularization term to prevent weights from growing too large
        let regularization = 0.01; // Regularization coefficient

        // Compute the updated weight
        let updated_weight = weight + learning_rate * gradient - regularization * weight;

        println!("Updated weight: {}", updated_weight);

        // Update the weight in the HashMap
        self.weights.insert(key, updated_weight.clamp(0.0, 1.0));
    }
}
