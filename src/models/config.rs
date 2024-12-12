pub struct DocumentCompanyNameExtractorConfig {
    pub min_text_doc_token_sim_threshold: f32,
    pub token_window_size: usize,
    pub max_allowable_query_token_gap: usize,
    pub continuity_reward: f32,
    pub confidence_score_duplicate_threshold: usize,
    pub low_confidence_penalty_factor: f32,
    pub min_confidence_level_threshold: f32,
}
