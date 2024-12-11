pub struct DocumentCompanyNameExtractorConfig {
    pub min_text_doc_token_sim_threshold: f32,
    // pub token_length_diff_tolerance: usize,
    pub token_window_size: usize,
    pub token_gap_penalty: f32,
    pub continuity_reward: f32,
    pub low_confidence_penalty_factor: f32,
    pub min_confidence_level_threshold: f32,
}
