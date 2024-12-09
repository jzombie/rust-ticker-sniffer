use crate::types::{CompanySymbolsList, CompanyTokenSourceType, TickerSymbol};
use crate::utils::cosine_similarity;
use crate::{CompanyTokenProcessor, Tokenizer};
use std::fmt;

pub struct TickerExtractorWeights {
    pub min_similarity_threshold: f32,
    pub token_length_diff_tolerance: usize,
}

impl fmt::Display for TickerExtractorWeights {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let struct_name = stringify!(Weights);
        let fields = vec![
            ("min_similarity_threshold", self.min_similarity_threshold),
            (
                "token_length_diff_tolerance",
                self.token_length_diff_tolerance as f32,
            ),
        ];

        writeln!(f, "{} (", struct_name)?;
        for (name, value) in fields {
            writeln!(f, "\t{}: {},", name, value)?;
        }
        write!(f, ")") // Final closing parenthesis
    }
}

pub struct TickerExtractor<'a> {
    ticker_symbol_tokenizer: Tokenizer,
    text_doc_tokenizer: Tokenizer,
    company_token_processor: CompanyTokenProcessor<'a>,
    text: Option<String>,
    weights: TickerExtractorWeights,
    tokenized_query_vectors: Vec<Vec<u32>>,
    results: Vec<TickerSymbol>,
}

impl<'a> TickerExtractor<'a> {
    pub fn new(
        company_symbols_list: &'a CompanySymbolsList,
        weights: TickerExtractorWeights,
    ) -> Self {
        let ticker_symbol_tokenizer = Tokenizer::ticker_symbol_parser();
        let text_doc_tokenizer = Tokenizer::text_doc_parser();

        let company_token_processor = CompanyTokenProcessor::new(&company_symbols_list);

        Self {
            ticker_symbol_tokenizer,
            text_doc_tokenizer,
            company_token_processor,
            text: Some("".to_string()),
            // TODO: Apply default weights
            weights,
            tokenized_query_vectors: vec![],
            results: vec![],
        }
    }

    pub fn extract(&mut self, text: &str) {
        self.text = Some(text.to_string());
        self.tokenized_query_vectors = self.text_doc_tokenizer.tokenize_to_charcode_vectors(&text);

        self.parse(0)
    }

    // TODO: Handle "intermediate results collector"
    fn parse(&mut self, token_window_index: usize) {
        println!(
            "Query tokens: {:?}",
            self.tokenized_query_vectors
                .iter()
                .map(|vector| self.text_doc_tokenizer.charcode_vector_to_token(vector))
                .collect::<Vec<String>>()
        );

        // let length_tolerance: usize = 0;
        let mut match_count: usize = 0;
        for (query_token_index, query_vector) in self.tokenized_query_vectors.iter().enumerate() {
            // println!(
            //     "index: {}, query: {}",
            //     index,
            //     charcode_vector_to_token(query_vector)
            // );

            let query_vector_length = query_vector.len();

            // TODO: Use to help make queries like "Google" -> "GOOGL" work
            // let min_token_length =
            //     (query_vector_length - length_tolerance).clamp(1, query_vector_length);
            // let max_token_length = query_vector_length + length_tolerance;

            // TODO: This should perform multiple passes, moving the window up as it goes; results that are no
            // longer present in a subsequent pass should be capped off at that; each query token (at the
            // relevant index) that is matched, should be stored, where the longest consecutive matches of tokens
            // for a particular query should be scored higher than the others; where each token at each index,
            // ultimately, should only be associated with a single company, if any at all.
            //
            let token_start_index = 0;
            let token_end_index = 3;

            // let include_source_types = &[CompanyTokenSourceType::CompanyName];

            let token_length_bins = self
                .company_token_processor
                .token_length_bins
                .get(query_vector_length);

            match token_length_bins {
                Some(bins) => {
                    for (company_index, company_token_index) in bins {
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

                            // if similarity > 0.999 {
                            if similarity == 1.0 {
                                // println!(
                                //     "Matched company: {:?}; Token Index: {}",
                                //     company_symbols_list.get(*company_index),
                                //     query_token_index
                                // );

                                match_count += 1;
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

            // let results_iter = token_processor.filter_token_space(
            //     min_token_length,
            //     max_token_length,
            //     token_start_index,
            //     token_end_index,
            //     include_source_types,
            // );

            // for (index, result) in results_iter.enumerate() {
            //     // println!(
            //     //     "#: {}, Company Index: {}, Token Index: {} - Symbol: {} - String Token: {:?} - Source Type: {:?} - Company Tokens: {:?}",
            //     //     index,
            //     //     result.company_index,
            //     //     result.token_index,
            //     //     result.symbol,
            //     //     charcode_vector_to_token(result.token_vector),
            //     //     result.source_type,
            //     //     result.company_tokenized_entries
            //     // );
            //     let (padded_query_vector, padded_result_vector) =
            //         pad_vectors_to_match(query_vector, &result.token_vector);

            //     let similarity = cosine_similarity(&padded_query_vector, &padded_result_vector);

            //     // if similarity == 1.0 {
            //     //     println!(
            //     //         "Similarity: {}, {}, {:?}",
            //     //         similarity, result.symbol, result.company_name
            //     //     );
            //     // }
            // }
        }

        // TODO: Handle
        println!("Matches: {:?}", match_count);
    }
}
