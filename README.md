# MarkPlus OSS (Community Edition)

Welcome to the open-source community edition of the MarkPlus ecosystem!

This crate serves two primary purposes:
1. **Command Line Interface**: It provides the `markplus-oss` terminal command, letting you access the MarkPlus open-source tools directly from your shell.
2. **Facade Library**: It acts as a central hub, re-exporting the underlying MarkPlus crates so developers only need to manage a single dependency in their `Cargo.toml`.

## Included Components

Currently, this central crate bundles and links to the following foundational library:

- [**markplus-core**](https://crates.io/crates/markplus-core): The core Markdown parser and Abstract Syntax Tree (AST) generator for MarkPlus.

*(More crates like `markplus-render` will be added to this facade in the future!)*

## Installation

To install the open-source CLI, run:
```bash
cargo install markplus
```
*(Note: The crate name is `markplus`, but the executable it installs is named `markplus-oss` to prevent collisions with the Pro version).*

## Usage

### As a CLI
```bash
# Parse a document and output compact JSON
markplus-oss core document.md

# Parse a document and output pretty-printed JSON
markplus-oss core --pretty document.md
```

### As a Library
If you want to use MarkPlus programmatically without pulling in CLI-specific dependencies (like `clap`), you can disable the default features in your `Cargo.toml`:
```toml
[dependencies]
markplus = { version = "0.1", default-features = false }
```

And use the re-exported core module in your Rust code:
```rust
use markplus::core::parse_document;
```
