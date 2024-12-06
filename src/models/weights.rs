use std::fmt;

#[derive(Debug, Clone, Copy)]
pub struct Weights {
    pub continuity: f32,
    pub mismatched_letter_penalty: f32,
    pub mismatched_word_penalty: f32,
    pub stop_word_match_ratio: f32,
    pub match_score_threshold: f32,
    pub symbol_abbr_threshold: f32,
}

impl fmt::Display for Weights {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
          f,
          "Weights (\n\tcontinuity: {},\n\tmismatched_letter_penalty: {},\n\tmismatched_word_penalty: {},\n\tstop_word_match_ratio: {},\n\tmatch_score_threshold: {},\n\tsymbol_abbr_threshold: {}\n)",
          self.continuity,
          self.mismatched_letter_penalty,
          self.mismatched_word_penalty,
          self.stop_word_match_ratio,
          self.match_score_threshold,
          self.symbol_abbr_threshold
      )
    }
}
