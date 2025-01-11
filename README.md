## Test with Filename Capturing in Output

```bash
cargo test --features csv-support  -- --nocapture
```

## Benching

```bash
cargo bench --features csv-support
```

## WASM Test Builds

```bash
docker build -f docker/wasm-test/Dockerfile -t rust-wasm-test .
```

## Prototype `n-gram` Bin

Probably not going to use n-gram itself, but this is the latest prototype.

```bash
RUST_LOG=debug RUST_BACKTRACE=1 cargo run --bin proto_n_gram --features="csv-support env_logger"
```

## ChatGPT-based Alternative Name Suggester

This app, powered by ChatGPT, assists in generating alternative names for companies. For more details, refer to [data/company_symbol_list.csv](data/company_symbol_list.csv).

https://chatgpt.com/g/g-675e2b64d02c8191ab4819b971aeeded-stock-company-alternative-name-suggester
