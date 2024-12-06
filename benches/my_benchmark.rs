#[path = "../test_utils/lib.rs"]
mod test_utils;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use test_utils::load_symbols_from_file;
use ticker_sniffer::{extract_tickers_from_text, Weights};

fn benchmark_extract_tickers(c: &mut Criterion) {
    let symbols_map =
        load_symbols_from_file("tests/test_symbols.csv").expect("Failed to load symbols from CSV");

    let text = "AAPL is performing well, but MSFT is also a strong contender. There are also Amazon is another company.";

    const WEIGHTS: Weights = Weights {
        continuity: 0.025,
        mismatched_letter_penalty: 0.3,
        mismatched_word_penalty: 0.3,
        match_score_threshold: 0.25,
        // continuity: 0.3688305957567424,
        // coverage_input: 0.026040188967873246,
        // coverage_company: 0.5971237581795172,
        // match_score_threshold: 1.6376519441299855,
    };

    c.bench_function("extract_tickers", |b| {
        b.iter(|| extract_tickers_from_text(black_box(text), black_box(&symbols_map), WEIGHTS))
    });
}

criterion_group!(benches, benchmark_extract_tickers);
criterion_main!(benches);
