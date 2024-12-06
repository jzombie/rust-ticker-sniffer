#[derive(Debug, Clone, Copy)]
pub struct Weights {
    pub continuity: f32,
    pub mismatched_letter_penalty: f32,
    pub mismatched_word_penalty: f32,
    pub match_score_threshold: f32,
    pub symbol_abbr_threshold: f32,
}

impl Weights {
    pub fn new(
        continuity: f32,
        mismatched_letter_penalty: f32,
        mismatched_word_penalty: f32,
        match_score_threshold: f32,
        symbol_abbr_threshold: f32,
    ) -> Self {
        Self {
            continuity,
            mismatched_letter_penalty,
            mismatched_word_penalty,
            match_score_threshold,
            symbol_abbr_threshold,
        }
    }
}
