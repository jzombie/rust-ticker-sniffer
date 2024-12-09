mod constants;
// use crate::constants::STOP_WORDS;
pub mod models;
pub use constants::{
    DEFAULT_BIAS_ADJUSTER_SCORE, DEFAULT_CONFIG, DEFAULT_RESULT_BIAS_ADJUSTER_WEIGHTS, TLD_LIST,
};
pub use models::{
    CompanyNameTokenRanking, CompanyTokenProcessor, ResultBiasAdjuster, TickerExtractor,
    TickerExtractorConfig, Tokenizer,
};
pub mod utils;
pub use utils::{cosine_similarity, pad_vector, pad_vectors_to_match};
pub mod types;
pub use types::{
    CompanyName, CompanySymbolsList, CompanyTokenSourceType, TickerSymbol, TokenizerVectorTokenType,
};

pub fn extract_tickers_from_text(
    text: &str,
    company_symbols_list: &CompanySymbolsList,
) -> (Vec<TickerSymbol>, f32, Vec<CompanyNameTokenRanking>) {
    let result_bias_adjuster =
        ResultBiasAdjuster::from_weights(DEFAULT_RESULT_BIAS_ADJUSTER_WEIGHTS);

    extract_tickers_from_text_with_custom_weights(
        &text,
        &company_symbols_list,
        DEFAULT_CONFIG,
        &result_bias_adjuster,
    )
}

pub fn extract_tickers_from_text_with_custom_weights(
    text: &str,
    company_symbols_list: &CompanySymbolsList,
    weights: TickerExtractorConfig,
    _result_bias_adjuster: &ResultBiasAdjuster,
) -> (Vec<TickerSymbol>, f32, Vec<CompanyNameTokenRanking>) {
    // let mut matches = HashSet::new();

    // TODO: Commit to tests; expect: Tokens: ["WELL", "IPHONE", "DEVELOPMENT", "EBAY", "DEVELOPMENT", "WALMART", "WALMARTS"]
    // let tokens = tokenize(
    //     &"Well okay iPhone turtle develoPment e-Bay  Deve-\nlopment Wal-mart's at it again!",
    // );

    // TODO: Commit to tests; expect: Tokens: ["ECOMMERCE", "AMAZONCOM", "AMAZON", "INC", "AMZN", "QUICK", "QUOTEAMZN", "FREE", "REPORT", "DOW", "JONES", "INDUSTRIAL", "AVERAGE", "WALGREENS", "BOOTS", "ALLIANCE", "WBA", "QUICK", "QUOTEWBA", "FREE", "REPORT", "FEB", "THE", "THE", "AMAZON", "DOW"]
    // let tokens = tokenize(
    //     &"E-commerce giant Amazon.com Inc. (AMZN Quick QuoteAMZN - Free Report) joined the blue-chip index, Dow Jones Industrial Average, replacing drugstore operator Walgreens Boots Alliance (WBA Quick QuoteWBA - Free Report) on Feb 26. The reshuffle reflects the ongoing shift in economic power from traditional brick-and-mortar retail to e-commerce and technology-driven companies. The inclusion of Amazon in the Dow marks a significant milestone in the recognition of the e-commerce giant's influence and its role in the broader market.",
    // );

    // let tokens = tokenize("A cool test, way to go Apple, e-Trade");
    // let vectors: Vec<Vec<u32>> = tokens
    //     .into_iter()
    //     .map(|token| token_to_charcode_vector(&token)) // Each token produces a Vec<u32>
    //     .collect();

    // eprintln!("vectors: {:?}", vectors);

    // let token_processor = CompanyTokenProcessor::new(&company_symbols_list);

    // TODO: Refactor
    // Query for tokens of a specific length (e.g., length 8)
    // let min_token_legnth = 1;
    // let max_token_length = 20;
    // let token_start_index = 0;
    // let token_end_index = 4;
    // // let include_source_types = &[CompanyTokenSourceType::Symbol, TokenSourceType::CompanyName];
    // // let include_source_types = &[CompanyTokenSourceType::CompanyName];
    // let include_source_types = &[CompanyTokenSourceType::Symbol];
    // let results_iter = token_processor.filter_token_space(
    //     min_token_legnth,
    //     max_token_length,
    //     token_start_index,
    //     token_end_index,
    //     include_source_types
    // );

    // for (index, result) in results_iter.enumerate() {
    //     println!(
    //         "#: {}, Company Index: {}, Token Index: {} - Symbol: {} - String Token: {:?} - Source Type: {:?} - Company Tokens: {:?}",
    //         index,
    //         result.company_index,
    //         result.token_index,
    //         result.symbol,
    //         charcode_vector_to_token(result.token_vector),
    //         result.source_type,
    //         result.company_tokenized_entries

    //     );
    // }

    // let query = "Apple is not Walmart, but maybe Amazon.com is okay, REIT?";
    let query = "Ashford";
    // let query = "Power REIT";
    // let query = "REIT";
    // let query = "Arbor";
    // let query = "Arbor Realty";
    // let query = "Berkshire Hathaway is not Apple, but owns Apple, of course, which is not Apple Hospitality REIT.";
    // let query = "Berkshire Hathaway";
    // let query = "Apple";
    // let query = "Alphabet";
    // let query = r#"E-commerce giant Amazon.com Inc. (AMZN Quick QuoteAMZN - Free Report) joined the blue-chip index, Dow Jones Industrial Average, replacing drugstore operator Walgreens Boots Alliance (WBA Quick QuoteWBA - Free Report) on Feb 26. The reshuffle reflects the ongoing shift in economic power from traditional brick-and-mortar retail to e-commerce and technology-driven companies. The inclusion of Amazon in the Dow marks a significant milestone in the recognition of the e-commerce giant's influence and its role in the broader market.
    //     The shift was prompted by Walmart's (WMT Quick QuoteWMT - Free Report) decision to execute a 3-to-1 stock split, which has reduced its stock's weighting in the index. The Dow is a price-weighted index. So, stocks that fetch higher prices are given more weight. Amazon's addition has increased consumer retail exposure within the index, alongside enhancing the representation of various other business sectors that Amazon engages in, including cloud computing, digital streaming and artificial intelligence, among others (read: Walmart Soars on Earnings, Dividend & Vizio Deal: ETFs to Buy).
    //     Amazon took the 17th position in the index, while Walmart's weighting dropped to 26 from 17. UnitedHealth Group remained the most heavily weighted stock in the index. Amazon's entry into the Dow Jones is not just a symbolic change but a reflection of the evolving priorities and dynamics within the investment world. It signals a broader recognition of the value and impact of technology and e-commerce sectors, encouraging investors to perhaps rethink their investment approaches in light of these trends.

    //     While the Dow Jones is making new record highs, its performance is lagging behind the S&P and Nasdaq over the past year. The underperformance is due to the lack of exposure in tech stocks and the “Magnificent Seven” companies in particular. The Dow includes two of the Magnificent Seven — Apple (AAPL Quick QuoteAAPL - Free Report) and Microsoft (MSFT Quick QuoteMSFT - Free Report) . Amazon will be the third. As such, the addition of Amazon will help Dow Jones catch up with the S&P 500 gains. The shares of the commerce giant have surged more than 80% over the past year (read: ETFs to Tap on Amazon's Strong Q4 Earnings).

    //     Given this, investors seeking to tap the potential strength in the Dow Jones trend could consider SPDR Dow Jones Industrial Average ETF (DIA Quick QuoteDIA - Free Report) , iShares Dow Jones U.S. ETF (IYY Quick QuoteIYY - Free Report) , Invesco Dow Jones Industrial Average Dividend ETF (DJD Quick QuoteDJD - Free Report) and First Trust Dow 30 Equal Weight ETF (EDOW Quick QuoteEDOW - Free Report) .

    //     ETFs to Tap
    //     SPDR Dow Jones Industrial Average ETF (DIA Quick QuoteDIA - Free Report)
    //     SPDR Dow Jones Industrial Average ETF is one of the largest and most popular ETFs in the large-cap space, with AUM of $33.1 billion and an average daily volume of 3.8 million shares. It tracks the Dow Jones Industrial Average Index, holding 30 stocks in its basket with each making up for less than 9% share. Financials (21.7%), information technology (19.5%), healthcare (18.5%), consumer discretionary (15.9%) and industrials (14.613.7%) and are the top five sectors (read: Will Dow Jones ETFs Rule in 2024?).

    //     SPDR Dow Jones Industrial Average ETF charges 16 bps in annual fees and has a Zacks ETF Rank #1 (Strong Buy) with a Medium risk outlook.

    //     iShares Dow Jones U.S. ETF (IYY Quick QuoteIYY - Free Report)

    //     iShares Dow Jones U.S. ETF tracks the Dow Jones U.S. Index, holding 1077 stocks in its basket, with none accounting for more than 6.4% of the assets. Information technology takes the largest share at 29%, while financials, healthcare and consumer discretionary round off the next spots with double-digit exposure each.

    //     iShares Dow Jones U.S. ETF has amassed $1.9 billion in its asset base while trading in an average daily volume of 36,000 shares. It charges 20 bps in annual fees and has a Zacks ETF Rank #3 (Hold) with a Medium risk outlook.

    //     Invesco Dow Jones Industrial Average Dividend ETF (DJD Quick QuoteDJD - Free Report)

    //     Invesco Dow Jones Industrial Average Dividend ETF offers exposure to dividend-paying companies included in the Dow Jones Industrial Average by their 12-month dividend yield over the prior 12 months. It holds 27 stocks in its basket, with none accounting for more than 12% of the assets.

    //     Invesco Dow Jones Industrial Average Dividend ETF has been able to manage assets worth $294.4 million while trading in a volume of 56,000 shares a day on average. It charges 7 bps in annual fees and has a Zacks ETF Rank #3.

    //     First Trust Dow 30 Equal Weight ETF (EDOW Quick QuoteEDOW - Free Report)

    //     First Trust Dow 30 Equal Weight ETF offers equal-weight exposure to all the 30 components of the Dow Jones Industrial Average by tracking the Dow Jones Industrial Average Equal Weight Index.

    //     First Trust Dow 30 Equal Weight ETF has accumulated $249.1 million in its asset base and trades in an average daily volume of 58,000 shares. It charges 50 bps in annual fees.

    //     Want key ETF info delivered straight to your inbox?
    // Zacks’ free Fund Newsletter will brief you on top news and analysis, as well as top-performing ETFs, each week.
    //     "#;

    println!("Query: {}", query);

    let mut ticker_extractor = TickerExtractor::new(&company_symbols_list, weights);

    // TODO: Handle results
    ticker_extractor.extract(&query);

    let results = vec![];
    let total_score = 0.0;
    let company_rankings = vec![];

    (results, total_score, company_rankings)
}

// pub fn extract_tickers_from_text_with_custom_weights(
//     text: &str,
//     symbols_map: &SymbolsMap,
//     weights: Weights,
//     result_bias_adjuster: &ResultBiasAdjuster,
// ) -> (Vec<TickerSymbol>, f32, Vec<CompanyNameTokenRanking>) {
//     let mut matches = HashSet::new();

//     // Extract tickers by company name
//     let (company_name_matches, total_score, tokenized_filter, company_rankings) =
//         extract_tickers_from_company_names(text, symbols_map, weights, result_bias_adjuster);
//     let company_name_match_count = company_name_matches.len();

//     matches.extend(company_name_matches);

//     let filtered_text: String = text
//         .split_whitespace()
//         .filter(|word| !tokenized_filter.contains(&word.to_string()))
//         .collect::<Vec<&str>>()
//         .join(" ");

//     // Extract tickers by symbol
//     let mut symbol_matches = extract_tickers_from_symbols(&filtered_text, symbols_map);
//     let symbol_match_count = symbol_matches.len();

//     // Calculate the ratio of symbol matches to company name matches
//     let match_ratio = if company_name_match_count > 0 {
//         symbol_match_count as f32 / company_name_match_count as f32
//     } else {
//         f32::MAX
//     };

//     // eprintln!(
//     //     "Symbol match ratio: {:4} {:4}, Symbol match count: {}, Company name match count: {}",
//     //     match_ratio, weights.stop_word_filter_ratio, symbol_match_count, company_name_match_count
//     // );

//     // Decide whether to prune symbol matches based on the ratio and weight
//     if match_ratio < weights.stop_word_filter_ratio {
//         symbol_matches.retain(|symbol| {
//             if STOP_WORDS.contains(&symbol.to_lowercase().as_str()) {
//                 false // Remove stop words entirely
//             } else {
//                 true // Keep non-stop words
//             }
//         });
//     }

//     matches.extend(symbol_matches.clone());

//     let abbreviation_matches =
//         extract_tickers_from_abbreviations(&filtered_text, symbols_map, weights);
//     matches.extend(abbreviation_matches);

//     // Convert HashSet to Vec and return sorted for consistency
//     let mut results: Vec<TickerSymbol> = matches.into_iter().collect();
//     results.sort();

//     (results, total_score, company_rankings)
// }

// TODO: Remove `generate_alternative_symbols` and match on pure alphanumeric
// fn extract_tickers_from_symbols(text: &str, symbols_map: &SymbolsMap) -> Vec<TickerSymbol> {
//     let mut matches = HashSet::new();
//     let tokens = tokenize(text);

//     for token in tokens {
//         // Only match on tokens that are fully upper-case
//         if token == token.to_uppercase() {
//             let normalized = token.to_string();

//             // Check if the normalized token directly matches any symbol
//             if symbols_map.contains_key(&normalized) {
//                 matches.insert(normalized.clone());
//             } else {
//                 // Generate alternative symbols and check matches
//                 let alternatives = generate_alternative_symbols(&normalized);
//                 for alt in alternatives {
//                     if symbols_map.contains_key(&alt) {
//                         matches.insert(alt);
//                         break; // Stop checking alternatives once matched
//                     }
//                 }
//             }
//         }
//     }

//     matches.into_iter().collect()
// }

// TODO: Replace w/ cosine similarity
// fn extract_tickers_from_abbreviations(
//     text: &str,
//     symbols_map: &SymbolsMap,
//     weights: Weights,
// ) -> Vec<TickerSymbol> {
//     let mut matches = HashSet::new();

//     let input_tokens_capitalized: Vec<&str> = tokenize_company_name_query(text);

//     for token in input_tokens_capitalized {
//         // Normalize the token to lowercase
//         let lc_token = token.to_lowercase();

//         let token_length = token.len();

//         for (symbol, _company_name) in symbols_map {
//             let symbol_length = symbol.len();

//             // let lc_company_name = company_name.to_lowercase();
//             let lc_symbol = symbol.to_lowercase();

//             // Check if the token starts with part of the company name
//             if lc_token.starts_with(&lc_symbol) {
//                 let abbr_perc = symbol_length as f32 / token_length as f32;

//                 if abbr_perc > weights.abbreviation_match_threshold {
//                     matches.insert(symbol.to_string());
//                 }
//             }
//         }
//     }

//     matches.into_iter().collect()
// }

// TODO: Replace w/ cosine similarity
// fn extract_tickers_from_company_names(
//     text: &str,
//     symbols_map: &SymbolsMap,
//     weights: Weights,
//     result_bias_adjuster: &ResultBiasAdjuster,
// ) -> (
//     Vec<TickerSymbol>,
//     f32,
//     HashSet<String>,
//     Vec<CompanyNameTokenRanking>,
// ) {
//     let mut scored_results: HashMap<TickerSymbol, f32> = HashMap::new();

//     // Note: This is not a vector of symbols; maybe explicit type defining could make this more apparent
//     let mut tokenized_filter: HashSet<String> = HashSet::new();

//     let input_tokens_capitalized: Vec<&str> = tokenize_company_name_query(text);
//     let mut company_rankings: Vec<CompanyNameTokenRanking> = Vec::new();

//     let mut input_token_index_to_top_company_ranking_map: HashMap<
//         usize,
//         Vec<CompanyNameTokenRanking>,
//     > = HashMap::new();

//     if !input_tokens_capitalized.is_empty() {
//         // Filter input tokens: Only consider tokens starting with a capital letter and of sufficient length, then remove stop words

//         for (symbol, company_name) in symbols_map {
//             // Skip entries without a valid company name
//             if let Some(company_name) = company_name {
//                 if company_name.is_empty() {
//                     continue;
//                 }

//                 // TODO: This could run through "tokenize" as well if it doesn't incur a perf penalty for some reason
//                 //
//                 // Normalize, filter stop words, and tokenize the company name
//                 let company_tokens: Vec<String> = company_name
//                     .to_lowercase()
//                     .replace(|c: char| !c.is_alphanumeric() && c != ' ', " ")
//                     .split_whitespace()
//                     .filter(|word| !STOP_WORDS.contains(word))
//                     .map(String::from)
//                     .collect();

//                 if company_tokens.is_empty() {
//                     continue;
//                 }

//                 let total_company_words = company_tokens.len();
//                 let mut consecutive_match_count = 0;
//                 let mut top_consecutive_match_count = 0;

//                 let mut match_score = 0.0;

//                 // Single pass through input tokens
//                 let mut company_index = 0;

//                 // let mut seen_tokens: HashSet<String> = HashSet::new();

//                 // let mut input_token_indices: Vec<usize> = Vec::new();
//                 let mut company_index_token_index_map: HashMap<usize, usize> = HashMap::new();
//                 let mut top_company_index_token_index_map: HashMap<usize, usize> = HashMap::new();

//                 for (input_token_position, input_token) in
//                     input_tokens_capitalized.iter().enumerate()
//                 {
//                     let lc_input_token = input_token.to_lowercase();

//                     if &lc_input_token != &company_tokens[company_index] {
//                         // Note: This reset is perfomrmed before the following `if` statement to fix an issue
//                         // where a phrase with `Apple Apple Hopitality REIT` are identified as separate companies.
//                         // Previously, the consecutive match mechanism would get out of sync and identify
//                         // `Apple Hospitality REIT` with a low score.
//                         consecutive_match_count = 0;

//                         company_index = 0;

//                         company_index_token_index_map.remove(&company_index);
//                     }

//                     if &lc_input_token == &company_tokens[company_index] {
//                         // input_token_indices.push(input_token_position);
//                         company_index_token_index_map.insert(company_index, input_token_position);

//                         // Match found, increment the company pointer
//                         consecutive_match_count += 1;

//                         company_index += 1;

//                         if consecutive_match_count > top_consecutive_match_count {
//                             top_consecutive_match_count = consecutive_match_count;

//                             top_company_index_token_index_map =
//                                 company_index_token_index_map.clone();
//                         }

//                         // If we've matched the entire company_tokens, score it
//                         if company_index == total_company_words {
//                             // match_score += consecutive_score;

//                             // Reset for further potential matches in input tokens
//                             consecutive_match_count = 0;
//                             company_index = 0;
//                         }
//                     }
//                 }

//                 let mut consecutive_jaccard_similarity: f32 = 0.0;

//                 let mut result_bias_adjuster_score: f32 = 0.0;

//                 // TODO: Ideally, this should be set inide the following block,
//                 // but I'm fighting with the borrower-checker to try to retain this value
//                 let lc_norm_input_string: String = top_company_index_token_index_map
//                     .values()
//                     .map(|&index| input_tokens_capitalized[index])
//                     .collect::<Vec<&str>>()
//                     .join(" ")
//                     .to_lowercase();

//                 if top_consecutive_match_count > 0 {
//                     match_score +=
//                         top_consecutive_match_count as f32 * weights.consecutive_match_weight;

//                     let lc_norm_company_string: String = company_tokens.join(" ");

//                     consecutive_jaccard_similarity =
//                         jaccard_similarity_chars(&lc_norm_input_string, &lc_norm_company_string);

//                     match_score +=
//                         consecutive_jaccard_similarity * (1.0 - weights.letter_mismatch_penalty);

//                     match_score += (top_consecutive_match_count as f32
//                         / total_company_words as f32)
//                         * (1.0 - weights.word_mismatch_penalty);

//                     // TODO: Apply configurable weighting?
//                     result_bias_adjuster_score =
//                         result_bias_adjuster.score(&lc_norm_input_string, &company_tokens);

//                     // Scale match_score by result_bias_adjuster_score
//                     match_score *= result_bias_adjuster_score * (1.0 / DEFAULT_BIAS_ADJUSTER_SCORE);
//                 }

//                 if lc_norm_input_string.len() > 0 {
//                     // if match_score > weights.minimum_match_score {
//                     let company_ranking: CompanyNameTokenRanking = CompanyNameTokenRanking {
//                         ticker_symbol: symbol.to_string(),
//                         company_name: company_name.to_string(),
//                         input_token_indices: top_company_index_token_index_map
//                             .values()
//                             .cloned()
//                             .collect(),
//                         consecutive_match_count: top_consecutive_match_count,
//                         consecutive_jaccard_similarity,
//                         match_score,
//                         result_bias_adjuster_score,
//                         context_query_string: lc_norm_input_string,
//                         context_company_tokens: company_tokens,
//                     };

//                     company_rankings.push(company_ranking);

//                     eprintln!(
//                         "Company name: {}, Context attention score: {}",
//                         company_name, result_bias_adjuster_score
//                     );
//                 }

//                 // } else if match_score > 0.0 {
//                 //     eprintln!(
//                 //         "Discarded symbol: {}; Match Score: {:.4}, Consecutive Matches: {}, Jaccard: {}",
//                 //         symbol, match_score, top_consecutive_match_count, consecutive_jaccard_similarity
//                 //     );
//                 // }
//             }
//         }
//     }

//     for company_ranking in &company_rankings {
//         if company_ranking.match_score > 0.0 {
//             eprintln!(
//                 "Company name: {}; Match Score: {}; Input Token Positions: {:?}; Jaccard: {}",
//                 company_ranking.company_name,
//                 company_ranking.match_score,
//                 company_ranking.input_token_indices,
//                 company_ranking.consecutive_jaccard_similarity
//             );

//             for input_token_index in company_ranking.input_token_indices.iter() {
//                 // Check if this token index already has an entry
//                 if let Some(existing_rankings) =
//                     input_token_index_to_top_company_ranking_map.get_mut(input_token_index)
//                 {
//                     // Find the highest score in the current list
//                     let max_score = existing_rankings
//                         .iter()
//                         .map(|ranking| ranking.match_score)
//                         .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
//                         .unwrap_or(0.0);

//                     if company_ranking.match_score > max_score {
//                         // New higher score, replace the existing vector
//                         *existing_rankings = vec![company_ranking.clone()];
//                     } else if (company_ranking.match_score - max_score).abs() < f32::EPSILON {
//                         // Scores are equal, append the new ranking
//                         existing_rankings.push(company_ranking.clone());
//                     }
//                 } else {
//                     // No entry exists, insert this company ranking as a new vector
//                     input_token_index_to_top_company_ranking_map
//                         .insert(*input_token_index, vec![company_ranking.clone()]);
//                 }
//             }
//         }
//     }

//     for (_, company_rankings) in input_token_index_to_top_company_ranking_map {
//         for company_ranking in company_rankings {
//             // Tokenize the company name and add tokens to the filter
//             let tokenized_company_name = tokenize(&company_ranking.company_name);
//             for word in tokenized_company_name {
//                 tokenized_filter.insert(word.to_string());
//             }

//             // Update the scored_results with the match score
//             scored_results
//                 .entry(company_ranking.ticker_symbol.to_string())
//                 .and_modify(|e| *e += company_ranking.match_score)
//                 .or_insert(company_ranking.match_score);
//         }
//     }

//     let mut result_ticker_symbols = Vec::new();
//     let mut total_score = 0.0;

//     for (symbol, score) in scored_results.clone() {
//         // Print the result
//         eprintln!(
//             "Matched Symbol: {}, Score: {:.4}, Company Name: {:?}",
//             symbol, score, symbols_map[&symbol]
//         );

//         // Update the result keys and total score
//         result_ticker_symbols.push(symbol);
//         total_score += score;
//     }

//     eprintln!("Total score: {:.2}", total_score);

//     (
//         result_ticker_symbols,
//         total_score,
//         tokenized_filter,
//         company_rankings,
//     )
// }
