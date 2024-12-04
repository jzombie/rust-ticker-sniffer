use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::collections::HashMap;
use ticker_sniffer::extract_tickers_from_text;

fn benchmark_extract_tickers(c: &mut Criterion) {
    let mut symbols_map = HashMap::new();
    symbols_map.insert("AAPL".to_string(), Some("Apple Inc.".to_string()));
    symbols_map.insert(
        "MSFT".to_string(),
        Some("Microsoft Corporation".to_string()),
    );

    let text = "AAPL is performing well, but MSFT is also a strong contender.";

    c.bench_function("extract_tickers", |b| {
        b.iter(|| extract_tickers_from_text(black_box(text), black_box(&symbols_map)))
    });
}

criterion_group!(benches, benchmark_extract_tickers);
criterion_main!(benches);
