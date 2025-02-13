# Ticker Sniffer (Work in Progress)

[![made-with-rust][rust-logo]][rust-src-page]
[![crates.io][crates-badge]][crates-page]
[![Documentation][docs-badge]][docs-page]
[![MIT licensed][license-badge]][license-page]
[![CI Pipeline][ci-badge]][ci-page]

`Ticker Sniffer` is a Rust crate for extracting U.S. stock market ticker symbols from text. It analyzes content, identifies ticker references, and calculates their frequency, returning the results as a `HashMap`.

Use cases include extracting tickers from news articles and search queries.

Parsing is performed using a [self-contained CSV file](data) embedded in the binary. No external CSV or file-reading dependencies are required in the final build, and it is fully compatible with WASM.

## Install

```bash
cargo add ticker-sniffer
```

## Code Example

```rust
use ticker_sniffer::extract_tickers_from_text;
use ticker_sniffer::types::TickerSymbolFrequencyMap;

fn main() {
    let text = "E-commerce giant Amazon.com Inc. joined \
        the blue-chip index, Dow Jones Industrial Average, replacing drugstore operator \
        Walgreens Boots Alliance on Feb 26. The reshuffle \
        reflects the ongoing shift in economic power from traditional brick-and-mortar retail \
        to e-commerce and technology-driven companies. The inclusion of Amazon in the Dow marks \
        a significant milestone in the recognition of the e-commerce giant's influence and its role \
        in the broader market. The shift was prompted by Walmart's  \
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

    // Setting this to false will increase false positives between nouns 
    // (e.g., "apple") and company names (e.g., "Apple"), but might be useful 
    // for certain use cases.
    let is_case_sensitive_doc_parsing = true;

    match extract_tickers_from_text(text, is_case_sensitive_doc_parsing) {
        Ok(results) => {
            assert_eq!(
                results,
                TickerSymbolFrequencyMap::from([
                    ("AMZN".to_string(), 2),
                    ("WMT".to_string(), 1),
                    ("DIA".to_string(), 1),
                    ("WBA".to_string(), 1),
                    ("UNH".to_string(), 1),
                ])
            );

            for (ticker, count) in &results {
                println!("{}: {}", ticker, count);
            }
        }
        Err(e) => eprintln!("Error extracting tickers: {}", e),
    }
}
```

## How it Works

The text search engine employs a hybrid approach to identify company names and stock symbols in documents.

Initially, it filters out stop words and applies a sequence-based tokenizer to detect potential company names, preserving word order for contextual accuracy.

Simultaneously, a secondary tokenizer uses a Bag of Words approach to identify stock symbols, which may occasionally collide with stop words.

The engine calculates a ratio by comparing the number of company name matches to exact stock symbol matches found in the document.

Based on this ratio, it determines whether to include exact stock symbol matches in the results.

Regardless of the decision, the engine ensures that stock symbols are always matched, but the contextual importance of symbols is weighted by their relationship to identified company names.



## Running Tests with Output Capturing

When running tests, you can use the `--nocapture` flag to display output from tests in the console. This is particularly useful for this package as there are tests which process several files at once.

### Running All Tests

```bash
cargo test -- --nocapture
```

### Running Specific Tests

For example, to run the `tokenizer_tests` module in isolation with visible output:

```bash
cargo test --test tokenizer_tests  -- --nocapture
```

## Benching

```bash
cargo bench
```


## Prototype Debug

```bash
RUST_LOG=debug cargo dev
```

Note: `dev` is an aliased Cargo command, as specified in the [.cargo/config.toml](.cargo/config.toml) file.

More information about Cargo aliases can be found at: https://doc.rust-lang.org/cargo/reference/config.html#configuration-format.

## Lint

If clippy is not already installed:

```bash
rustup component add clippy
```

```bash
cargo clippy --fix
```

Suggestions:

```bash
cargo clippy -- -W clippy::all
```

## Building CLI tool

### Without Logging Support

```bash
cargo build --release --bin ticker-sniffer-cli
```

### With Logging Support

```bash
cargo build --release --bin ticker-sniffer-cli --features="logger-support"
```

## Running CLI tool (on Unix)

With debugging enabled. Note, it has to be compiled with `logger-support` feature added.

```bash
echo "Amazon" | RUST_LOG=debug ./target/release/ticker-sniffer-cli
```

## Publishing Note

Currently, the build process does not use the `OUT_DIR` environment variable to generate temporary artifacts. Instead, temporary files are created directly within the repository. This approach ensures that a compressed form of the [company_symbol_list.csv](data/company_symbol_list.csv) file is bundled correctly with the build, though it is acknowledged that this solution could likely be improved.

### Known Issue During Publishing

When publishing the crate, you may encounter the following error:

```bash
error: 1 files in the working directory contain changes that were not yet committed into git:
```

### Workaround

To proceed with publishing, use the `--allow-dirty` flag:

```bash
cargo publish --allow-dirty
```

## ChatGPT-based Alternative Name Suggester

This app, powered by ChatGPT, assists in generating alternative names for companies. For more details, refer to [data/README.md](data/README.md).

https://chatgpt.com/g/g-675e2b64d02c8191ab4819b971aeeded-stock-company-alternative-name-suggester

## License

[MIT License](LICENSE) (c) 2025 Jeremy Harris.

[rust-src-page]: https://www.rust-lang.org/
[rust-logo]: https://img.shields.io/badge/Made%20with-Rust-black?&logo=Rust

[crates-page]: https://crates.io/crates/ticker-sniffer
[crates-badge]: https://img.shields.io/crates/v/ticker-sniffer.svg

[docs-page]: https://docs.rs/ticker-sniffer
[docs-badge]: https://docs.rs/ticker-sniffer/badge.svg

[license-page]: ./LICENSE
[license-badge]: https://img.shields.io/badge/license-MIT-blue.svg

[ci-page]: https://github.com/jzombie/rust-ticker-sniffer/actions/workflows/ci.yml
[ci-badge]: https://github.com/jzombie/rust-ticker-sniffer/actions/workflows/ci.yml/badge.svg
