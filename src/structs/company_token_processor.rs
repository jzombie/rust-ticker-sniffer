use crate::types::{
    CompanySequenceIndex, CompanySymbolList, TickerSymbol, TickerSymbolFrequencyMap,
    TickerSymbolTokenId, Token, TokenId,
};
use crate::utils::{count_ticker_symbol_frequencies, dedup_vector};
use crate::{CompanyTokenMapper, Error, TokenParityState, TokenRangeState};

use log::info;
use std::collections::HashMap;

type PotentialTokenSequenceMap =
    HashMap<TickerSymbolTokenId, Vec<(CompanySequenceIndex, Vec<TokenId>)>>;

pub struct CompanyTokenProcessorConfig {
    // TODO: Rename
    pub threshold_ratio_exact_matches: f32,
    pub threshold_min_company_token_coverage: f32,
}

pub struct CompanyTokenProcessor<'a> {
    config: &'a CompanyTokenProcessorConfig,
    company_token_mapper: CompanyTokenMapper,
}

impl<'a> CompanyTokenProcessor<'a> {
    /// Creates a new `CompanyTokenProcessor` with the given configuration and company symbol list.
    ///
    /// # Arguments
    /// * `config` - A reference to the configuration for processing tokens.
    /// * `company_symbol_list` - A reference to the list of company symbols.
    /// * `case_sensitive` - Whether or not the text document should be filtered by case sensitivity.
    ///
    /// # Errors
    /// Returns an error if initialization fails.
    pub fn new(
        config: &'a CompanyTokenProcessorConfig,
        company_symbol_list: &'a CompanySymbolList,
        case_sensitive: bool,
    ) -> Result<Self, Error> {
        let company_token_mapper = CompanyTokenMapper::new(company_symbol_list, case_sensitive)?;

        Ok(CompanyTokenProcessor {
            config,
            company_token_mapper,
        })
    }

    /// Processes a text document and extracts ticker symbols with their frequencies.
    ///
    /// # Arguments
    /// * `text` - The text document to process.
    ///
    /// # Errors
    /// Returns an error if the processing fails.
    pub fn process_text_doc(&mut self, text: &str) -> Result<TickerSymbolFrequencyMap, Error> {
        // Tokenize the input text
        info!("Tokenizing...");

        // Note: Being pre-filtered, these may contain tokens that are not actually ticker symbol tokens,
        // but meet the Tokenizer requirements for them.
        let ticker_symbol_tokens_pre_filtered = self
            .company_token_mapper
            .ticker_symbol_tokenizer
            .tokenize(text);

        let text_doc_tokens_pre_filtered =
            self.company_token_mapper.text_doc_tokenizer.tokenize(text);

        info!("Gathering filtered tokens...");
        let (query_text_doc_token_ids, mut query_ticker_symbol_token_ids) = self
            .get_filtered_query_token_ids(
                &text_doc_tokens_pre_filtered,
                &ticker_symbol_tokens_pre_filtered,
            );

        // Identify token ID sequences which start with the first token of a company token sequence
        info!("Identifying token ID sequences...");
        let potential_token_id_sequences =
            self.get_potential_token_sequences(&query_text_doc_token_ids)?;

        // Aggregate token parity states
        info!("Collecting token parity states...");
        let token_parity_states = TokenParityState::collect_token_parity_states(
            &query_text_doc_token_ids,
            &potential_token_id_sequences,
        );

        // Determine range states
        info!("Collecting token range states...");
        let mut token_range_states = TokenRangeState::collect_token_range_states(
            &self.company_token_mapper,
            &potential_token_id_sequences,
            &token_parity_states,
        )?;

        // Assign scores to the range states
        info!("Assigning range scores...");
        TokenRangeState::assign_token_range_scores(
            &query_text_doc_token_ids,
            &mut token_range_states,
        );

        // Discard token range states which do not meet minimum threshold
        token_range_states.retain(|state| {
            state.company_token_coverage >= self.config.threshold_min_company_token_coverage
        });

        // Collect top range states
        info!("Collecting top range states...");
        let top_range_states = TokenRangeState::collect_top_range_states(
            &query_text_doc_token_ids,
            &token_range_states,
        )?;

        // Used to determine whether to explicitly parse out symbols which may also be stop words, based on
        // percentage of symbols to company names in the doc (for instance, determine if "A" should be parsed
        // as a symbol)
        let ratio_exact_matches =
            TokenRangeState::calc_exact_ticker_symbol_match_ratio(&top_range_states);

        // Clear exact ticker symbol matches if ratio of exact matches is less than configured minimum
        if ratio_exact_matches < self.config.threshold_ratio_exact_matches {
            query_ticker_symbol_token_ids = vec![];
        }

        // Keep track of number of occurrences, per extracted symbol, for context stats
        let text_doc_ticker_frequencies =
            TokenRangeState::count_token_range_ticker_symbol_frequencies(&top_range_states);

        let query_ticker_symbols: Vec<&TickerSymbol> = query_ticker_symbol_token_ids
            .iter()
            .map(|token_id| {
                self.company_token_mapper
                    .get_ticker_symbol_by_token_id(token_id)
                    .map_err(|e| {
                        crate::Error::ParserError(format!(
                            "Failed to fetch token ID {:?}: {:?}",
                            token_id, e
                        ))
                    })
            })
            .collect::<Result<Vec<&TickerSymbol>, _>>()?;

        let unique_query_ticker_symbols = dedup_vector(&query_ticker_symbols);

        let unique_text_doc_ticker_symbols: Vec<TickerSymbol> =
            text_doc_ticker_frequencies.keys().cloned().collect();

        let query_tickers_not_in_text_doc: Vec<&TickerSymbol> = unique_query_ticker_symbols
            .clone()
            .into_iter()
            .filter(|symbol| !unique_text_doc_ticker_symbols.contains(symbol))
            .collect();

        let query_tickers_not_in_text_doc: Vec<TickerSymbol> = query_tickers_not_in_text_doc
            .iter()
            .cloned()
            .cloned()
            .collect();

        let mut query_ticker_frequencies =
            count_ticker_symbol_frequencies(&query_tickers_not_in_text_doc);

        self.adjust_query_ticker_frequencies(&mut query_ticker_frequencies, &top_range_states)?;

        let combined_ticker_frequencies = self.combine_ticker_symbol_frequencies(&[
            text_doc_ticker_frequencies.clone(),
            query_ticker_frequencies.clone(),
        ]);

        Ok(combined_ticker_frequencies)
    }

    /// Reduces ticker symbol frequency counts based on matches in token range states.
    ///
    /// # Arguments
    /// * `query_ticker_frequencies` - Mutable reference to the query ticker frequencies.
    /// * `top_range_states` - A reference to the top token range states.
    ///
    /// # Errors
    /// Returns an error if adjustment fails.
    fn adjust_query_ticker_frequencies(
        &self,
        query_ticker_frequencies: &mut TickerSymbolFrequencyMap,
        top_range_states: &[TokenRangeState],
    ) -> Result<(), Error> {
        for range_state in top_range_states {
            let range_text_doc_token_ids = &range_state.query_text_doc_token_ids;

            for (query_ticker_symbol, query_ticker_symbol_frequency) in
                query_ticker_frequencies.iter_mut()
            {
                let query_ticker_symbol_token_id = self
                    .company_token_mapper
                    .get_ticker_symbol_token_id(query_ticker_symbol)?;

                if range_text_doc_token_ids.contains(query_ticker_symbol_token_id) {
                    *query_ticker_symbol_frequency =
                        query_ticker_symbol_frequency.saturating_sub(1);
                }
            }
        }

        // Remove entries with zero frequencies
        query_ticker_frequencies.retain(|_, &mut frequency| frequency > 0);

        Ok(())
    }

    /// Combines multiple maps of ticker symbol frequencies into one.
    ///
    /// # Arguments
    /// * `ticker_symbol_frequency_hash_maps` - A slice of frequency maps to combine.
    ///
    /// # Returns
    /// A single map with combined frequencies.
    fn combine_ticker_symbol_frequencies(
        &self,
        ticker_symbol_frequency_hash_maps: &[TickerSymbolFrequencyMap],
    ) -> TickerSymbolFrequencyMap {
        let mut combined_ticker_frequencies: HashMap<TickerSymbol, usize> = HashMap::new();

        for frequency_hash_map in ticker_symbol_frequency_hash_maps {
            for (ticker_symbol, frequency) in frequency_hash_map {
                *combined_ticker_frequencies
                    .entry(ticker_symbol.clone())
                    .or_insert(0) += frequency;
            }
        }

        combined_ticker_frequencies
    }

    // TODO: Keep? If so, use logger method
    // #[allow(dead_code)]
    // /// For debugging purposes
    // fn display_company_tokens(&self, ticker_symbol: &TickerSymbol) {
    //     if let Some(company_token_sequences) = self.company_token_sequences.get(ticker_symbol) {
    //         for company_token_sequence in company_token_sequences {
    //             println!(
    //                 "{:?}",
    //                 self.token_mapper.get_tokens_by_ids(company_token_sequence)
    //             );
    //         }
    //     } else {
    //         println!("No tokens found for ticker symbol: {}", ticker_symbol);
    //     }
    // }

    /// Retrieves filtered token IDs for both text documents and ticker symbols.
    ///
    /// # Arguments
    /// * `text_doc_tokens` - Tokens from the text document.
    /// * `ticker_symbol_tokens` - Tokens from the ticker symbols.
    ///
    /// # Returns
    /// A tuple containing vectors of token IDs.
    fn get_filtered_query_token_ids(
        &self,
        text_doc_tokens: &[Token],
        ticker_symbol_tokens: &[Token],
    ) -> (Vec<TokenId>, Vec<TokenId>) {
        // Get the filtered token IDs (IDs present in the TokenMapper)
        let query_text_doc_token_ids = self
            .company_token_mapper
            .token_mapper
            .get_filtered_token_ids(text_doc_tokens.iter().map(|s| s.as_str()).collect());

        let query_ticker_symbol_token_ids = self
            .company_token_mapper
            .token_mapper
            .get_filtered_token_ids(ticker_symbol_tokens.iter().map(|s| s.as_str()).collect())
            .into_iter()
            // Filter to ticker symbol tokens
            .filter(|token_id| {
                self.company_token_mapper
                    .company_token_sequences_map
                    .contains_key(token_id)
            })
            .collect();

        (query_text_doc_token_ids, query_ticker_symbol_token_ids)
    }

    /// Identifies potential token sequences from the query tokens.
    ///
    /// # Arguments
    /// * `query_text_doc_token_ids` - A slice of token IDs from the query text document.
    ///
    /// # Errors
    /// Returns an error if the process fails.
    ///
    /// # Returns
    /// A map of potential token sequences.
    fn get_potential_token_sequences(
        &self,
        query_text_doc_token_ids: &[TokenId],
    ) -> Result<PotentialTokenSequenceMap, Error> {
        let mut potential_token_id_sequences: HashMap<
            TickerSymbolTokenId,
            Vec<(CompanySequenceIndex, Vec<TokenId>)>,
        > = HashMap::new();

        for query_token_id in query_text_doc_token_ids {
            if let Some(possible_ticker_symbol_token_ids) = self
                .company_token_mapper
                .company_reverse_token_map
                .get(query_token_id)
            {
                for ticker_symbol_token_id in possible_ticker_symbol_token_ids {
                    if let Some(company_name_variations_token_ids_list) = self
                        .company_token_mapper
                        .company_token_sequences_map
                        .get(ticker_symbol_token_id)
                    {
                        for (company_sequence_idx, company_name_variations_token_ids) in
                            company_name_variations_token_ids_list.iter().enumerate()
                        {
                            if company_name_variations_token_ids.is_empty() {
                                continue;
                            }

                            let company_name_first_token_id = company_name_variations_token_ids[0];

                            if *query_token_id == company_name_first_token_id {
                                // Add or update the hashmap entry for this ticker_symbol
                                potential_token_id_sequences
                                    .entry(*ticker_symbol_token_id)
                                    .or_default()
                                    .retain(|(existing_idx, existing_vec)| {
                                        *existing_idx != company_sequence_idx
                                            || *existing_vec != *company_name_variations_token_ids
                                    }); // Remove duplicates

                                // If the ticker_symbol_token_id exists in the HashMap, retrieve its mutable Vec reference
                                // and add the new (company_sequence_idx, company_name_variations_token_ids) entry to it.
                                if let Some(vec) =
                                    potential_token_id_sequences.get_mut(ticker_symbol_token_id)
                                {
                                    vec.push((
                                        company_sequence_idx,
                                        company_name_variations_token_ids.clone(),
                                    ));
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(potential_token_id_sequences)
    }
}
