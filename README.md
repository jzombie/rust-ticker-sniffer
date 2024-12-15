## Test with Filename Capturing in Output

```bash
cargo test --features csv-support  -- --nocapture
```

## Benching

```bash
cargo bench --features csv-support
```

## Tuning Weights

This process adjusts the weights used in the ticker extraction and matching algorithms to minimize errors. It employs gradient descent with momentum and regularization to optimize performance based on test cases.

```bash
cargo run --bin tune --features="csv-support rand-support libc-support"
```

## Training Bias Adjuster

```bash
cargo run --bin train_bias_adjuster --features="csv-support rand-support libc-support"
```

## WASM Test Builds

```bash
docker build -f docker/wasm-test/Dockerfile -t rust-wasm-test .
```

## Prototype `n-gram` Bin

Probably not going to use n-gram itself, but this is the latest prototype.

```bash
RUST_BACKTRACE=1 cargo run --bin proto_n_gram --features="csv-support rand-support libc-support"
```

## ChatGPT-based Alternative Name Suggester

https://chatgpt.com/g/g-675e2b64d02c8191ab4819b971aeeded-stock-company-alternative-name-suggester
