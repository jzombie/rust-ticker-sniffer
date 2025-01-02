// These types may be shared throughout files. Types which are used in single files are typically defined in those files.

pub type TokenId = usize;
pub type TickerSymbol = String;
pub type CompanyName = String;
pub type AlternateCompanyName = String;

pub type CompanySymbolList = Vec<(TickerSymbol, Option<CompanyName>, Vec<AlternateCompanyName>)>;

pub type TokenizerVectorToken = Vec<u32>;
