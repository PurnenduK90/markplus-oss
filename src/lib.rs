//! # markplus
//!
//! **MarkPlus Community Facade Library**
//!
//! A single-dependency gateway to the MarkPlus open-source ecosystem.
//! Re-exports [`markplus_core`] and [`markplus_render`] under the `core` and
//! `render` modules so downstream crates only need one entry in `Cargo.toml`.
//!
//! ## Quick start
//!
//! ```toml
//! [dependencies]
//! markplus = { version = "0.1", default-features = false }
//! ```
//!
//! ```ignore
//! use markplus::core::parse_document;
//! use markplus::render::{RenderEngine, RenderError};
//!
//! let asset = parse_document("# Hello\n\nWorld.")?;
//!
//! let engine = RenderEngine::builder()
//!     .with_templates(std::path::Path::new("templates"))
//!     .build()?;
//!
//! let html = engine.render_html(&asset, "default/article.html.tera")?;
//! let typ_src = engine.render_typst_string(&asset, "default/article.typ.tera")?;
//! let pdf_bytes = engine.compile_pdf(&typ_src)?;
//! ```
//!
//! ## Modules
//!
//! | Module | Crate | Description |
//! |--------|-------|-------------|
//! | [`core`] | `markplus_core` | Markdown → AST (JSON) compiler |
//! | [`render`] | `markplus_render` | AST → HTML / Typst source / PDF renderer |

pub use markplus_core as core;
pub use markplus_render as render;
