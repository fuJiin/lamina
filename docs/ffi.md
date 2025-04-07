# Lamina FFI: Bi-directional Interoperability with Rust

This document describes how to use Lamina's Foreign Function Interface (FFI) to achieve bi-directional interoperability between Lamina and Rust. With this system, you can:

1. Call Rust functions from Lamina code
2. Use Lamina as an embedded scripting language in Rust applications
3. Create modular Rust libraries that can be imported into Lamina code

## 1. Calling Rust Functions from Lamina

The simplest way to make Rust functionality available to Lamina code is by registering individual functions:

```rust
use lamina::ffi;
use lamina::value::Value;

// Register a Rust function that Lamina can call
ffi::register_function("rust-multiply", |args| {
    if args.len() != 2 {
        return Err("rust-multiply requires 2 arguments".into());
    }
    
    let arg1 = ffi::value_to_f64(&args[0])?;
    let arg2 = ffi::value_to_f64(&args[1])?;
    
    Ok(ffi::f64_to_value(arg1 * arg2))
});
```

Once registered, Lamina code can call this function:

```scheme
(define result (rust-multiply 3.5 2.5))
```

### Type Conversion

The FFI system provides helper functions for converting between Rust and Lamina types:

- `ffi::i64_to_value` - Convert a Rust i64 to a Lamina value
- `ffi::f64_to_value` - Convert a Rust f64 to a Lamina value
- `ffi::bool_to_value` - Convert a Rust bool to a Lamina value
- `ffi::string_to_value` - Convert a Rust String to a Lamina value

And for the reverse direction:

- `ffi::value_to_i64` - Convert a Lamina value to a Rust i64
- `ffi::value_to_f64` - Convert a Lamina value to a Rust f64
- `ffi::value_to_bool` - Convert a Lamina value to a Rust bool
- `ffi::value_to_string` - Convert a Lamina value to a Rust String

## 2. Using Lamina as an Embedded Language in Rust

Lamina provides an embedding API that allows Rust applications to use Lamina as a scripting language:

```rust
use lamina::embed;
use lamina::value::Value;

fn main() {
    // Create a Lamina interpreter
    let interpreter = embed::init();
    
    // Evaluate Lamina code
    let result = interpreter.eval("(+ 1 2 3)").unwrap();
    println!("Result: {}", result);
    
    // Define variables in the Lamina environment
    interpreter.define("my-var", Value::from(42));
    
    // Get variables from the Lamina environment
    let value = interpreter.get("my-var").unwrap();
    
    // Call Lamina procedures from Rust
    let args = vec![Value::from(5), Value::from(7)];
    let result = interpreter.call("+", args).unwrap();
    
    // Register Rust functions in the Lamina environment
    interpreter.register_function("rust-function", |args| {
        // Implementation...
        Ok(Value::from(42))
    });
}
```

## 3. Creating Rust Modules for Lamina

For more complex integrations, you can create Rust modules that can be imported into Lamina code:

```rust
use lamina::ffi::rustlib;

// Create a module
rustlib::create_module("math", |module| {
    // Add functions to the module
    module.add_function("add", |args| {
        // Implementation...
        Ok(ffi::f64_to_value(result))
    });
    
    module.add_function("subtract", |args| {
        // Implementation...
        Ok(ffi::f64_to_value(result))
    });
});
```

Lamina code can then access these functions with qualified names:

```scheme
; Use math functions from the Rust module
(define result (math/add 5 3))
(display (math/subtract 10 4))
```

### Import Modules at Runtime

You can also import Rust modules at runtime using:

```rust
rustlib::import_module("math", &environment);
```

## Best Practices for FFI

1. **Error Handling:** Always return clear error messages from Rust functions when argument validation fails.

2. **Type Safety:** Use the provided type conversion functions to ensure proper type handling between Rust and Lamina.

3. **Performance:** For performance-critical operations, consider implementing them in Rust and exposing them to Lamina via FFI.

4. **Module Organization:** Group related functionality into modules for better organization and to avoid namespace pollution.

5. **Documentation:** Document the interfaces of your exposed Rust functions so Lamina users know how to use them correctly.

## Examples

For complete examples of using the FFI system, see the example files in the `examples/` directory:

- `rust_to_lamina.rs` - Example of using Lamina from Rust
- `lamina_to_rust.rs` - Example of using Rust from Lamina
- `rust_module.rs` - Example of creating and using Rust modules in Lamina 