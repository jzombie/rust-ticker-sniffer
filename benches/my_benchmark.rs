#![allow(dead_code, unused_imports, unused_variables)]

#[path = "../test_utils/lib.rs"]
mod test_utils;
use test_utils::constants::TEST_SYMBOLS_CSV_PATH;
use test_utils::load_company_symbol_list_from_file;

use criterion::{black_box, criterion_group, criterion_main, Criterion};

use ticker_sniffer::{extract_tickers_from_text, ResultBiasAdjuster};

fn benchmark_extract_tickers(c: &mut Criterion) {
    let symbols_map =
        load_company_symbol_list_from_file(TEST_SYMBOLS_CSV_PATH).expect("Failed to load symbols from CSV");

    let text = "AAPL is performing well, but MSFT is also a strong contender. There are also Amazon is another company.";

    c.bench_function("extract_tickers", |b| {
        b.iter(|| extract_tickers_from_text(black_box(text), black_box(&symbols_map)))
    });
}

criterion_group!(benches, benchmark_extract_tickers);
criterion_main!(benches);
