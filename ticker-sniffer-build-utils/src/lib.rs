use csv::Reader;
use serde::Serialize;
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::Write;

mod utils;
use utils::generate_hashmap_code;

// use bincode;

type TickerSymbol = String;
type CompanyName = String;
type AlternateCompanyName = String;

// CompanySymbolList is a type alias for a vector of tuples,
// and all components already implement Serialize and Deserialize
type CompanySymbolList = Vec<(TickerSymbol, Option<CompanyName>, Vec<AlternateCompanyName>)>;

#[derive(Debug, Serialize)]
pub struct Temp {
    pub a: String,
    pub b: i32,
    pub c: Vec<i32>,
}

pub fn run_build_utils() -> Result<(), Box<dyn std::error::Error>> {
    // Load the company symbols list from a CSV file
    let company_symbols_list =
        load_company_symbol_list_from_file("../data/company_symbol_list.csv")?;

    // Serialize the vector into binary format
    // let encoded: Vec<u8> = bincode::serialize(&company_symbols_list)?;

    // let generated_code = format!(
    //     r#"
    //     /// Auto-generated data from build.rs
    //     pub const COMPANY_SYMBOLS: &[CustomData] = &{:?};
    //             "#,
    //     company_symbols_list
    // );

    // let mut my_map = HashMap::new();
    // my_map.insert("AAPL", 1);
    // my_map.insert("MSFT", 2);

    let mut inner_map = HashMap::new();
    inner_map.insert("CEO", vec![vec![vec![1, 2, 3]]]);
    inner_map.insert("Founded", vec![vec![vec![4, 5, 6]]]);

    let mut my_map = HashMap::new();
    my_map.insert("AAPL", inner_map);

    let generated_code = generate_hashmap_code("ticker_map", &my_map);

    // Write the serialized data to a binary file
    let mut file = File::create("src/__dummy_generated__.bin")?;
    file.write_all(&generated_code.as_bytes())?;

    Ok(())
}

/// Utility to load symbols from a CSV file for testing and benchmarking.
fn load_company_symbol_list_from_file(
    file_path: &str,
) -> Result<CompanySymbolList, Box<dyn Error>> {
    let mut company_symbols_list = CompanySymbolList::new();
    let mut reader = Reader::from_path(file_path)?;

    // Extract column headers
    let headers = reader.headers()?.clone();

    for record in reader.records() {
        let record = record?;
        // Extract values based on header names
        let symbol = record.get(headers.iter().position(|h| h == "Symbol").unwrap());
        let company_name = record.get(headers.iter().position(|h| h == "Company Name").unwrap());
        let comma_separated_alternate_names =
            record.get(headers.iter().position(|h| h == "Alternate Names").unwrap());

        let alternate_names: Vec<String> = if let Some(names) = comma_separated_alternate_names {
            names
                .split(',')
                .map(|name| name.trim().to_string()) // Trim whitespace and convert to String
                .collect()
        } else {
            Vec::new() // Default to an empty vector if alternate names are missing
        };

        if let Some(symbol) = symbol {
            company_symbols_list.push((
                symbol.to_uppercase(),
                company_name.map(|name| name.to_string()),
                alternate_names,
            ));
        } else {
            eprintln!("Skipping invalid row: {:?}", record);
        }
    }

    Ok(company_symbols_list)
}
