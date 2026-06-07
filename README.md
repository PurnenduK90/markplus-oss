# MarkPlus OSS (Community Edition)

Welcome to the open-source community edition of the MarkPlus ecosystem!

This crate serves two primary purposes:
1. **Command Line Interface**: It provides the `markplus-oss` terminal command, letting you access the MarkPlus open-source tools directly from your shell.
2. **Facade Library**: It acts as a central hub, re-exporting the underlying MarkPlus crates so developers only need to manage a single dependency in their `Cargo.toml`.

## Included Components

| Crate | Role |
|---|---|
| [`markplus_core`](https://crates.io/crates/markplus_core) | Markdown → AST (JSON) compiler |
| [`markplus_render`](https://github.com/PurnenduK90/markplus-render) | AST → HTML / Typst source / PDF renderer |

## Installation

```bash
cargo install markplus
```

> The crate name is `markplus`, but the installed executable is `markplus-oss` to avoid collisions with the Pro edition.

## CLI Usage

### `core` — Parse to AST JSON

Parses a Markdown file and emits the full MarkPlus AST as JSON to stdout.

```bash
# Compact JSON
markplus-oss core document.md

# Pretty-printed JSON
markplus-oss core --pretty document.md
```

### `render` — Render with explicit template

Parses a Markdown file and renders it to the output format determined by the output file's extension. Requires a templates directory.

```bash
# Render to HTML
markplus-oss render document.md out.html

# Render to Typst source
markplus-oss render document.md out.typ

# Render to PDF
markplus-oss render document.md out.pdf

# Custom template and templates directory
markplus-oss render document.md out.html \
  --template custom/report.html.tera \
  --templates-dir ./my-templates
```

### `compile` — One-stop parse + render

The unified command — parses and renders in a single step. The output format is inferred automatically from the output file extension.

| Extension | Output |
|---|---|
| `.json` | MarkPlus AST JSON |
| `.html` | Rendered HTML |
| `.typ` | Typst source |
| `.pdf` | Compiled PDF |

```bash
markplus-oss compile document.md out.json          # AST
markplus-oss compile document.md out.html          # HTML
markplus-oss compile document.md out.typ           # Typst source
markplus-oss compile document.md out.pdf           # PDF

# Pretty-print the JSON output
markplus-oss compile --pretty document.md out.json

# Custom template
markplus-oss compile document.md out.html --template custom/page.html.tera
```

## Library Usage

Add the facade as your single dependency:

```toml
[dependencies]
# With CLI support (default)
markplus = "0.1"

# Library only — excludes clap
markplus = { version = "0.1", default-features = false }
```

Access `core` and `render` through the facade modules:

```rust
use markplus::core::{parse_document, parse_body};
use markplus::render::{RenderEngine, RenderError};

// Parse a Markdown document
let asset = parse_document("# Hello\n\nWorld.")?;

// Render to HTML
let engine = RenderEngine::builder()
    .with_templates(std::path::Path::new("templates"))
    .build()?;
let html = engine.render_html(&asset, "default/article.html.tera")?;

// Render to PDF
let typ_src = engine.render_typst_string(&asset, "default/article.typ.tera")?;
let pdf_bytes = engine.compile_pdf(&typ_src)?;
```

