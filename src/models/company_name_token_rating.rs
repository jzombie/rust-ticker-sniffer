// TODO: Remove Debug trait
#[derive(Debug, Clone)]
pub struct CompanyNameTokenRanking {
    pub ticker_symbol: String,
    pub company_name: String,
    pub input_token_indices: Vec<usize>,
    pub consecutive_match_count: usize,
    pub consecutive_jaccard_similarity: f32,
    pub match_score: f32,
    pub context_attention_score: f32,
    pub context_query_string: String,
    pub context_company_tokens: Vec<String>,
}
