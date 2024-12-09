use crate::types::{CompanySymbolsList, CompanyTokenSourceType, TickerSymbol};
use crate::utils::cosine_similarity;
use crate::{CompanyTokenProcessor, Tokenizer};
use std::collections::HashSet;
// use std::fmt;

pub struct TickerExtractorConfig {
    pub min_similarity_threshold: f64,
    pub token_length_diff_tolerance: usize,
    pub token_window_size: usize,
}

// TODO: Is this needed?
// impl fmt::Display for TickerExtractorConfig {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         let struct_name = stringify!(Weights);
//         let fields = vec![
//             (
//                 "min_similarity_threshold",
//                 self.min_similarity_threshold as f32,
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

#[derive(Debug)]
struct QueryVectorCompanySimilarityState {
    query_token_index: usize,
    company_index: usize,
    company_token_index: usize,
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
    company_similarity_states: Vec<QueryVectorCompanySimilarityState>,
    seen_company_indices: HashSet<usize>,
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
            seen_company_indices: HashSet::new(),
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
        self.seen_company_indices.clear();
        self.results.clear();

        self.text = Some(text.to_string());
        self.tokenized_query_vectors = self.text_doc_tokenizer.tokenize_to_charcode_vectors(&text);

        self.parse(0);

        for similarity_state in &self.company_similarity_states {
            println!(
                "Similarity state: {:?}, Symbols entry: {:?}",
                similarity_state,
                self.company_symbols_list
                    .get(similarity_state.company_index)
            );
        }
    }

    fn calc_token_window_indexes(&self, token_window_index: usize) -> (usize, usize) {
        let token_start_index = token_window_index * self.user_config.token_window_size;
        let token_end_index = token_start_index + self.user_config.token_window_size;

        (token_start_index, token_end_index)
    }

    // TODO: Handle "intermediate results collector"
    fn parse(&mut self, token_window_index: usize) {
        // println!(
        //     "Query tokens: {:?}",
        //     self.tokenized_query_vectors
        //         .iter()
        //         .map(|vector| self.text_doc_tokenizer.charcode_vector_to_token(vector))
        //         .collect::<Vec<String>>()
        // );

        let (token_start_index, token_end_index) =
            self.calc_token_window_indexes(token_window_index);

        println!(
            "Start index: {}, End index: {}",
            token_start_index, token_end_index
        );

        let mut window_match_count: usize = 0;

        for (query_token_index, query_vector) in self.tokenized_query_vectors.iter().enumerate() {
            // // Ensure the vector has enough capacity to accommodate the current query_token_index
            // if self.query_vector_states.len() <= query_token_index {
            //     self.query_vector_states.resize(
            //         query_token_index + 1,
            //         QueryVectorState {
            //             query_token_index: 0,
            //             company_similarity_states: vec![],
            //         },
            //     );
            // }

            // // Access or create the state at the desired index
            // let query_vector_state = &mut self.query_vector_states[query_token_index];
            // query_vector_state.query_token_index = query_token_index; // Update if necessary

            // println!(
            //     "index: {}, query_vector_state: {:?}",
            //     query_token_index, query_vector_state
            // );

            let query_vector_length = query_vector.len();

            // TODO: Use to help make queries like "Google" -> "GOOGL" work
            // let min_token_length =
            //     (query_vector_length - self.weights.token_length_diff_tolerance).clamp(1, query_vector_length);
            // let max_token_length = query_vector_length + self.weights.token_length_diff_tolerance;

            // let include_source_types = &[CompanyTokenSourceType::CompanyName];

            let token_length_bins = self
                .company_token_processor
                .token_length_bins
                .get(query_vector_length);

            match token_length_bins {
                Some(bins) => {
                    for (company_index, company_token_index) in bins {
                        if token_window_index > 0
                            && !self.seen_company_indices.contains(company_index)
                        {
                            continue;
                        }

                        if company_token_index >= &token_start_index
                            && company_token_index < &token_end_index
                        {
                            let (company_token_vector, company_token_type) = &self
                                .company_token_processor
                                .tokenized_entries[*company_index][*company_token_index];

                            // Uncomment this condition if filtering is needed
                            if *company_token_type != CompanyTokenSourceType::CompanyName {
                                continue;
                            }

                            // Note: Cosine similarity isn't used for "semantic relevance" in this context
                            // because these vectors are just simple vectors obtained from character codes.
                            // But the algorithm happens to be pretty efficient at what it does and seems
                            // faster at making comparisons than other algorithms I have experimented with.
                            let similarity = cosine_similarity(&query_vector, company_token_vector);

                            // let (padded_query_vector, padded_company_token_vector) =
                            //     pad_vectors_to_match(&query_vector, &company_token_vector);
                            // let similarity =
                            //     cosine_similarity(&padded_query_vector, &padded_company_token_vector);

                            if similarity >= self.user_config.min_similarity_threshold {
                                // println!(
                                //     "Matched company: {:?}; Token Index: {}",
                                //     company_symbols_list.get(*company_index),
                                //     query_token_index
                                // );

                                window_match_count += 1;

                                self.company_similarity_states.push(
                                    QueryVectorCompanySimilarityState {
                                        query_token_index,
                                        company_index: *company_index,
                                        company_token_index: *company_token_index,
                                        similarity: (similarity * token_window_index as f64 + 1.0),
                                    },
                                );

                                self.seen_company_indices.insert(*company_index);
                            }
                        }
                    }
                }

                None => {
                    // println!(
                    //     "No bins found for token lengths between {} and {}",
                    //     min_token_length, max_token_length
                    // );
                }
            }
        }

        // TODO: After the first window, query tokens which didn't have a match
        // can probably be completely discarded, as there is no point in
        // querying them again. Perhaps just mark the indexes as "used up"
        // and use the `continue` keyword in
        // ` self.tokenized_query_vectors.iter().enumerate()`.

        // TODO: Remove; just for debugging
        println!("Matches: {:?}", window_match_count);

        if window_match_count > 0 {
            self.parse(token_window_index + 1);
        }
    }
}
