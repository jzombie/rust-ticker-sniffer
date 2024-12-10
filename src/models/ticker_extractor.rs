use crate::types::{CompanySymbolsList, CompanyTokenSourceType, TickerSymbol};
use crate::utils::cosine_similarity;
use crate::{CompanyTokenProcessor, Tokenizer};
use std::collections::{BTreeMap, HashMap, HashSet};
// use std::fmt;

pub struct TickerExtractorConfig {
    pub min_text_doc_token_sim_threshold: f64,
    // pub token_length_diff_tolerance: usize,
    pub token_window_size: usize,
}

// TODO: Is this needed?
// impl fmt::Display for TickerExtractorConfig {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         let struct_name = stringify!(Weights);
//         let fields = vec![
//             (
//                 "min_text_doc_token_sim_threshold",
//                 self.min_text_doc_token_sim_threshold as f32,
//             ),
//             (
//                 "token_length_diff_tolerance",
//                 self.token_length_diff_tolerance as f32,
//             ),
//         ];

//         writeln!(f, "{} (", struct_name)?;
//         for (name, value) in fields {
//             writeln!(f, "\t{}: {},", name, value)?;
//         }
//         write!(f, ")") // Final closing parenthesis
//     }
// }

#[derive(Debug, Clone, Copy)]
struct QueryVectorIntermediateSimilarityState {
    token_window_index: usize,
    query_token_index: usize,
    company_index: usize,
    company_token_type: CompanyTokenSourceType,
    company_token_index_by_source_type: usize,
    similarity: f64,
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
        type QueryTokenIndex = usize;

        // Group similarity states by ticker symbol then order them by `query_token_index`
        let mut ordered_collection: HashMap<
            TickerSymbol,
            BTreeMap<QueryTokenIndex, QueryVectorIntermediateSimilarityState>,
        > = HashMap::new();

        for similarity_state in &self.company_similarity_states {
            let (company_token_vector, company_token_type, _company_token_index_by_source_type) =
                &self.company_token_processor.tokenized_entries[similarity_state.company_index]
                    [similarity_state.company_token_index_by_source_type];

            println!(
                "Similarity state: {:?}, Symbols entry: {:?}, Token: {:?}, Token Type: {:?}",
                similarity_state,
                self.company_symbols_list
                    .get(similarity_state.company_index),
                self.text_doc_tokenizer
                    .charcode_vector_to_token(company_token_vector),
                company_token_type
            );

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
            symbol_group.insert(similarity_state.query_token_index, *similarity_state);

            // Token Order Bonus: Reward matches where the query_token_index aligns with the query sequence.
            // let order_bonus = if query_token_index == token_window_index { 1.0 } else { 0.5 };

            // Proximity Penalty: Penalize matches that span a large range of query tokens.
            // let proximity_penalty = 1.0 / (1.0 + (end_index - start_index) as f64);
        }

        println!("\n\n\n{:?}\n\n\n", ordered_collection);

        for (symbol, intermediate_states) in ordered_collection {
            println!("Symbol: {}", symbol);

            for (state_index, state) in intermediate_states {
                println!("\t{} State: {:?}", state_index, state);
                //     println!(
                //         "\t\t{:?}",
                //         self.company_symbols_list.get(state.company_index)
                //     );
            }
        }

        // TODO: Apply a penalty if the `query_token_type` and `query_token_index` are not in order of constituents
        // Query: REIT Hospitality Apple stuff
        // Start index: 0, End index: 1
        // Matched company: Some(("AAPL", Some("Apple Inc."))); Token Index: 2
        // Matched company: Some(("APLE", Some("Apple Hospitality REIT, Inc."))); Token Index: 2
        // Matches: 2
        // Start index: 1, End index: 2
        // Matched company: Some(("APLE", Some("Apple Hospitality REIT, Inc."))); Token Index: 1
        // Matches: 1
        // Start index: 2, End index: 3
        // Matched company: Some(("APLE", Some("Apple Hospitality REIT, Inc."))); Token Index: 0
        // Matches: 1
        // Start index: 3, End index: 4
        // Matches: 0
        // Similarity state: QueryVectorIntermediateSimilarityState { token_window_index: 0, query_token_index: 2, company_index: 34, company_token_type: CompanyName, company_token_index_by_source_type: 0, similarity: 0.09090909090909091 }, Symbols entry: Some(("AAPL", Some("Apple Inc."))), Token: "AAPL", Token Type: Symbol
        // Similarity state: QueryVectorIntermediateSimilarityState { token_window_index: 0, query_token_index: 2, company_index: 721, company_token_type: CompanyName, company_token_index_by_source_type: 0, similarity: 0.034482758620689655 }, Symbols entry: Some(("APLE", Some("Apple Hospitality REIT, Inc."))), Token: "APLE", Token Type: Symbol
        // Similarity state: QueryVectorIntermediateSimilarityState { token_window_index: 1, query_token_index: 1, company_index: 721, company_token_type: CompanyName, company_token_index_by_source_type: 1, similarity: 0.03448275862068966 }, Symbols entry: Some(("APLE", Some("Apple Hospitality REIT, Inc."))), Token: "APPLE", Token Type: CompanyName
        // Similarity state: QueryVectorIntermediateSimilarityState { token_window_index: 2, query_token_index: 0, company_index: 721, company_token_type: CompanyName, company_token_index_by_source_type: 2, similarity: 0.03448275862068966 }, Symbols entry: Some(("APLE", Some("Apple Hospitality REIT, Inc."))), Token: "HOSPITALITY", Token Type: CompanyName
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
                                println!(
                                    "Matched company: {:?}; Token Index: {}",
                                    self.company_symbols_list.get(*company_index),
                                    query_token_index
                                );

                                window_match_count += 1;

                                match self
                                    .company_token_processor
                                    .company_name_lengths
                                    .get(*company_index)
                                {
                                    Some(company_name_length) => {
                                        self.company_similarity_states.push(
                                            QueryVectorIntermediateSimilarityState {
                                                token_window_index,
                                                query_token_index,
                                                company_index: *company_index,
                                                company_token_type: *company_token_type,
                                                company_token_index_by_source_type:
                                                    *company_token_index_by_source_type,
                                                similarity: similarity
                                                    / (company_name_length + 1) as f64,
                                            },
                                        );
                                    }
                                    None => {
                                        unreachable!()
                                    }
                                }

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
