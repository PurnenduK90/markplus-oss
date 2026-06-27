//! File-type handlers — dispatch an include to the right parser.

mod markdown;
mod csv_handler;
mod json_handler;
mod code;
mod mermaid;
pub mod tabs;

pub use markdown::handle_markdown_include;
pub use csv_handler::handle_csv_include;
pub use json_handler::handle_json_include;
pub use code::handle_code_include;
pub use mermaid::handle_mermaid_include;

use std::path::Path;
use serde_json::Value;

use crate::error::IncludeError;

/// Determine the handler based on file extension and process the include.
///
/// Returns the replacement AST nodes that should be spliced in place of the
/// include directive.
pub fn dispatch_include(
    path: &Path,
    attrs: &serde_json::Map<String, Value>,
    meta: &mut Option<Value>,
) -> Result<Vec<Value>, IncludeError> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");

    match ext {
        "md" | "markdown" => handle_markdown_include(path, meta),
        "csv" => handle_csv_include(path),
        "json" => handle_json_include(path),
        "mmd" | "mermaid" => handle_mermaid_include(path, attrs),
        _ if markplus_core::is_known_code_extension(ext) => {
            handle_code_include(path)
        }
        _ => Err(IncludeError::UnsupportedExtension(ext.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, Map};
    use std::fs;

    #[test]
    fn dispatch_markdown() {
        let tmp = tempfile::tempdir().unwrap();
        let md = tmp.path().join("test.md");
        fs::write(&md, "# Hello\n").unwrap();
        let mut meta = None;
        let result = dispatch_include(&md, &Map::new(), &mut meta);
        assert!(result.is_ok());
        let nodes = result.unwrap();
        assert!(!nodes.is_empty());
        assert_eq!(nodes[0]["t"], "heading");
    }

    #[test]
    fn dispatch_csv() {
        let tmp = tempfile::tempdir().unwrap();
        let csv = tmp.path().join("data.csv");
        fs::write(&csv, "a,b\n1,2\n").unwrap();
        let mut meta = None;
        let nodes = dispatch_include(&csv, &Map::new(), &mut meta).unwrap();
        assert_eq!(nodes[0]["t"], "table");
    }

    #[test]
    fn dispatch_json() {
        let tmp = tempfile::tempdir().unwrap();
        let j = tmp.path().join("data.json");
        fs::write(&j, r#"{"key":"val"}"#).unwrap();
        let mut meta = None;
        let nodes = dispatch_include(&j, &Map::new(), &mut meta).unwrap();
        assert_eq!(nodes[0]["t"], "definition_list");
    }

    #[test]
    fn dispatch_code() {
        let tmp = tempfile::tempdir().unwrap();
        let py = tmp.path().join("example.py");
        fs::write(&py, "x = 1\n").unwrap();
        let mut meta = None;
        let nodes = dispatch_include(&py, &Map::new(), &mut meta).unwrap();
        assert_eq!(nodes[0]["t"], "fenced");
        assert_eq!(nodes[0]["name"], "python");
    }

    #[test]
    fn dispatch_mermaid() {
        let tmp = tempfile::tempdir().unwrap();
        let mmd = tmp.path().join("diagram.mmd");
        fs::write(&mmd, "graph TD\nA-->B").unwrap();
        let mut meta = None;
        let nodes = dispatch_include(&mmd, &Map::new(), &mut meta).unwrap();
        assert_eq!(nodes[0]["t"], "fenced");
        assert_eq!(nodes[0]["name"], "mermaid");
    }

    #[test]
    fn dispatch_unknown_extension_errors() {
        let tmp = tempfile::tempdir().unwrap();
        let f = tmp.path().join("file.xyz");
        fs::write(&f, "data").unwrap();
        let mut meta = None;
        let result = dispatch_include(&f, &Map::new(), &mut meta);
        assert!(matches!(result, Err(IncludeError::UnsupportedExtension(_))));
    }
}
