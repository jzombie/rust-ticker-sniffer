pub struct TickerExtractorWeights {
    min_similarity_threshold: f32,
    token_length_diff_tolerance: usize,
}

pub struct TickerExtractor {
    weights: TickerExtractorWeights,
    query: String,
    tokenized_query_vectors: Vec<Vec<u32>>,
    results: Vec<TickerSymbol>,
}
