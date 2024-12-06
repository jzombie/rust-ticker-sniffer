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
        let struct_name = stringify!(Weights);
        let fields = vec![
            ("consecutive_match_weight", self.consecutive_match_weight),
            ("letter_mismatch_penalty", self.letter_mismatch_penalty),
            ("word_mismatch_penalty", self.word_mismatch_penalty),
            ("stop_word_filter_ratio", self.stop_word_filter_ratio),
            ("minimum_match_score", self.minimum_match_score),
            (
                "abbreviation_match_threshold",
                self.abbreviation_match_threshold,
            ),
        ];

        writeln!(f, "{} (", struct_name)?;
        for (name, value) in fields {
            writeln!(f, "\t{}: {},", name, value)?;
        }
        write!(f, ")") // Final closing parenthesis
    }
}
