use crate::test_utils::constants::{DEFAULT_FALSE_NEGATIVE_WEIGHT, DEFAULT_FALSE_POSITIVE_WEIGHT};
use std::collections::HashSet;
use ticker_sniffer::{CompanyNameTokenRanking, TickerSymbol};

pub struct EvaluationResult {
    pub expected: Vec<TickerSymbol>,
    pub false_positives: Vec<TickerSymbol>,
    pub false_negatives: Vec<TickerSymbol>,
    pub expected_rankings: Vec<CompanyNameTokenRanking>,
    pub false_positive_rankings: Vec<CompanyNameTokenRanking>,
    pub false_negative_rankings: Vec<CompanyNameTokenRanking>,
    pub mse: f32,
}

impl EvaluationResult {
    pub fn new(
        expected: &[TickerSymbol],
        results: &[TickerSymbol],
        rankings: &[CompanyNameTokenRanking],
    ) -> Self {
        let expected_set: HashSet<_> = expected.iter().cloned().collect();
        let results_set: HashSet<_> = results.iter().cloned().collect();

        let false_positives: Vec<TickerSymbol> =
            results_set.difference(&expected_set).cloned().collect();
        let false_negatives: Vec<TickerSymbol> =
            expected_set.difference(&results_set).cloned().collect();

        // Separate rankings into groups
        let mut expected_rankings = Vec::new();
        let mut false_positive_rankings = Vec::new();
        let mut false_negative_rankings = Vec::new();

        for ranking in rankings {
            if expected_set.contains(&ranking.ticker_symbol) {
                expected_rankings.push(ranking.clone());
            } else if false_positives.contains(&ranking.ticker_symbol) {
                false_positive_rankings.push(ranking.clone());
            } else if false_negatives.contains(&ranking.ticker_symbol) {
                false_negative_rankings.push(ranking.clone());
            }
        }

        let mut evaluation_result = Self {
            expected: expected.to_vec(),
            false_positives,
            false_negatives,
            expected_rankings,
            false_positive_rankings,
            false_negative_rankings,
            mse: 0.0, // Placeholder, will be updated
        };

        let mse = evaluation_result
            .compute_mse(DEFAULT_FALSE_NEGATIVE_WEIGHT, DEFAULT_FALSE_POSITIVE_WEIGHT);

        // Update the struct
        evaluation_result.mse = mse;

        evaluation_result
    }

    /// Compute MSE and update the `mse` field.
    pub fn compute_mse(&mut self, false_negative_weight: f32, false_positive_weight: f32) -> f32 {
        // Compute the weighted squared differences
        let weighted_squared_differences: f32 = self.false_negatives.len() as f32
            * false_negative_weight
            + self.false_positives.len() as f32 * false_positive_weight;

        // Calculate the mean squared error
        let total_tickers =
            self.expected.len() + self.false_positives.len() + self.false_negatives.len();

        let mse = if total_tickers > 0 {
            weighted_squared_differences / total_tickers as f32
        } else {
            0.0 // Handle edge case where there are no tickers
        };

        mse
    }
}
