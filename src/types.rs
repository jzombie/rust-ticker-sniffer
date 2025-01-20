use std::collections::HashMap;

// Types listed here are either shared across multiple files and/or exposed via the library.

/// Represents a token as an owned `String`. Tokens are the basic units used for processing text.
pub type Token = String;

/// Represents a borrowed view of a token as a `str`. This is used when ownership is not required.
pub type TokenRef = str;

/// Represents the numeric Unicode scalar value (`char`) of a token's character.
///
/// Each character in a token is converted to its corresponding Unicode scalar
/// value (`char`), which is stored as a `u32`. This type is used for better
/// semantic clarity in representing character codes of tokens.
pub type TokenCharCode = u32;

/// A vector of token character codes, represented as `u32` values.
pub type TokenVector = Vec<TokenCharCode>;

/// A unique identifier for a token, represented as a `usize`. This identifier is
/// used for a static mapping between a token (e.g., a word or symbol) and its
/// associated data in structures like `HashMap` or `Vec`.
///
/// Unlike a query token index, which refers to a position-based index during
/// runtime queries, a `TokenId` is a persistent, position-independent value
/// that serves as a reference to a specific token. It does not imply or rely
/// on index positioning within any particular data structure.
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

/// Represents a word number in a text document after non-sequence words have been filtered out.
pub type QueryTokenIndex = usize;

// Represents a sequence number of a company name or alias.
pub type CompanySequenceIndex = usize;

// Represents a word number in a company name or alias.
pub type CompanySequenceTokenIndex = usize;

// Represents the token ID of a ticker symbol.
pub type TickerSymbolTokenId = TokenId;
