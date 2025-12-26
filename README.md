# PrUn Star Map

An interactive 2D star map visualization for [Prosperous Universe](https://prosperousuniverse.com/) using data from the [FIO API](https://rest.fnar.net/).

Built with Rust/WASM using [egui](https://github.com/emilk/egui) for the UI and [petgraph](https://github.com/petgraph/petgraph) for graph data structures.

## Features

- **Interactive 2D Map**: Pan and zoom to explore the star systems
- **Multiple Projections**: View the map in X-Y, X-Z, or Y-Z planes
- **Star Type Colors**: Stars are colored by their spectral type (O, B, A, F, G, K, M)
- **Connection Visualization**: See jump connections between star systems
- **Search**: Find stars by name or ID
- **Star Details**: Click on a star to see its details and connections

## Prerequisites

- Rust toolchain with `wasm32-unknown-unknown` target
- [trunk](https://trunkrs.dev/) for building and serving

```bash
rustup target add wasm32-unknown-unknown
cargo install trunk
```

## Development

Run the development server with hot reloading:

```bash
trunk serve
```

Then open `http://localhost:8080` in your browser.

## Build

Build for production:

```bash
trunk build --release
```

The output will be in the `dist/` directory.

## Data Source

Star system data is fetched from the FIO REST API at `https://rest.fnar.net/systemstars`.

## Architecture

- **src/lib.rs**: Main application with egui UI and rendering
- **src/data.rs**: Data models for star systems and graph structure
- **src/api.rs**: FIO API client using the Fetch API
