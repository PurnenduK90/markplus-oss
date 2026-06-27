//! JSON data include handler — delegates to `markplus_core::read_json_data_as_ast`.

use serde_json::Value;
use std::path::Path;

use crate::error::IncludeError;

/// Read a JSON data file and return displayable AST nodes using `markplus_core`.
///
/// Delegates to `markplus_core::read_json_data_as_ast` which returns:
/// - Array of objects → table node
/// - Single object    → definition_list node
/// - Other            → fenced JSON node
pub fn handle_json_include(path: &Path) -> Result<Vec<Value>, IncludeError> {
    markplus_core::read_json_data_as_ast(path).map_err(|msg| {
        // Try to determine if it was a parse error or IO error
        if msg.contains("No such file") || msg.contains("cannot find") || msg.contains("not found")
        {
            IncludeError::Io {
                path: path.to_path_buf(),
                source: std::io::Error::new(std::io::ErrorKind::NotFound, msg),
            }
        } else {
            IncludeError::JsonParse {
                path: path.to_path_buf(),
                source: serde_json::from_str::<serde_json::Value>(&msg).unwrap_err(),
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn json_array_becomes_table() {
        let tmp = tempfile::tempdir().unwrap();
        let j = tmp.path().join("arr.json");
        fs::write(&j, r#"[{"name":"Alice","age":30}]"#).unwrap();
        let nodes = handle_json_include(&j).unwrap();
        assert_eq!(nodes[0]["t"], "table");
    }

    #[test]
    fn json_object_becomes_definition_list() {
        let tmp = tempfile::tempdir().unwrap();
        let j = tmp.path().join("obj.json");
        fs::write(&j, r#"{"key":"val","num":42}"#).unwrap();
        let nodes = handle_json_include(&j).unwrap();
        assert_eq!(nodes[0]["t"], "definition_list");
    }

    #[test]
    fn json_scalar_becomes_fenced() {
        let tmp = tempfile::tempdir().unwrap();
        let j = tmp.path().join("scalar.json");
        fs::write(&j, r#""hello world""#).unwrap();
        let nodes = handle_json_include(&j).unwrap();
        assert_eq!(nodes[0]["t"], "fenced");
        assert_eq!(nodes[0]["name"], "json");
    }
}
