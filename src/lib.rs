use std::collections::{HashMap, HashSet};

/// Represents a single extracted ticker result.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TickerMatch {
    pub symbol: String,
    pub company_name: Option<String>,
}

/// Extracts ticker symbols and company names from a text document.
///
/// # Arguments
/// - `text`: The text document to analyze.
/// - `symbols_map`: A map of symbols to company names.
///
/// # Returns
/// A vector of `TickerMatch` structs representing the matches found.
pub fn extract_tickers_from_text(
    text: &str,
    symbols_map: &HashMap<String, Option<String>>,
) -> Vec<TickerMatch> {
    let mut matches = HashSet::new();
    let tokens: Vec<&str> = text.split_whitespace().collect();

    for token in tokens {
        let normalized = token
            .trim_matches(|c: char| !c.is_alphanumeric())
            .to_lowercase();

        if let Some(company_name) = symbols_map.get(&normalized) {
            matches.insert(TickerMatch {
                symbol: normalized.clone(),
                company_name: company_name.clone(),
            });
        }
    }

    matches.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_tickers() {
        let mut symbols_map = HashMap::new();
        symbols_map.insert("aapl".to_string(), Some("Apple Inc.".to_string()));
        symbols_map.insert(
            "msft".to_string(),
            Some("Microsoft Corporation".to_string()),
        );

        let text = "AAPL is performing well, but MSFT is also a strong contender.";

        let results = extract_tickers_from_text(text, &symbols_map);

        assert_eq!(results.len(), 2);
        assert!(results.contains(&TickerMatch {
            symbol: "aapl".to_string(),
            company_name: Some("Apple Inc.".to_string()),
        }));
        assert!(results.contains(&TickerMatch {
            symbol: "msft".to_string(),
            company_name: Some("Microsoft Corporation".to_string()),
        }));
    }
}
