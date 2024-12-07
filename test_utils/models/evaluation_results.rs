use std::collections::HashSet;
use ticker_sniffer::TickerSymbol;

#[derive(Debug)]
pub struct EvaluationResult {
    pub expected: Vec<TickerSymbol>,
    pub false_positives: Vec<TickerSymbol>,
    pub false_negatives: Vec<TickerSymbol>,
}

impl EvaluationResult {
    pub fn new(expected: &[TickerSymbol], results: &[TickerSymbol]) -> Self {
        let expected_set: HashSet<_> = expected.iter().cloned().collect();
        let results_set: HashSet<_> = results.iter().cloned().collect();

        let false_positives: Vec<TickerSymbol> =
            results_set.difference(&expected_set).cloned().collect();
        let false_negatives: Vec<TickerSymbol> =
            expected_set.difference(&results_set).cloned().collect();

        Self {
            expected: expected.to_vec(),
            false_positives,
            false_negatives,
        }
    }

    // TODO: Reimplement?
    // Compute summary statistics for the evaluation.
    // pub fn summary(&self) -> String {
    //     format!(
    //         "Expected: {}, False Positives: {}, False Negatives: {}",
    //         self.expected.len(),
    //         self.false_positives.len(),
    //         self.false_negatives.len()
    //     )
    // }
}
