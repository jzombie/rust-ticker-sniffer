# Ticker Sniffer

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
pub type TickerSymbol = String;
pub type TickerSymbolFrequency = usize;
pub type TickerSymbolFrequencyMap = HashMap<TickerSymbol, TickerSymbolFrequency>;
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

## WASM Test Builds

```bash
DOCKER_BUILDKIT=0 docker build -f docker/wasm-test/Dockerfile -t rust-wasm-test .
```

## Prototype `n-gram` Bin

Probably not going to use n-gram itself, but this is the latest prototype.

```bash
RUST_LOG=debug RUST_BACKTRACE=1 cargo run --bin proto_n_gram --features="logger-support"
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

## ChatGPT-based Alternative Name Suggester

This app, powered by ChatGPT, assists in generating alternative names for companies. For more details, refer to [data/README.md](data/README.md).

https://chatgpt.com/g/g-675e2b64d02c8191ab4819b971aeeded-stock-company-alternative-name-suggester
