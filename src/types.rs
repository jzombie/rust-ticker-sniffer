pub type TickerSymbol = String;
pub type CompanyName = String;

pub type CompanySymbolsList = Vec<(TickerSymbol, Option<CompanyName>)>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)] // Use a numeric representation for efficiency
pub enum TokenSourceType {
    Symbol = 0,      // Tokens derived from the symbol
    CompanyName = 1, // Tokens derived from the company name
}
