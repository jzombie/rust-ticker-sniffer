use crate::types::{
    CompanySymbolsList, CompanyTokenSourceType, TickerSymbol, TokenizerVectorTokenType,
};
use crate::utils::cosine_similarity;
use crate::{CompanyTokenProcessor, Tokenizer};
use core::f64;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::thread::current;

type QueryTokenIndex = usize;
type TokenWindowIndex = usize;

pub struct TickerExtractorConfig {
    pub min_text_doc_token_sim_threshold: f64,
    // pub token_length_diff_tolerance: usize,
    pub token_window_size: usize,
    pub token_gap_penalty: f64,
    pub low_confidence_penalty_factor: f64,
}

#[derive(Debug, Clone)]
struct QueryVectorIntermediateSimilarityState {
    token_window_index: usize,
    query_token_index: usize,
    query_vector: TokenizerVectorTokenType,
    company_index: usize,
    company_token_type: CompanyTokenSourceType,
    company_token_index_by_source_type: usize,
    company_token_vector: TokenizerVectorTokenType,
    company_name_similarity_at_index: f64,
}

pub struct TickerExtractor<'a> {
    company_symbols_list: &'a CompanySymbolsList,
    ticker_symbol_tokenizer: Tokenizer,
    text_doc_tokenizer: Tokenizer,
    company_token_processor: CompanyTokenProcessor<'a>,
    user_config: TickerExtractorConfig,
    is_extracting: bool,
    text: Option<String>,
    tokenized_query_vectors: Vec<Vec<u32>>,
    company_similarity_states: Vec<QueryVectorIntermediateSimilarityState>,
    progressible_company_indices: HashSet<usize>,
    results: Vec<TickerSymbol>,
}

impl<'a> TickerExtractor<'a> {
    pub fn new(
        company_symbols_list: &'a CompanySymbolsList,
        user_config: TickerExtractorConfig,
    ) -> Self {
        let ticker_symbol_tokenizer = Tokenizer::ticker_symbol_parser();
        let text_doc_tokenizer = Tokenizer::text_doc_parser();

        let company_token_processor = CompanyTokenProcessor::new(&company_symbols_list);

        Self {
            company_symbols_list,
            ticker_symbol_tokenizer,
            text_doc_tokenizer,
            company_token_processor,
            user_config,
            is_extracting: false,
            text: None,
            tokenized_query_vectors: vec![],
            company_similarity_states: vec![],
            progressible_company_indices: HashSet::new(),
            results: vec![],
        }
    }

    pub fn extract(&mut self, text: &str) {
        if self.is_extracting {
            panic!("Cannot perform multiple extractions concurrently from same `TickerExtractor` instance");
        } else {
            self.is_extracting = true;
        }

        self.company_similarity_states.clear();
        self.progressible_company_indices.clear();
        self.results.clear();

        self.text = Some(text.to_string());
        self.tokenized_query_vectors = self.text_doc_tokenizer.tokenize_to_charcode_vectors(&text);

        // Begin parsing at the first page
        self.parse_company_names(None);

        self.collect_results();
    }

    fn collect_results(&self) {
        let coverage_grouped_results = self.collect_coverage_filtered_results();
        let confidence_scores = self.calculate_confidence_scores();

        for (symbol, states) in coverage_grouped_results {
            let confidence = confidence_scores
                .get(&symbol)
                .expect("Could not obtain confidence score");

            println!("Symbol: {}, Confidence: {}", symbol, confidence);

            for state in states {
                let query_token = self
                    .text_doc_tokenizer
                    .charcode_vector_to_token(&state.query_vector);

                let company_token = self
                    .text_doc_tokenizer
                    .charcode_vector_to_token(&state.company_token_vector);

                println!(
                    r#"
                        {} : {}
                        Tokenized Entries: {:?},
                        Query Token Index: {}
                        Token Window Index: {}
                        Company Name Similarity at Index: {},
                        State: {:?}
                    "#,
                    query_token,
                    company_token,
                    self.company_token_processor
                        .get_company_name_tokens(state.company_index),
                    state.query_token_index,
                    state.token_window_index,
                    state.company_name_similarity_at_index,
                    state
                );
            }
        }

        // TODO: For each query token index, take the symbol with the highest confidence score
    }

    fn calculate_confidence_scores(&self) -> HashMap<TickerSymbol, f64> {
        let coverage_grouped_results = self.collect_coverage_filtered_results();

        let mut confidence_scores: HashMap<TickerSymbol, f64> = HashMap::new();
        let mut all_scores = Vec::new();

        // First pass: Calculate initial confidence scores
        for (symbol, states) in &coverage_grouped_results {
            let mut combined_similarity: f64 = 0.0;

            for (i, state) in states.iter().enumerate() {
                let mut inverse_weight = 1.0;

                if i > 0 {
                    // Calculate the gap between the current and previous indices
                    let prev_query_token_index = states[i - 1].query_token_index;
                    let gap = (state.query_token_index as isize - prev_query_token_index as isize)
                        .abs() as usize;

                    // Apply a penalty for larger gaps (e.g., inverse weighting)
                    inverse_weight = if gap > 1 {
                        1.0 / (self.user_config.token_gap_penalty + gap as f64)
                    } else {
                        1.0
                    };
                }

                // Weigh the similarity score based on the calculated weight
                combined_similarity += state.company_name_similarity_at_index * inverse_weight;
            }

            confidence_scores.insert(symbol.clone(), combined_similarity);
            all_scores.push(combined_similarity);
        }

        // ------------------

        // Analyze the distribution of scores
        let mean_score: f64 = all_scores.iter().copied().sum::<f64>() / all_scores.len() as f64;
        let std_dev: f64 = (all_scores
            .iter()
            .map(|&score| (score - mean_score).powi(2))
            .sum::<f64>()
            / all_scores.len() as f64)
            .sqrt();
        let threshold = mean_score - std_dev; // Scores below this threshold are penalized further

        // Calculate the sum of all scores below the threshold
        let total_low_scores: f64 = confidence_scores
            .values()
            .filter(|&&score| score < threshold)
            .sum::<f64>();

        // Second pass: Penalize scores below the threshold based on their proportion
        for (_symbol, score) in &mut confidence_scores {
            if *score < threshold && total_low_scores > 0.0 {
                // Calculate the proportion of this score relative to the total low scores
                let proportion = *score / total_low_scores;

                // Penalize the score based on its proportion
                *score *= proportion * self.user_config.low_confidence_penalty_factor;
            }
        }

        confidence_scores
    }

    fn collect_coverage_filtered_results(
        &self,
    ) -> HashMap<TickerSymbol, Vec<QueryVectorIntermediateSimilarityState>> {
        let grouped_states = self.group_by_symbol();
        let coverage_increase_states = self.analyze_coverage_increase();

        let mut coverage_grouped: HashMap<
            TickerSymbol,
            Vec<QueryVectorIntermediateSimilarityState>,
        > = HashMap::new();

        for (symbol, states) in grouped_states {
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

    fn calc_token_window_indexes(&self, token_window_index: usize) -> (usize, usize) {
        let token_start_index = token_window_index * self.user_config.token_window_size;
        let token_end_index = token_start_index + self.user_config.token_window_size;

        (token_start_index, token_end_index)
    }

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

    fn analyze_coverage_increase(&self) -> HashMap<TickerSymbol, Vec<QueryTokenIndex>> {
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

    fn parse_company_names(&mut self, token_window_index: Option<usize>) {
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
                    // Note: Cosine similarity isn't used for "semantic relevance" in this context
                    // because these vectors are just simple vectors obtained from character codes.
                    // But the algorithm happens to be pretty efficient at what it does and seems
                    // faster at making comparisons than other algorithms I have experimented with.
                    let similarity = cosine_similarity(&query_vector, company_token_vector);

                    if similarity >= self.user_config.min_text_doc_token_sim_threshold {
                        window_match_count += 1;

                        // let company_name_length = company_name.len();

                        let total_company_name_tokens_length = self
                            .company_token_processor
                            .get_company_name_tokens_length(*company_index);

                        let company_name_similarity_at_index = similarity
                            * (query_vector.len() as f64 / total_company_name_tokens_length as f64);

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
}
