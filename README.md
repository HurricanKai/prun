# prun

A Rust/WASM web application.

## Prerequisites

- Rust toolchain with `wasm32-unknown-unknown` target
- `wasm-bindgen-cli`

```bash
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli
```

## Build

```bash
cargo build --target wasm32-unknown-unknown --release
wasm-bindgen --target web --out-dir pkg target/wasm32-unknown-unknown/release/prun.wasm
```

## Run

Serve the project directory with any static file server:

```bash
python3 -m http.server 8000
```

Then open `http://localhost:8000` in your browser.

## Test

```bash
cargo test
```
