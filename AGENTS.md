# Agent Guidelines

## Project Overview

This is a Rust/WASM web application. Logic is written in Rust and compiled to WebAssembly.

## Architecture

- **No bundlers**: We do not use bundlers (webpack, vite, esbuild, etc.). Plain HTML with `<script type="module">` tags and relative imports.
- **No wasm-pack**: Build WASM directly with cargo and use `wasm-bindgen-cli` to generate JS bindings.

## Build Process

1. Compile Rust to WASM:
   ```bash
   cargo build --target wasm32-unknown-unknown --release
   ```

2. Generate JS bindings:
   ```bash
   wasm-bindgen --target web --out-dir pkg target/wasm32-unknown-unknown/release/prun.wasm
   ```

## File Structure

- `src/lib.rs` - Rust source code
- `index.html` - HTML entry point
- `pkg/` - Generated WASM and JS bindings (after build)

## Running Locally

Serve with any static file server:
```bash
python3 -m http.server 8000
```

## Testing

```bash
cargo test
```
