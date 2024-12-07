#[derive(Clone)]
pub struct ContextAttention {
    pub global_weights: Vec<f32>, // Shared global weights
    pub embedding_size: usize,    // Fixed size of embeddings
}

impl ContextAttention {
    /// Initialize the ContextAttention with a fixed embedding size
    pub fn new(embedding_size: usize) -> Self {
        Self {
            global_weights: vec![0.1; embedding_size], // Initialize with small values
            embedding_size,
        }
    }

    /// Create a new ContextAttention with the specified weights
    /// Ensures the weights' length matches the embedding size
    pub fn from_weights(weights: Vec<f32>) -> Result<Self, &'static str> {
        let embedding_size = weights.len();
        Ok(Self {
            global_weights: weights,
            embedding_size,
        })
    }

    /// Represent a token (ticker or word) as a fixed-size vector
    /// Example: Normalize character values for lightweight embeddings
    // pub fn representation(&self, token: &str) -> Vec<f32> {
    //     token
    //         .chars()
    //         .map(|c| (c as u32 as f32) / 255.0) // Normalize character codes
    //         .take(self.embedding_size) // Limit to embedding size
    //         .chain(std::iter::repeat(0.0)) // Pad with zeros if necessary
    //         .take(self.embedding_size)
    //         .collect()
    // }

    pub fn representation(&self, token: &str) -> Vec<f32> {
        let mut embedding = vec![0.0; self.embedding_size];

        // Influence all embedding dimensions using character and position
        for (i, c) in token.chars().enumerate() {
            let char_value = (c as u32 as f32) / 255.0; // Normalize character value
            for j in 0..self.embedding_size {
                // Distribute influence across dimensions using character and position
                let pos_enc = if j % 2 == 0 {
                    ((i as f32) / 10000.0_f32.powf(j as f32 / self.embedding_size as f32)).sin()
                } else {
                    ((i as f32) / 10000.0_f32.powf(j as f32 / self.embedding_size as f32)).cos()
                };
                embedding[j] += char_value * pos_enc;
            }
        }

        // Normalize the embedding vector
        let norm = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for value in embedding.iter_mut() {
                *value /= norm;
            }
        }

        embedding
    }

    /// Aggregate a context (array of words) into a single representation
    /// Example: Use the average of all word embeddings
    // pub fn aggregate_context(&self, context: &[String]) -> Vec<f32> {
    //     let mut aggregated = vec![0.0; self.embedding_size];
    //     for word in context {
    //         let word_vec = self.representation(word);
    //         for i in 0..self.embedding_size {
    //             aggregated[i] += word_vec[i];
    //         }
    //     }
    //     if !context.is_empty() {
    //         for value in aggregated.iter_mut() {
    //             *value /= context.len() as f32; // Normalize by context length
    //         }
    //     }
    //     aggregated
    // }

    // pub fn aggregate_context(&self, context: &[String]) -> Vec<f32> {
    //     let mut aggregated = vec![0.0; self.embedding_size];
    //     for word in context {
    //         let word_vec = self.representation(word);
    //         for i in 0..self.embedding_size {
    //             aggregated[i] += word_vec[i];
    //         }
    //     }
    //     if !context.is_empty() {
    //         for value in aggregated.iter_mut() {
    //             *value /= context.len() as f32; // Normalize by context length
    //         }
    //     }
    //     // Ensure no exact zeros in the aggregated vector
    //     for value in aggregated.iter_mut() {
    //         *value += 1e-6; // Small constant to ensure gradients propagate
    //     }
    //     aggregated
    // }

    pub fn aggregate_context(&self, context: &[String]) -> Vec<f32> {
        let mut aggregated = vec![0.0; self.embedding_size];

        for (pos, word) in context.iter().enumerate() {
            let word_vec = self.representation(word);
            let pos_weight = (1.0 + pos as f32).ln(); // Positional weighting (e.g., log scale)
            for j in 0..self.embedding_size {
                aggregated[j] += pos_weight * word_vec[j];
            }
        }

        // Normalize the aggregated vector
        let norm = aggregated.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for value in aggregated.iter_mut() {
                *value /= norm; // Normalize vector
            }
        }

        aggregated
    }

    // TODO: Rename ticker to query
    /// Compute the score for a ticker against the aggregated context
    pub fn score(&self, ticker: &str, context: &[String]) -> f32 {
        let ticker_vec = self.representation(ticker);
        let context_vec = self.aggregate_context(context);

        ticker_vec
            .iter()
            .zip(context_vec.iter())
            .zip(self.global_weights.iter())
            .map(|((t, c), gw)| t * c * gw)
            .sum() // Dot product with global weights
    }

    // TODO: Rename ticker to query
    /// Update the global weights based on the reward signal
    /// Uses gradient descent to adjust the weights
    pub fn update_weights(
        &mut self,
        ticker: &str,
        context: &[String],
        target: f32, // 1.0 for true positive, 0.0 for false positive
        learning_rate: f32,
    ) {
        let ticker_vec = self.representation(ticker);
        let context_vec = self.aggregate_context(context);
        let predicted = self.score(ticker, context);

        for i in 0..self.embedding_size {
            let gradient = 2.0 * (predicted - target) * ticker_vec[i] * context_vec[i];
            // println!("Gradient[{}]: {:.6}", i, gradient); // Log gradient
            self.global_weights[i] -= learning_rate * gradient; // Gradient descent
        }
    }

    // With regularization
    // pub fn update_weights(
    //     &mut self,
    //     ticker: &str,
    //     context: &[String],
    //     target: f32, // 1.0 for true positive, 0.0 for false positive
    //     learning_rate: f32,
    // ) {
    //     let ticker_vec = self.representation(ticker);
    //     let context_vec = self.aggregate_context(context);
    //     let predicted = self.score(ticker, context);

    //     let regularization_factor = 0.001; // Small regularization term

    //     for i in 0..self.embedding_size {
    //         let gradient = 2.0 * (predicted - target) * ticker_vec[i] * context_vec[i];
    //         self.global_weights[i] -= learning_rate * gradient;

    //         // Apply regularization to encourage non-zero values
    //         self.global_weights[i] -=
    //             learning_rate * regularization_factor * self.global_weights[i];
    //     }
    // }
}
