use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};

/// Utility to load symbols from a CSV file for testing and benchmarking.
pub fn load_symbols_from_file(file_path: &str) -> HashMap<String, Option<String>> {
    let mut symbols_map = HashMap::new();
    let file = File::open(file_path).expect("Failed to open test symbols file");
    let reader = BufReader::new(file);

    for line in reader.lines() {
        let line = line.expect("Failed to read line");
        let parts: Vec<&str> = line.splitn(2, ',').collect();
        if parts.len() == 2 {
            symbols_map.insert(parts[0].to_uppercase(), Some(parts[1].to_string()));
        } else {
            symbols_map.insert(parts[0].to_uppercase(), None);
        }
    }

    symbols_map
}
