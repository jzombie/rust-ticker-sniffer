use std::collections::HashMap;

// Types listed here are either shared across multiple files and/or exposed via the library.

/// Represents a token as an owned `String`. Tokens are the basic units used for processing text.
pub type Token = String;

/// Represents a borrowed view of a token as a `str`. This is used when ownership is not required.
pub type TokenRef = str;

// TODO: This might could be a u16 for this use case
//
/// A vector of token IDs, represented as `u32`. This type is used to store sequences of token IDs
/// that map to specific tokens in a `TokenMapper`.
pub type TokenVector = Vec<u32>;

/// A unique identifier for a token, represented as a `usize`. This is typically used to index
/// tokens in a data structure such as a `HashMap` or `Vec`.
pub type TokenId = usize;

/// Represents the name of a company as an owned `String`.
pub type CompanyName = String;

/// Represents an alternate name for a company as an owned `String`. These are used to match
/// variations in naming conventions or aliases for companies.
pub type AlternateCompanyName = String;

/// A list of company symbols, where each entry includes:
/// - `TickerSymbol`: The company's stock ticker.
/// - `Option<CompanyName>`: The company's primary name (optional if not available).
/// - `Vec<AlternateCompanyName>`: A list of alternate names or aliases for the company.
pub type CompanySymbolList = Vec<(TickerSymbol, Option<CompanyName>, Vec<AlternateCompanyName>)>;

/// Represents a ticker symbol (e.g., stock ticker) as an owned `String`.
pub type TickerSymbol = String;

/// Represents the total number occurrences of a ticker symbol within a text document.
pub type TickerSymbolFrequency = usize;

/// Represents a map of ticker symbols to their frequency counts within a text document.
/// The key is the `TickerSymbol`, and the value is the `TickerSymbolFrequency`.
pub type TickerSymbolFrequencyMap = HashMap<TickerSymbol, TickerSymbolFrequency>;

// TODO: Document
pub type QueryTokenIndex = usize;
pub type CompanySequenceIndex = usize;
pub type CompanySequenceTokenIndex = usize;
pub type TickerSymbolTokenId = TokenId;
