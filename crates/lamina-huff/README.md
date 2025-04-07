# Lamina Huff Backend

A backend for the Lamina language that compiles Lamina code to [Huff](https://github.com/huff-language/huff-rs), an EVM assembly language.

## Features

- Compiles Lamina code to Huff
- Automatic function selector generation
- Mapping of Lamina constructs to EVM opcodes

## Usage

```rust
use lamina_huff::huff;
use lamina::parser;
use lamina::lexer;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let code = r#"
    (define (my-function x)
      (+ x 1))
    "#;
    
    let tokens = lexer::lex(code)?;
    let expr = parser::parse(&tokens)?;
    
    let huff_code = huff::compile(&expr, "MyContract")?;
    println!("{}", huff_code);
    
    Ok(())
}
```

See the `examples/` directory for more comprehensive examples. 