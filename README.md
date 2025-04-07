# Lamina

Lamina is a Scheme-inspired language with a modular design that can target different backends. While it has robust support for smart contract development, its architecture allows for various compilation targets and use cases.

## Project Structure

The Lamina project is structured as a Rust workspace containing these crates:

- **[lamina](crates/lamina)** - The core language interpreter and compiler
- **[lamina-huff](crates/lamina-huff)** - Backend for compiling Lamina to Huff (EVM assembly)
- **[lx](crates/lx)** - Build tool for Lamina projects

## Getting Started

### Installation

```bash
# Clone the repository
git clone https://github.com/yourusername/lamina.git
cd lamina

# Build all components
cargo build
```

### Running the REPL

```bash
cargo run -p lamina
```

### Building a Project with lx

```bash
# Create a new project
cargo run -p lx -- new my-project
cd my-project

# Build the project
cargo run -p lx -- build
```

## Example Lamina Code

```scheme
(begin
  ;; Define a storage slot for our counter
  (define counter-slot 0)
  
  ;; Get the counter value
  (define (get-counter)
    (storage-load counter-slot))
  
  ;; Increment the counter
  (define (increment)
    (begin
      (define current (storage-load counter-slot))
      (storage-store counter-slot (+ current 1))
      (storage-load counter-slot)))
)
```

## License

[MIT](LICENSE)
