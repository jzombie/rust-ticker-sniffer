## Test with Filename Capturing in Output

```bash
cargo test --features csv-support  -- --nocapture
```

## Benching

```bash
cargo bench --features csv-support
```

## Tuning Weights

```bash
cargo run --bin tune --features="csv-support rand-support"
```

## WASM Test Builds

```bash
docker build -f docker/wasm-test/Dockerfile -t rust-wasm-test .
```
