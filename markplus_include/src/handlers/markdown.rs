//! Markdown include handler — uses `markplus_core::parse_document` to parse
//! the included `.md` file and merges its frontmatter into the parent.

use std::path::Path;
use serde_json::Value;

use crate::error::IncludeError;
use crate::frontmatter::merge_frontmatter;

/// Parse an included Markdown file with `markplus_core` and merge its
/// frontmatter into the parent document's metadata.
///
/// Returns the AST nodes from the included document to be spliced in place
/// of the include directive.
pub fn handle_markdown_include(
    path: &Path,
    parent_meta: &mut Option<Value>,
) -> Result<Vec<Value>, IncludeError> {
    let content = std::fs::read_to_string(path).map_err(|e| IncludeError::Io {
        path: path.to_path_buf(),
        source: e,
    })?;

    let inner_asset =
        markplus_core::parse_document(&content).map_err(|e| IncludeError::MarkdownParse {
            path: path.to_path_buf(),
            message: e.to_string(),
        })?;

    // Merge frontmatter (parent wins on conflicts)
    merge_frontmatter(parent_meta, &inner_asset.meta);

    Ok(inner_asset.ast)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::fs;

    #[test]
    fn include_simple_markdown() {
        let tmp = tempfile::tempdir().unwrap();
        let md = tmp.path().join("chapter.md");
        fs::write(&md, "# Chapter One\n\nSome content.\n").unwrap();

        let mut meta = None;
        let nodes = handle_markdown_include(&md, &mut meta).unwrap();
        assert!(!nodes.is_empty());
        assert_eq!(nodes[0]["t"], "heading");
        assert_eq!(nodes[0]["level"], 1);
    }

    #[test]
    fn include_merges_frontmatter() {
        let tmp = tempfile::tempdir().unwrap();
        let md = tmp.path().join("with_meta.md");
        fs::write(
            &md,
            "---\ntags:\n  - inner_tag\nauthor: Inner\n---\n# Title\n",
        )
        .unwrap();

        let mut meta = Some(json!({"title": "Parent", "tags": ["outer_tag"]}));
        let _nodes = handle_markdown_include(&md, &mut meta).unwrap();
        let m = meta.unwrap();
        assert_eq!(m["title"], "Parent"); // parent wins
        assert_eq!(m["author"], "Inner"); // added from inner
        let tags = m["tags"].as_array().unwrap();
        assert!(tags.contains(&json!("outer_tag")));
        assert!(tags.contains(&json!("inner_tag")));
    }

    #[test]
    fn include_missing_file_errors() {
        let result = handle_markdown_include(Path::new("/nonexistent/file.md"), &mut None);
        assert!(matches!(result, Err(IncludeError::Io { .. })));
    }
}
