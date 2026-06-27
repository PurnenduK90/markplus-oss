//! Mermaid include handler — delegates to `markplus_core::mermaid`.

use serde_json::{Map, Value};
use std::path::Path;

use crate::error::IncludeError;

/// Read a `.mmd` file and return a fenced mermaid AST node using `markplus_core`.
///
/// Extra attributes from the include directive (e.g. `theme=dark`) are
/// passed through. The `src` key is filtered out by core.
pub fn handle_mermaid_include(
    path: &Path,
    attrs: &Map<String, Value>,
) -> Result<Vec<Value>, IncludeError> {
    let node = markplus_core::mermaid::read_mermaid_as_fenced_ast_with_attrs(path, attrs).map_err(
        |msg| IncludeError::Io {
            path: path.to_path_buf(),
            source: std::io::Error::other(msg),
        },
    )?;
    Ok(vec![node])
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::fs;

    #[test]
    fn mermaid_produces_fenced_node() {
        let tmp = tempfile::tempdir().unwrap();
        let mmd = tmp.path().join("flow.mmd");
        fs::write(&mmd, "graph TD\n  A-->B\n").unwrap();
        let nodes = handle_mermaid_include(&mmd, &Map::new()).unwrap();
        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0]["t"], "fenced");
        assert_eq!(nodes[0]["name"], "mermaid");
        assert!(nodes[0]["raw"].as_str().unwrap().contains("graph TD"));
    }

    #[test]
    fn mermaid_passes_attrs_through() {
        let tmp = tempfile::tempdir().unwrap();
        let mmd = tmp.path().join("styled.mmd");
        fs::write(&mmd, "graph LR\nX-->Y").unwrap();
        let mut attrs = Map::new();
        attrs.insert("theme".into(), json!("dark"));
        attrs.insert("src".into(), json!("./styled.mmd"));
        let nodes = handle_mermaid_include(&mmd, &attrs).unwrap();
        assert_eq!(nodes[0]["attrs"]["theme"], "dark");
        assert!(nodes[0]["attrs"].get("src").is_none());
    }

    #[test]
    fn mermaid_missing_file_errors() {
        let result = handle_mermaid_include(Path::new("/nonexistent/diagram.mmd"), &Map::new());
        assert!(matches!(result, Err(IncludeError::Io { .. })));
    }
}
