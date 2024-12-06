#[path = "../test_utils/lib.rs"]
mod test_utils;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use test_utils::load_symbols_from_file;
use ticker_sniffer::{extract_tickers_from_text, DEFAULT_WEIGHTS};

fn benchmark_extract_tickers(c: &mut Criterion) {
    let symbols_map =
        load_symbols_from_file("tests/test_symbols.csv").expect("Failed to load symbols from CSV");

    let text = "AAPL is performing well, but MSFT is also a strong contender. There are also Amazon is another company.";

    c.bench_function("extract_tickers", |b| {
        b.iter(|| {
            extract_tickers_from_text(black_box(text), black_box(&symbols_map), DEFAULT_WEIGHTS)
        })
    });
}

criterion_group!(benches, benchmark_extract_tickers);
criterion_main!(benches);
