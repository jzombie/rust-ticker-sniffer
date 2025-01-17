#![allow(dead_code, unused_imports, unused_variables)]

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use ticker_sniffer::{
    extract_tickers_from_text_with_custom_config, DEFAULT_COMPANY_TOKEN_PROCESSOR_CONFIG,
};

fn benchmark_extract_tickers_short(c: &mut Criterion) {
    // Example text for benchmarking (shorter text)
    let text = "AAPL is performing well, but MSFT is also a strong contender. \
                Amazon is another company making waves in the market.";

    c.bench_function("extract_tickers_short", |b| {
        b.iter(|| {
            extract_tickers_from_text_with_custom_config(
                black_box(DEFAULT_COMPANY_TOKEN_PROCESSOR_CONFIG),
                black_box(text),
            )
            .expect("Ticker extraction failed");
        })
    });
}

fn benchmark_extract_tickers_long(c: &mut Criterion) {
    // Provided long example text for benchmarking
    let text = "E-commerce giant Amazon.com Inc. (AMZN Quick QuoteAMZN - Free Report) joined \
        the blue-chip index, Dow Jones Industrial Average, replacing drugstore operator \
        Walgreens Boots Alliance (WBA Quick QuoteWBA - Free Report) on Feb 26. The reshuffle \
        reflects the ongoing shift in economic power from traditional brick-and-mortar retail \
        to e-commerce and technology-driven companies. The inclusion of Amazon in the Dow marks \
        a significant milestone in the recognition of the e-commerce giant's influence and its role \
        in the broader market. The shift was prompted by Walmart's (WMT Quick QuoteWMT - Free Report) \
        decision to execute a 3-to-1 stock split, which has reduced its stock's weighting in the index. \
        The Dow is a price-weighted index. So, stocks that fetch higher prices are given more weight. \
        Amazon's addition has increased consumer retail exposure within the index, alongside enhancing \
        the representation of various other business sectors that Amazon engages in, including cloud \
        computing, digital streaming and artificial intelligence, among others. Amazon took the 17th \
        position in the index, while Walmart's weighting dropped to 26 from 17. UnitedHealth Group \
        remained the most heavily weighted stock in the index. Amazon's entry into the Dow Jones is \
        not just a symbolic change but a reflection of the evolving priorities and dynamics within the \
        investment world. It signals a broader recognition of the value and impact of technology and \
        e-commerce sectors, encouraging investors to perhaps rethink their investment approaches in \
        light of these trends.";

    c.bench_function("extract_tickers_long", |b| {
        b.iter(|| {
            extract_tickers_from_text_with_custom_config(
                black_box(DEFAULT_COMPANY_TOKEN_PROCESSOR_CONFIG),
                black_box(text),
            )
            .expect("Ticker extraction failed");
        })
    });
}

criterion_group!(
    benches,
    benchmark_extract_tickers_short,
    benchmark_extract_tickers_long
);
criterion_main!(benches);
