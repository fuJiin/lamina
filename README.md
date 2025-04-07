# Lamina

Lamina is a Scheme-inspired language for smart contracts and native applications with an emphasis on functional programming and strong typing.

## Feature/LXC Branch

This branch introduces a new architecture to support compiling Lamina code to machine code and other backends:

### Architecture

Lamina now uses a multi-stage compilation pipeline with the following components:

1. **Frontend**: Parses source code and performs syntax and semantic analysis, generating a high-level IR.
2. **Middle-End**: Optimizes the high-level IR and translates it into backend-specific representations.
3. **Backends**: 
   - Native: Uses `rustc` components to emit machine code for native targets
   - EVM: Generates Huff code for Ethereum Virtual Machine

### Commands

The main CLI tool is `lx`, which now supports the following commands:

```
lx [FILE] [ARGS...]      # Run a Lamina file or start REPL if no file provided
lx new NAME              # Create a new Lamina project
lx init                  # Initialize a Lamina project in the current directory
lx build                 # Build the Lamina project using the configured backend
lx run SCRIPT            # Run a Lamina script
lx repl                  # Start the Lamina REPL
```

Additionally, we now have a separate `lxc` compiler tool for advanced compilation options:

```
lxc FILE                # Compile a Lamina file to native code
lxc check FILE          # Check a Lamina file for errors
lxc ir FILE             # Print the IR for a Lamina file
```

### REPL Environment

The integrated REPL environment allows for interactive development with:

- Dynamic compilation and execution
- Immediate feedback
- Ability to load and modify definitions on the fly

## Project Structure

```
crates/
  ├── lamina/        # Core language implementation
  ├── lamina-ir/     # Intermediate representation
  ├── lamina-huff/   # EVM backend using Huff
  ├── lx/            # Main CLI tool
  └── lxc/           # Native compiler
```

## Installation

```bash
# Clone the repository
git clone https://github.com/yourusername/lamina.git
cd lamina

# Build the project
cargo build

# Install the CLI tools
cargo install --path crates/lx
cargo install --path crates/lxc
```

## Getting Started

```bash
# Create a new project
lx new my-project

# Enter the project directory
cd my-project

# Run the REPL
lx repl

# Compile and run a file
lx run examples/hello.lam

# Build the project
lx build
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
