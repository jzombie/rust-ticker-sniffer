use crate::types::{
    CompanySymbolsList, CompanyTokenSourceType, TickerSymbol, TokenizerVectorTokenType,
};
use crate::utils::index_difference_similarity;
use crate::DocumentCompanyNameExtractorConfig;
use crate::{CompanyTokenProcessor, Tokenizer};
use core::f32;
use std::collections::{HashMap, HashSet};

type TokenWindowIndex = usize;
type QueryTokenIndex = usize;

// TODO: If possible to add an optional report of why things were filtered or
// included in the final result, within reason (i.e. within
// `min_text_doc_token_sim_threshold`), would be beneficial for debugging.

#[derive(Debug, Clone)]
struct QueryVectorIntermediateSimilarityState {
    token_window_index: TokenWindowIndex,
    query_token_index: QueryTokenIndex,
    query_vector: TokenizerVectorTokenType,
    company_index: usize, // TODO: Add more specific type
    company_token_type: CompanyTokenSourceType,
    company_token_index_by_source_type: usize, // TODO: Add more specific type
    company_token_vector: TokenizerVectorTokenType,
    company_name_similarity_at_index: f32,
}

pub struct DocumentCompanyNameExtractor<'a> {
    company_symbols_list: &'a CompanySymbolsList,
    text_doc_tokenizer: &'a Tokenizer,
    company_token_processor: &'a CompanyTokenProcessor<'a>,
    user_config: DocumentCompanyNameExtractorConfig,
    is_extracting: bool,

    tokenized_query_vectors: Vec<TokenizerVectorTokenType>,
    company_similarity_states: Vec<QueryVectorIntermediateSimilarityState>,
    progressible_company_indices: HashSet<usize>, // TODO: Add type
    results: Vec<TickerSymbol>,
}

impl<'a> DocumentCompanyNameExtractor<'a> {
    /// Creates a new instance of the `DocumentCompanyNameExtractor`
    /// with the provided company symbols list and configuration.
    /// Initializes the necessary tokenizers and token processors.
    pub fn new(
        company_symbols_list: &'a CompanySymbolsList,
        user_config: DocumentCompanyNameExtractorConfig,
        text_doc_tokenizer: &'a Tokenizer,
        company_token_processor: &'a CompanyTokenProcessor,
    ) -> Self {
        Self {
            company_symbols_list,
            text_doc_tokenizer: &text_doc_tokenizer,
            company_token_processor: &company_token_processor,
            user_config,
            is_extracting: false,
            tokenized_query_vectors: vec![],
            company_similarity_states: vec![],
            progressible_company_indices: HashSet::new(),
            results: vec![],
        }
    }

    /// Extracts ticker symbols from the given text document by tokenizing
    /// and comparing against known company names. Ensures only one extraction
    /// process runs at a time.
    pub fn extract(&mut self, text: &str) -> (HashMap<TickerSymbol, f32>, Vec<usize>) {
        if self.is_extracting {
            panic!("Cannot perform multiple extractions concurrently from same `DocumentCompanyNameExtractor` instance");
        } else {
            self.is_extracting = true;
        }

        self.company_similarity_states.clear();
        self.progressible_company_indices.clear();
        self.results.clear();

        self.tokenized_query_vectors = self.text_doc_tokenizer.tokenize_to_charcode_vectors(&text);

        println!(
            "Tokenized query: {:?}",
            self.text_doc_tokenizer
                .charcode_vectors_to_tokens(&self.tokenized_query_vectors)
        );

        // Begin parsing at the first page
        self.parse_company_names(None);

        let symbols_with_confidence = self.get_symbols_with_confidence();

        let consumed_query_token_indices: Vec<QueryTokenIndex> = self
            .company_similarity_states
            .iter()
            .filter_map(|state| {
                let symbol = self
                    .company_symbols_list
                    .get(state.company_index)
                    .map(|(s, _)| s);

                if let Some(symbol) = symbol {
                    if symbols_with_confidence.contains_key(symbol) {
                        return Some(state.query_token_index);
                    }
                }
                None
            })
            .collect::<HashSet<_>>() // Ensure unique indices
            .into_iter()
            .collect();

        (symbols_with_confidence, consumed_query_token_indices)
    }

    /// Parses company names from the text document, processing a specific
    /// range of tokens defined by the token window index. Filters and evaluates
    /// token similarity for potential ticker symbol matches.
    fn parse_company_names(&mut self, token_window_index: Option<TokenWindowIndex>) {
        let token_window_index = match token_window_index {
            Some(token_window_index) => token_window_index,
            None => 0,
        };

        let (token_start_index, token_end_index) =
            self.calc_token_window_indexes(token_window_index);

        println!(
            "Start index: {}, End index: {}",
            token_start_index, token_end_index
        );

        let mut window_match_count: usize = 0;

        for (query_token_index, query_vector) in self.tokenized_query_vectors.iter().enumerate() {
            let query_vector_length = query_vector.len();

            // TODO: Use to help make queries like "Google" -> "GOOGL" work
            // let min_token_length =
            //     (query_vector_length - self.weights.token_length_diff_tolerance).clamp(1, query_vector_length);
            // let max_token_length = query_vector_length + self.weights.token_length_diff_tolerance;

            // let include_source_types = &[CompanyTokenSourceType::CompanyName];

            let bins = self
                .company_token_processor
                .token_length_bins
                .get(query_vector_length)
                .expect("Could not locate bins");

            for (company_index, tokenized_entry_index) in bins {
                if token_window_index > 0
                    && !self.progressible_company_indices.contains(company_index)
                {
                    continue;
                }

                let (company_token_vector, company_token_type, company_token_index_by_source_type) =
                    &self.company_token_processor.tokenized_entries[*company_index]
                        [*tokenized_entry_index];

                if *company_token_type != CompanyTokenSourceType::CompanyName {
                    continue;
                }

                if *company_token_index_by_source_type >= token_start_index
                    && *company_token_index_by_source_type < token_end_index
                {
                    let similarity =
                        index_difference_similarity(&query_vector, company_token_vector);

                    // TODO: Remove
                    // let ticker_symbol = &self.company_symbols_list.get(*company_index).expect("").0;
                    // if ticker_symbol == "NVDA" {
                    //     println!("hello??");

                    //     println!("{}", similarity);
                    // }

                    if similarity >= self.user_config.min_text_doc_token_sim_threshold {
                        window_match_count += 1;

                        // let company_name_length = company_name.len();

                        let total_company_name_tokens_length = self
                            .company_token_processor
                            .get_company_name_tokens_length(*company_index);

                        let company_name_similarity_at_index = similarity
                            * (query_vector.len() as f32 / total_company_name_tokens_length as f32);

                        self.company_similarity_states.push(
                            QueryVectorIntermediateSimilarityState {
                                token_window_index,
                                query_token_index,
                                query_vector: query_vector.clone(),
                                company_index: *company_index,
                                company_token_type: *company_token_type,
                                company_token_index_by_source_type:
                                    *company_token_index_by_source_type,
                                company_token_vector: company_token_vector.clone(),
                                company_name_similarity_at_index,
                            },
                        );

                        self.progressible_company_indices.insert(*company_index);
                    } else {
                        self.progressible_company_indices.remove(company_index);
                    }
                }
            }
        }

        // Continue looping if new matches have been discovered
        if window_match_count > 0 {
            self.parse_company_names(Some(token_window_index + 1));
        }
    }

    /// Calculates the start and end token indices for a given window
    /// based on the configured token window size.
    fn calc_token_window_indexes(
        &self,
        token_window_index: TokenWindowIndex,
    ) -> (TokenWindowIndex, TokenWindowIndex) {
        let token_start_index = token_window_index * self.user_config.token_window_size;
        let token_end_index = token_start_index + self.user_config.token_window_size;

        (token_start_index, token_end_index)
    }

    /// Retrieves a map of ticker symbols and their highest confidence scores.
    /// Ensures that only the highest confidence score is retained for each symbol.
    fn get_symbols_with_confidence(&self) -> HashMap<TickerSymbol, f32> {
        let query_token_rankings = self.map_highest_ranking_symbols_to_query_tokens();

        // Prepare a map for symbols and their highest confidence scores
        let mut symbols_with_confidence: HashMap<TickerSymbol, f32> = HashMap::new();

        for (_query_token_index, (symbols, confidence_level)) in query_token_rankings {
            if confidence_level < self.user_config.min_confidence_level_threshold {
                continue;
            }

            for symbol in symbols {
                // TODO: Remove
                // println!("symbol: {}, score: {}", symbol, confidence_level);

                symbols_with_confidence
                    .entry(symbol.clone())
                    .and_modify(|existing_score| {
                        if *existing_score < confidence_level {
                            *existing_score = confidence_level; // Update with higher score
                        }
                    })
                    .or_insert(confidence_level);
            }
        }

        symbols_with_confidence
    }

    /// Maps the highest-ranking ticker symbols to their corresponding query
    /// tokens based on confidence scores. Ensures no overlapping token indices
    /// unless they share the same confidence score.
    fn map_highest_ranking_symbols_to_query_tokens(
        &self,
    ) -> HashMap<QueryTokenIndex, (Vec<TickerSymbol>, f32)> {
        // Calculate confidence scores for each symbol
        let confidence_scores = self.calc_confidence_scores();

        // TODO: Rename `valid_symbols`... it's confusing
        // Collect all valid symbols and their token indices
        let mut valid_symbols: Vec<(TickerSymbol, HashSet<QueryTokenIndex>, f32)> = Vec::new();
        for (symbol, states) in self.collect_coverage_filtered_results() {
            let confidence_score = *confidence_scores
                .get(&symbol)
                .expect("Confidence score not found for symbol");

            // TODO: Remove
            // if symbol == "NVDA" {
            // println!(" ... VALID.. {}, {}", symbol, confidence_score);
            // }

            let token_indices: HashSet<QueryTokenIndex> =
                states.iter().map(|state| state.query_token_index).collect();

            valid_symbols.push((symbol, token_indices, confidence_score));
        }

        // Sort by confidence score (descending) and then by match length (descending)
        valid_symbols.sort_by(|a, b| {
            b.2.partial_cmp(&a.2)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| b.1.len().cmp(&a.1.len()))
        });

        // Filter results to ensure no overlapping token indices unless they share the same score
        let mut used_indices: HashMap<QueryTokenIndex, (Vec<TickerSymbol>, f32)> = HashMap::new();

        for (symbol, token_indices, confidence_score) in valid_symbols {
            // TODO: Remove
            // if symbol == "NVDA" || symbol == "NUMG" {
            // println!(
            //     " ... VALID2.. {}, {}, {:?}",
            //     symbol, confidence_score, token_indices
            // );
            // }

            let mut is_valid = true;

            for &index in &token_indices {
                if let Some((_existing_symbols, existing_score)) = used_indices.get(&index) {
                    // Ensure the confidence score matches and token indices are identical
                    if *existing_score != confidence_score {
                        // TODO: Remove
                        // println!("...... discarded existing: {:?}", existing_symbols);

                        is_valid = false;
                        break;
                    }
                }
            }

            if is_valid {
                // Retain this symbol
                for &index in &token_indices {
                    used_indices
                        .entry(index)
                        .and_modify(|(existing_symbols, _)| {
                            existing_symbols.push(symbol.clone());
                        })
                        .or_insert((vec![symbol.clone()], confidence_score));
                }
            }
        }

        // TODO: Remove
        // println!("Used indices: {:?}", used_indices);

        // Convert used_indices to query token rankings
        let query_token_rankings: HashMap<QueryTokenIndex, (Vec<TickerSymbol>, f32)> = used_indices;

        // TODO: Remove
        println!("QUERY TOKEN RANKINGS");
        for ranking in &query_token_rankings {
            println!("\t{:?}", ranking);
        }

        query_token_rankings
    }

    /// Calculates confidence scores for each ticker symbol by weighing
    /// their similarity states. Applies penalties for large token gaps
    /// and normalizes scores based on the overall distribution.
    fn calc_confidence_scores(&self) -> HashMap<TickerSymbol, f32> {
        let coverage_grouped_results = self.collect_coverage_filtered_results();

        let mut per_symbol_confidence_scores: HashMap<TickerSymbol, f32> = HashMap::new();

        // First pass: Calculate initial confidence scores
        for (symbol, states) in &coverage_grouped_results {
            let mut symbol_confidence_score: f32 = 0.0;

            let mut seen_query_token_indexes: Vec<QueryTokenIndex> = Vec::new();

            for (i, state) in states.iter().enumerate() {
                // Skip repeat processinging of same query token indexes
                if seen_query_token_indexes.contains(&state.query_token_index) {
                    continue;
                } else {
                    seen_query_token_indexes.push(state.query_token_index);
                }

                let mut inverse_weight = 1.0;

                if i > 0 {
                    // Calculate the gap between the current and previous indices
                    let prev_query_token_index = states[i - 1].query_token_index;
                    let gap = (state.query_token_index as isize - prev_query_token_index as isize)
                        .abs() as usize;

                    // Apply a penalty for larger gaps (e.g., inverse weighting)
                    inverse_weight = if gap > 1 {
                        1.0 / (self.user_config.token_gap_penalty + gap as f32)
                    } else {
                        1.0
                    };
                }

                // TODO: Remove
                // if symbol == "NVDA" || symbol == "NUMG" || symbol == "BGM" {
                //     println!(
                //         "calc_confidence_scores similarity at index --------- symbol: {} {} {} : inverse weight: {} : {:?}",
                //         symbol, i, state.company_name_similarity_at_index, inverse_weight, state
                //     );
                // }

                let continuity_reward = ((state.token_window_index + 1) as f32
                    / (self
                        .company_token_processor
                        .get_total_company_name_tokens(state.company_index))
                        as f32)
                    * self.user_config.continuity_reward;

                // Weigh the similarity score based on the calculated weight
                symbol_confidence_score +=
                    (state.company_name_similarity_at_index * inverse_weight) + continuity_reward;
            }

            // TODO: Remove
            // if symbol == "NVDA" || symbol == "NUMG" {
            // if combined_similarity > 0.99 {
            //     println!(
            //         "calc_confidence_scores --------- symbol: {} {}",
            //         symbol, combined_similarity
            //     );
            // }

            per_symbol_confidence_scores.insert(symbol.clone(), symbol_confidence_score);
        }

        // ------------------

        let all_confidence_scores: Vec<f32> =
            per_symbol_confidence_scores.values().cloned().collect();

        // Analyze the distribution of scores
        let mean_score: f32 =
            all_confidence_scores.iter().copied().sum::<f32>() / all_confidence_scores.len() as f32;
        let std_dev: f32 = (all_confidence_scores
            .iter()
            .map(|&score| (score - mean_score).powi(2))
            .sum::<f32>()
            / all_confidence_scores.len() as f32)
            .sqrt();
        let threshold = mean_score - std_dev; // Scores below this threshold are penalized further

        // Calculate the sum of all scores below the threshold
        let total_low_scores: f32 = per_symbol_confidence_scores
            .values()
            .filter(|&&score| score < threshold)
            .sum::<f32>();

        // Second pass: Penalize scores below the threshold based on their proportion
        for (_symbol, score) in &mut per_symbol_confidence_scores {
            if *score < threshold && total_low_scores > 0.0 {
                // Calculate the proportion of this score relative to the total low scores
                let proportion = *score / total_low_scores;

                // Penalize the score based on its proportion
                *score *= proportion * self.user_config.low_confidence_penalty_factor;
            }
        }

        per_symbol_confidence_scores
    }

    /// Groups intermediate similarity states by ticker symbol, ensuring
    /// only states that contribute to coverage increases are retained.
    fn collect_coverage_filtered_results(
        &self,
    ) -> HashMap<TickerSymbol, Vec<QueryVectorIntermediateSimilarityState>> {
        let grouped_states = self.group_by_symbol();
        let coverage_increase_states = self.analyze_coverage_increases();

        let mut coverage_grouped: HashMap<
            TickerSymbol,
            Vec<QueryVectorIntermediateSimilarityState>,
        > = HashMap::new();

        for (symbol, states) in grouped_states {
            // TODO: Remove
            // if symbol == "NVDA" {
            //     println!("----- filtered; symbol: {}", symbol);
            // }

            let empty_vec = Vec::new();
            let coverage_increase = coverage_increase_states.get(&symbol).unwrap_or(&empty_vec);

            let has_coverage_increase = coverage_increase.len() > 0;
            let min_coverage_increase_query_token_index = if has_coverage_increase {
                coverage_increase[0]
            } else {
                usize::MAX
            };

            for state in states {
                if has_coverage_increase
                    && state.query_token_index < min_coverage_increase_query_token_index
                {
                    continue;
                }

                coverage_grouped
                    .entry(symbol.clone())
                    .or_insert_with(Vec::new)
                    .push(state.clone());

                // Skip progressing if no coverage increase
                if !has_coverage_increase {
                    break;
                }
            }
        }

        coverage_grouped
    }

    /// Determines the query token indexes which contribute to company name coverage increases.
    fn analyze_coverage_increases(&self) -> HashMap<TickerSymbol, Vec<QueryTokenIndex>> {
        let mut results = HashMap::new();

        for (symbol, states) in self.group_by_symbol() {
            let mut last_coverage: usize = 0;
            let mut increasing_range = Vec::new();

            for (i, state) in states.iter().enumerate() {
                let current_coverage = state.token_window_index + 1;

                if i > 0 && current_coverage > last_coverage {
                    // Add the previous index to the range if starting a new range
                    if increasing_range.is_empty() && i > 0 {
                        increasing_range.push(states[i - 1].query_token_index);
                    }

                    // Add the current index to the increasing range
                    increasing_range.push(state.query_token_index);
                } else if current_coverage < last_coverage {
                    // Coverage decreased; store the current range and reset
                    if !increasing_range.is_empty() {
                        results.insert(symbol.clone(), increasing_range.clone());
                        increasing_range.clear();
                    }
                }

                // Update the last coverage value
                last_coverage = current_coverage;
            }

            // Store any remaining increasing range
            if !increasing_range.is_empty() {
                results.insert(symbol, increasing_range);
            }
        }

        results
    }

    /// Groups company similarity states by symbol.
    fn group_by_symbol(
        &self,
    ) -> HashMap<TickerSymbol, Vec<QueryVectorIntermediateSimilarityState>> {
        let mut grouped = HashMap::new();

        for state in &self.company_similarity_states {
            let symbol = self
                .company_symbols_list
                .get(state.company_index)
                .map(|(s, _)| s.clone())
                .expect("Failed to retrieve symbol for company index");

            grouped
                .entry(symbol)
                .or_insert_with(Vec::new)
                .push(state.clone());
        }

        // Sort each group by query_token_index
        for states in grouped.values_mut() {
            states.sort_by_key(|state| state.query_token_index);
        }

        grouped
    }
}
