//! Code file include handler — delegates to `markplus_core::read_code_as_fenced_ast`.

use serde_json::Value;
use std::path::Path;

use crate::error::IncludeError;

/// Read a source-code file and return a fenced AST node using `markplus_core`.
pub fn handle_code_include(path: &Path) -> Result<Vec<Value>, IncludeError> {
    let node = markplus_core::read_code_as_fenced_ast(path).map_err(|msg| IncludeError::Io {
        path: path.to_path_buf(),
        source: std::io::Error::other(msg),
    })?;
    Ok(vec![node])
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn code_produces_fenced_node() {
        let tmp = tempfile::tempdir().unwrap();
        let py = tmp.path().join("example.py");
        fs::write(&py, "print('hello')\n").unwrap();
        let nodes = handle_code_include(&py).unwrap();
        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0]["t"], "fenced");
        assert_eq!(nodes[0]["name"], "python");
        assert_eq!(nodes[0]["raw"], "print('hello')");
    }

    #[test]
    fn code_rust_extension() {
        let tmp = tempfile::tempdir().unwrap();
        let rs = tmp.path().join("main.rs");
        fs::write(&rs, "fn main() {}\n").unwrap();
        let nodes = handle_code_include(&rs).unwrap();
        assert_eq!(nodes[0]["name"], "rust");
    }

    #[test]
    fn code_missing_file_errors() {
        let result = handle_code_include(Path::new("/nonexistent/code.py"));
        assert!(matches!(result, Err(IncludeError::Io { .. })));
    }
}
