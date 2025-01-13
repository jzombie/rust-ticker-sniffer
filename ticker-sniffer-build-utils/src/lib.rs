use csv::Reader;
use serde::Serialize;
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::Write;
use ticker_sniffer_common_lib::constants::CODE_AUTOGEN_PREFIX;
use ticker_sniffer_common_lib::models::CompanyTokenMapper;
use ticker_sniffer_common_lib::types::CompanySymbolList;

mod utils;
use utils::generate_hashmap_code;

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

    let company_token_mapper = CompanyTokenMapper::new(&company_symbols_list);

    let ticker_symbol_map = company_token_mapper.ticker_symbol_map;
    serialize_hashmap_to_file(
        &ticker_symbol_map,
        "COMPANY_TOKEN_MAPPER__TICKER_SYMBOL_MAP",
    )?;

    let reverse_ticker_symbol_map = company_token_mapper.reverse_ticker_symbol_map;
    serialize_hashmap_to_file(
        &reverse_ticker_symbol_map,
        "COMPANY_TOKEN_MAPPER__REVERSE_TICKER_SYMBOL_MAP",
    )?;

    let company_token_sequences_map = company_token_mapper.company_token_sequences_map;
    serialize_hashmap_to_file(
        &company_token_sequences_map,
        "COMPANY_TOKEN_MAPPER__COMPANY_TOKEN_SEQUENCES_MAP",
    )?;

    // TODO: This should use token IDs instead of tokens
    let company_reverse_token_map = company_token_mapper.company_reverse_token_map;
    serialize_hashmap_to_file(
        &company_reverse_token_map,
        "COMPANY_TOKEN_MAPPER__COMPANY_REVERSE_TOKEN_MAP",
    )?;

    let token_map = company_token_mapper.token_mapper.token_map;
    serialize_hashmap_to_file(&token_map, "TOKEN_MAPPER__TOKEN_MAP")?;

    let reverse_token_map = company_token_mapper.token_mapper.reverse_token_map;
    serialize_hashmap_to_file(&reverse_token_map, "TOKEN_MAPPER__REVERSE_TOKEN_MAP")?;

    Ok(())
}

fn serialize_hashmap_to_file<K, V>(hashmap: &HashMap<K, V>, map_name: &str) -> std::io::Result<()>
where
    K: Serialize,
    V: Serialize,
{
    let generated_code = generate_hashmap_code(map_name, hashmap);
    let file_path = format!("src/{}{}.rs", CODE_AUTOGEN_PREFIX, map_name);

    let mut file =
        File::create(&file_path).expect(&format!("Could not create file at path: {}", file_path));

    file.write_all(&generated_code.as_bytes())
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
