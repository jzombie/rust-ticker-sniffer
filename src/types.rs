use std::collections::HashMap;

pub type TickerSymbol = String;
pub type CompanyName = String;

pub type CompanySymbolList = Vec<(TickerSymbol, Option<CompanyName>)>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)] // Use a numeric representation for efficiency
pub enum CompanyTokenSourceType {
    Symbol = 0,      // Tokens derived from the symbol
    CompanyName = 1, // Tokens derived from the company name
}

pub type TokenizerVectorToken = Vec<u32>;
