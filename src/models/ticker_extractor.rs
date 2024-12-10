use crate::types::{
    CompanySymbolsList, CompanyTokenSourceType, TickerSymbol, TokenizerVectorTokenType,
};
use crate::utils::cosine_similarity;
use crate::{CompanyTokenProcessor, Tokenizer};
use std::collections::{BTreeMap, HashMap, HashSet};

type QueryTokenIndex = usize;
type TokenWindowIndex = usize;

pub struct TickerExtractorConfig {
    pub min_text_doc_token_sim_threshold: f64,
    // pub token_length_diff_tolerance: usize,
    pub token_window_size: usize,
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
    accumulated_company_name_coverage: f64,
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
        self.parse(None);

        self.collect_results();
    }

    fn collect_results(&self) {
        // Group similarity states by ticker symbol then order them by `query_token_index`
        let mut ordered_collection: HashMap<
            TickerSymbol,
            BTreeMap<QueryTokenIndex, QueryVectorIntermediateSimilarityState>,
        > = HashMap::new();

        // Collect state into ordered collection
        for similarity_state in &self.company_similarity_states {
            let (company_token_vector, company_token_type, _company_token_index_by_source_type) =
                &self.company_token_processor.tokenized_entries[similarity_state.company_index]
                    [similarity_state.company_token_index_by_source_type];

            // println!(
            //     "Similarity state: {:?}, Symbols entry: {:?}, Token: {:?}, Token Type: {:?}",
            //     similarity_state,
            //     self.company_symbols_list
            //         .get(similarity_state.company_index),
            //     self.text_doc_tokenizer
            //         .charcode_vector_to_token(company_token_vector),
            //     company_token_type
            // );

            // Retrieve the symbol for the given company index
            let ticker_symbol = match self
                .company_symbols_list
                .get(similarity_state.company_index)
            {
                Some((ticker_symbol, _)) => ticker_symbol,
                None => unreachable!("Could not obtain ticker symbol"),
            };

            // Get or insert the symbol group (BTreeMap for query token index -> similarity state)
            let symbol_group = ordered_collection
                .entry(ticker_symbol.clone())
                .or_insert_with(BTreeMap::new);

            // Insert the similarity state directly for the given query token index
            symbol_group.insert(similarity_state.query_token_index, similarity_state.clone());

            // Token Order Bonus: Reward matches where the query_token_index aligns with the query sequence.
            // let order_bonus = if query_token_index == token_window_index { 1.0 } else { 0.5 };

            // Proximity Penalty: Penalize matches that span a large range of query tokens.
            // let proximity_penalty = 1.0 / (1.0 + (end_index - start_index) as f64);
        }

        // println!("\n\n\n{:?}\n\n\n", ordered_collection);

        // TODO: Locate the results with the highest token window index and figure out which query token indexes make it up,
        // ensuring that a query like "Berkshire Hathaway is not Apple, but owns Apple, of course, which is not Apple Hospitality REIT."
        // correctly identifies, "BRK-A", "BRK-B", "AAPL", and "APLE" as top matches
        let highest_token_window_states = self.find_highest_token_window_states();

        for (symbol, intermediate_states) in ordered_collection {
            let highest_token_window_index = match highest_token_window_states.get(&symbol) {
                Some((highest_token_window_index, _)) => highest_token_window_index,
                None => unreachable!("Could not obtain highest window index"),
            };

            println!(
                "Symbol: {}, Highest token window index: {:?}",
                symbol, highest_token_window_index
            );

            for (state_index, state) in intermediate_states {
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
                        Accumulated Company Name Coverage at Index: {},
                        State: {:?}
                    "#,
                    query_token,
                    company_token,
                    self.company_token_processor
                        .get_company_name_tokens(state.company_index),
                    state_index,
                    state.token_window_index,
                    state.company_name_similarity_at_index,
                    state.accumulated_company_name_coverage,
                    state
                );

                // println!("\n");

                // let tokenized_query_vectors = self.tokenized_query_vectors.get(state.company_index);

                // println!(
                //     "\t\tWord: {:?}",

                // );
            }
        }
    }

    fn find_highest_token_window_states(&self) -> HashMap<TickerSymbol, (usize, Vec<usize>)> {
        // Group by symbol, then by the highest token_window_index
        let mut top_matches: HashMap<TickerSymbol, (TokenWindowIndex, Vec<QueryTokenIndex>)> =
            HashMap::new();

        for similarity_state in &self.company_similarity_states {
            let ticker_symbol = self
                .company_symbols_list
                .get(similarity_state.company_index)
                .map(|(symbol, _)| symbol.clone())
                .unwrap_or_else(|| "Unknown".to_string());

            let entry = top_matches
                .entry(ticker_symbol.clone())
                .or_insert((0, Vec::new()));

            // If this state has a higher token_window_index, update the entry
            if similarity_state.token_window_index > entry.0 {
                entry.0 = similarity_state.token_window_index;
                entry.1 = vec![similarity_state.query_token_index];
            } else if similarity_state.token_window_index == entry.0 {
                // If the token_window_index matches the current max, append the query_token_index
                entry.1.push(similarity_state.query_token_index);
            }
        }

        // Display the results
        // println!("Results by token window index:");
        // for (symbol, (max_window_index, query_indexes)) in &top_matches {
        //     println!(
        //         "Symbol: {}, Highest Token Window Index: {}, Query Token Indexes: {:?}",
        //         symbol, max_window_index, query_indexes
        //     );
        // }

        top_matches
    }

    fn calc_token_window_indexes(&self, token_window_index: usize) -> (usize, usize) {
        let token_start_index = token_window_index * self.user_config.token_window_size;
        let token_end_index = token_start_index + self.user_config.token_window_size;

        (token_start_index, token_end_index)
    }

    fn parse(&mut self, token_window_index: Option<usize>) {
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

            match self
                .company_token_processor
                .token_length_bins
                .get(query_vector_length)
            {
                Some(bins) => {
                    for (company_index, tokenized_entry_index) in bins {
                        if token_window_index > 0
                            && !self.progressible_company_indices.contains(company_index)
                        {
                            continue;
                        }

                        let (
                            company_token_vector,
                            company_token_type,
                            company_token_index_by_source_type,
                        ) = &self.company_token_processor.tokenized_entries[*company_index]
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
                                // println!(
                                //     "Matched company: {:?}; Token Index: {}",
                                //     self.company_symbols_list.get(*company_index),
                                //     query_token_index
                                // );

                                window_match_count += 1;

                                let company_name_length = match self
                                    .company_token_processor
                                    .company_name_lengths
                                    .get(*company_index)
                                {
                                    Some(company_name_length) => company_name_length,
                                    None => unreachable!("Could not obtain company name length"),
                                };

                                let company_name_token_count = match self
                                    .company_token_processor
                                    .company_name_token_counts
                                    .get(*company_index)
                                {
                                    Some(company_name_token_count) => company_name_token_count,
                                    None => {
                                        unreachable!("Could not obtain company name token count")
                                    }
                                };

                                let company_name_similarity_at_index = similarity
                                    / (*company_name_length as f64 + f64::EPSILON) as f64;

                                let accumulated_company_name_coverage = 1.0
                                    / (*company_name_token_count as f64
                                        / (*company_token_index_by_source_type as f64 + 1.0));

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
                                        accumulated_company_name_coverage,
                                    },
                                );

                                self.progressible_company_indices.insert(*company_index);
                            } else {
                                self.progressible_company_indices.remove(company_index);
                            }
                        }
                    }
                }

                None => unreachable!("Could not access target bins"),
            }
        }

        // Continue looping if new matches have been discovered
        if window_match_count > 0 {
            self.parse(Some(token_window_index + 1));
        }
    }
}
