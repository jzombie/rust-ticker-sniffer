use csv::ReaderBuilder;
use std::error::Error;
use std::io::Cursor;
use ticker_sniffer_common_lib::types::CompanySymbolList;

pub fn read_company_symbol_list_from_string(
    csv: &str,
) -> Result<CompanySymbolList, Box<dyn Error>> {
    let mut company_symbols_list = CompanySymbolList::new();

    // Use a cursor to simulate a file reader from the string
    let mut reader = ReaderBuilder::new()
        .has_headers(true) // Ensure headers are expected
        .from_reader(Cursor::new(csv));

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
