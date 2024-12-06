use std::fmt;

#[derive(Debug, Clone, Copy)]
pub struct Weights {
    pub consecutive_match_weight: f32,
    pub letter_mismatch_penalty: f32,
    pub word_mismatch_penalty: f32,
    pub stop_word_filter_ratio: f32,
    pub minimum_match_score: f32,
    pub abbreviation_match_threshold: f32,
}

impl fmt::Display for Weights {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
          f,
          "Weights (\n\tconsecutive_match_weight: {},\n\tletter_mismatch_penalty: {},\n\tword_mismatch_penalty: {},\n\tstop_word_filter_ratio: {},\n\tminimum_match_score: {},\n\tabbreviation_match_threshold: {}\n)",
          self.consecutive_match_weight,
          self.letter_mismatch_penalty,
          self.word_mismatch_penalty,
          self.stop_word_filter_ratio,
          self.minimum_match_score,
          self.abbreviation_match_threshold
      )
    }
}
