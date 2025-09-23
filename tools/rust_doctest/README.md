# rust_doctest

Rust documentation snippet test generator using `poly_doctest`.

This crate demonstrates how to use the `poly_doctest` library to generate Rust test files from documentation snippets.

## Installation

```bash
cargo install --path .
```

## Usage

### Local Documentation

```bash
rust-doctest --local ./docs --output ./tests/docs
```

### GitHub Repository

```bash
rust-doctest --owner myorg --repo myproject --branch main --remote-path docs/ --output ./tests/docs
```

## Generated Output

The tool generates:
- `main.rs`: Module declarations and common imports
- `module_name.rs`: Individual test modules per documentation file

Each code snippet becomes a `#[test]` function with appropriate setup and imports.

## Code Snippet Format

Documentation should use the following format:

````markdown
# My Feature

```rust test
fn example() {
    HIDE: use my_crate::prelude::*;  // Hidden setup
    let result = my_function();       // Included in test
    assert!(result.is_ok());
}
```
````

## Examples

See the generated tests in the Flecs-Rust project for real-world usage examples.

## License

MIT OR Apache-2.0