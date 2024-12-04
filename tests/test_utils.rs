use csv::Reader;
use std::collections::HashMap;
use std::error::Error;

/// Utility to load symbols from a CSV file for testing and benchmarking.
pub fn load_symbols_from_file(
    file_path: &str,
) -> Result<HashMap<String, Option<String>>, Box<dyn Error>> {
    let mut symbols_map = HashMap::new();
    let mut reader = Reader::from_path(file_path)?;

    for record in reader.records() {
        let record = record?;
        if record.len() == 2 {
            let symbol = record.get(0).unwrap().to_uppercase();
            let company_name = record.get(1).map(|name| name.to_string());
            symbols_map.insert(symbol, company_name);
        } else {
            eprintln!("Skipping invalid row: {:?}", record);
        }
    }

    Ok(symbols_map)
}
