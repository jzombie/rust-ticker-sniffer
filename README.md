# Ticker Sniffer

[![crates.io](https://img.shields.io/crates/v/ticker-sniffer.svg)](https://crates.io/crates/ticker-sniffer)
[![Documentation](https://docs.rs/ticker-sniffer/badge.svg)](https://docs.rs/ticker-sniffer)
[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE)
[![CI Pipeline](https://github.com/jzombie/rust-ticker-sniffer/actions/workflows/ci.yml/badge.svg)](https://github.com/jzombie/rust-ticker-sniffer/actions/workflows/ci.yml)

# Ticker Sniffer (Work in Progress)

`Ticker Sniffer` is a Rust crate built to parse and extract ticker symbols from text documents. It specializes in analyzing text content to identify references to U.S. stock market ticker symbols and calculates their frequency counts, returning the results as a `HashMap`.

## Examples

### CLI Example

```bash
cargo run --example simple
```

### Code Example

```rust
use ticker_sniffer::extract_tickers_from_text;

fn main() {
    let text = "Apple and Microsoft are performing well in the market.";

    match extract_tickers_from_text(text) {
        Ok(results) => {
            for (ticker_symbol, frequency) in results {
                println!("{}: {}", ticker_symbol, frequency);
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

## ChatGPT-based Alternative Name Suggester

This app, powered by ChatGPT, assists in generating alternative names for companies. For more details, refer to [data/README.md](data/README.md).

https://chatgpt.com/g/g-675e2b64d02c8191ab4819b971aeeded-stock-company-alternative-name-suggester

## License

[MIT License](LICENSE) (c) 2025 Jeremy Harris.
