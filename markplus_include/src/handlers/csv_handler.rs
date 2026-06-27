//! CSV include handler — delegates to `markplus_core::read_csv_as_table_ast`.

use std::path::Path;
use serde_json::Value;

use crate::error::IncludeError;

/// Read a CSV file and return a table AST node using `markplus_core`.
///
/// Uses default options: comma delimiter, header auto-detected from first
/// row, no column/row slicing.
pub fn handle_csv_include(path: &Path) -> Result<Vec<Value>, IncludeError> {
    let opts = markplus_core::CsvReadOptions {
        header: true,
        ..Default::default()
    };
    let table = markplus_core::read_csv_as_table_ast(path, &opts).map_err(|msg| {
        IncludeError::CsvParse {
            path: path.to_path_buf(),
            message: msg,
        }
    })?;
    Ok(vec![table])
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn csv_produces_table_node() {
        let tmp = tempfile::tempdir().unwrap();
        let csv = tmp.path().join("data.csv");
        fs::write(&csv, "Name,Value\nAlice,42\nBob,17\n").unwrap();
        let nodes = handle_csv_include(&csv).unwrap();
        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0]["t"], "table");
        let headers = nodes[0]["headers"].as_array().unwrap();
        assert_eq!(headers.len(), 2);
        assert_eq!(headers[0]["children"][0]["text"], "Name");
    }

    #[test]
    fn csv_missing_file_errors() {
        let result = handle_csv_include(Path::new("/nonexistent/data.csv"));
        assert!(matches!(result, Err(IncludeError::CsvParse { .. })));
    }
}
