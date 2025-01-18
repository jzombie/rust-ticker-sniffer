# Ticker Sniffer (Work in Progress)

`Ticker Sniffer` is a Rust library built to parse and extract ticker symbols from text documents. It specializes in analyzing text content to identify references to U.S. stock market ticker symbols and calculates their frequency counts, returning the results as a `HashMap`.

## How it Works (draft)

The text search engine employs a hybrid approach to identify company names and stock symbols in documents.

Initially, it filters out stop words and applies a sequence-based tokenizer to detect potential company names, preserving word order for contextual accuracy.

Simultaneously, a secondary tokenizer uses a Bag of Words approach to identify stock symbols, which may occasionally collide with stop words.

The engine calculates a ratio by comparing the number of company name matches to exact stock symbol matches found in the document.

Based on this ratio, it determines whether to include exact stock symbol matches in the results.

Regardless of the decision, the engine ensures that stock symbols are always matched, but the contextual importance of symbols is weighted by their relationship to identified company names.

---

```rust
use ticker_sniffer::extract_tickers_from_text;

fn main() {
    let text = "Apple and Microsoft are performing well in the market.";

    match extract_tickers_from_text(text) {
        Ok(results) => {
            for (ticker, frequency) in results {
                println!("{}: {}", ticker, frequency);
            }
        }
        Err(e) => eprintln!("Error extracting tickers: {}", e),
    }
}
```

## Test with Filename Capturing in Output

To run all tests:

```bash
cargo test -- --nocapture
```

To run Tokenizer tests in isolation:

```bash
cargo test --test tokenizer_tests  -- --nocapture
```

## Benching

```bash
cargo bench
```

## Prototype Bin (temporary debug bin)

```bash
cargo run --manifest-path dev/Cargo.toml
```

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

```bash
cargo build --release --bin ticker-sniffer-cli --features="logger-support"
```

## Running CLI tool (on Unix)

```bash
./target/release/ticker-sniffer-cli
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
