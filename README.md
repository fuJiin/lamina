# Lamina

R7RS Scheme implementation in Rust with AI, just for fun.

## Features

- R7RS Scheme compatibility
- REPL interface for interactive development
- Bi-directional FFI with Rust:
  - Call Rust functions from Lamina
  - Use Lamina as an embedded language in Rust applications
  - Create Rust modules that can be imported into Lamina

## Getting Started

Build the project:

```bash
cargo build
```

Run the REPL:

```bash
cargo run
```

Try out the examples:

```bash
# Example of using Lamina from Rust
cargo run --example rust_to_lamina

# Example of using Rust from Lamina
cargo run --example lamina_to_rust

# Example of creating and using Rust modules in Lamina
cargo run --example rust_module
```

## Documentation

For more information about the FFI system, see [docs/ffi.md](docs/ffi.md).
