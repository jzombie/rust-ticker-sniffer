#[derive(Debug, Clone, Copy)]
pub struct Weights {
    pub continuity: f32,
    pub mismatched_letter_penalty: f32,
    pub mismatched_word_penalty: f32,
    pub match_score_threshold: f32,
    pub symbol_abbr_threshold: f32,
}
