pub struct ContextAttention {
    global_weights: Vec<f32>, // Shared global weights
    embedding_size: usize,    // Fixed size of embeddings
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

    /// Retrieve the global weights embedding
    pub fn get_global_weights(&self) -> Vec<f32> {
        self.global_weights.clone()
    }

    /// Represent a token (ticker or word) as a fixed-size vector
    /// Example: Normalize character values for lightweight embeddings
    pub fn representation(&self, token: &str) -> Vec<f32> {
        token
            .chars()
            .map(|c| (c as u32 as f32) / 255.0) // Normalize character codes
            .take(self.embedding_size) // Limit to embedding size
            .chain(std::iter::repeat(0.0)) // Pad with zeros if necessary
            .take(self.embedding_size)
            .collect()
    }

    /// Aggregate a context (array of words) into a single representation
    /// Example: Use the average of all word embeddings
    pub fn aggregate_context(&self, context: &[String]) -> Vec<f32> {
        let mut aggregated = vec![0.0; self.embedding_size];
        for word in context {
            let word_vec = self.representation(word);
            for i in 0..self.embedding_size {
                aggregated[i] += word_vec[i];
            }
        }
        if !context.is_empty() {
            for value in aggregated.iter_mut() {
                *value /= context.len() as f32; // Normalize by context length
            }
        }
        aggregated
    }

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
            self.global_weights[i] -= learning_rate * gradient; // Gradient descent
        }
    }
}
