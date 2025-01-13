# Ticker Sniffer

`Ticker Sniffer` is a Rust library built to parse and extract ticker symbols from text documents. It specializes in analyzing text content to identify references to U.S. stock market ticker symbols and calculates their frequency counts, returning the results as a `HashMap`.

```rust
pub type TickerSymbol = String;
pub type TickerSymbolFrequency = usize;
pub type TickerSymbolFrequencyMap = HashMap<TickerSymbol, TickerSymbolFrequency>;
```


## Test with Filename Capturing in Output

```bash
cargo test -- --nocapture
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

## ChatGPT-based Alternative Name Suggester

This app, powered by ChatGPT, assists in generating alternative names for companies. For more details, refer to [data/README.md](data/README.md).

https://chatgpt.com/g/g-675e2b64d02c8191ab4819b971aeeded-stock-company-alternative-name-suggester
