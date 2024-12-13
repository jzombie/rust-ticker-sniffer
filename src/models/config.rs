pub struct DocumentCompanyNameExtractorConfig {
    pub min_text_doc_token_sim_threshold: f32,
    pub continuity_reward: f32,
    pub confidence_score_duplicate_threshold: usize,
    pub low_confidence_penalty_factor: f32,
    pub min_confidence_level_threshold: f32,
}
