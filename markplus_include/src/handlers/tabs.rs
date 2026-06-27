//! Tab processing — reads tab files referenced in frontmatter and wraps
//! the AST in `tab_group` / `tab` container nodes.

use serde_json::{json, Value};

use crate::error::IncludeError;
use crate::frontmatter::merge_frontmatter;
use crate::resolve::PathResolver;

/// Process frontmatter `tabs` entries: read each tab file, merge metadata,
/// and wrap the entire AST in a `tab_group` / `tab` structure.
///
/// If the frontmatter does not contain a `tabs` array this is a no-op.
///
/// After processing:
/// - `tabs` and `tab_default` keys are removed from meta
/// - `has_tabs: true` is set on meta
/// - The AST is replaced with a single `tab_group` node containing:
///   1. The original content as the first (active) tab
///   2. Each tab file's content as subsequent tabs
pub fn process_tabs(
    ast: &mut Vec<Value>,
    meta: &mut Option<Value>,
    resolver: &PathResolver,
) -> Result<(), IncludeError> {
    // Extract tabs from frontmatter
    let tabs = match meta {
        Some(Value::Object(ref mut obj)) => match obj.remove("tabs") {
            Some(Value::Array(tabs)) => tabs,
            _ => return Ok(()), // No tabs, nothing to do
        },
        _ => return Ok(()),
    };

    if tabs.is_empty() {
        return Ok(());
    }

    // Determine default tab name
    let default_tab_name = meta
        .as_ref()
        .and_then(|m| m.get("tab_default"))
        .and_then(|v| v.as_str())
        .unwrap_or("Article")
        .to_string();

    // Consume tab_default and set has_tabs
    if let Some(Value::Object(ref mut obj)) = meta {
        obj.remove("tab_default");
        obj.insert("has_tabs".into(), json!(true));
    }

    // Build tab nodes
    let mut tab_children: Vec<Value> = Vec::new();

    // First tab: the original content (the "Article" tab)
    let original_ast = std::mem::take(ast);
    tab_children.push(json!({
        "t": "tab",
        "title": default_tab_name,
        "active": true,
        "children": original_ast
    }));

    // Remaining tabs from frontmatter
    for tab_entry in &tabs {
        let file = tab_entry
            .get("file")
            .and_then(|v| v.as_str())
            .ok_or(IncludeError::MissingSrc)?;

        let title = tab_entry
            .get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("Untitled")
            .to_string();

        let path = resolver.resolve(file)?;
        let content =
            std::fs::read_to_string(&path).map_err(|e| IncludeError::Io {
                path: path.clone(),
                source: e,
            })?;

        let inner_asset =
            markplus_core::parse_document(&content).map_err(|e| IncludeError::MarkdownParse {
                path: path.clone(),
                message: e.to_string(),
            })?;

        // Merge tab frontmatter into parent
        merge_frontmatter(meta, &inner_asset.meta);

        tab_children.push(json!({
            "t": "tab",
            "title": title,
            "active": false,
            "children": inner_asset.ast
        }));
    }

    // Replace AST with single tab_group node
    *ast = vec![json!({
        "t": "tab_group",
        "children": tab_children
    })];

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::fs;

    fn make_resolver(tmp: &Path) -> PathResolver {
        let article = tmp.join("article.md");
        fs::write(&article, "# Main").unwrap();
        PathResolver::new(&article, None)
    }

    #[test]
    fn no_tabs_is_noop() {
        let tmp = tempfile::tempdir().unwrap();
        let resolver = make_resolver(tmp.path());
        let mut ast = vec![json!({"t": "heading", "level": 1})];
        let mut meta = Some(json!({"title": "Test"}));
        process_tabs(&mut ast, &mut meta, &resolver).unwrap();
        // AST unchanged
        assert_eq!(ast.len(), 1);
        assert_eq!(ast[0]["t"], "heading");
    }

    #[test]
    fn tabs_wrap_ast_in_tab_group() {
        let tmp = tempfile::tempdir().unwrap();
        let resolver = make_resolver(tmp.path());

        // Create tab file
        let overview = tmp.path().join("overview.md");
        fs::write(&overview, "# Overview\n\nOverview content.\n").unwrap();

        let mut ast = vec![json!({"t": "heading", "level": 1, "children": [{"t": "text", "text": "Main"}]})];
        let mut meta = Some(json!({
            "title": "Guide",
            "tabs": [
                {"file": "overview.md", "title": "Overview"}
            ]
        }));

        process_tabs(&mut ast, &mut meta, &resolver).unwrap();

        // AST should be a single tab_group
        assert_eq!(ast.len(), 1);
        assert_eq!(ast[0]["t"], "tab_group");

        let tabs = ast[0]["children"].as_array().unwrap();
        assert_eq!(tabs.len(), 2);

        // First tab: Article (default)
        assert_eq!(tabs[0]["t"], "tab");
        assert_eq!(tabs[0]["title"], "Article");
        assert_eq!(tabs[0]["active"], true);

        // Second tab: Overview
        assert_eq!(tabs[1]["t"], "tab");
        assert_eq!(tabs[1]["title"], "Overview");
        assert_eq!(tabs[1]["active"], false);
    }

    #[test]
    fn custom_default_tab_name() {
        let tmp = tempfile::tempdir().unwrap();
        let resolver = make_resolver(tmp.path());
        let tab_file = tmp.path().join("extra.md");
        fs::write(&tab_file, "# Extra").unwrap();

        let mut ast = vec![json!({"t": "paragraph"})];
        let mut meta = Some(json!({
            "tab_default": "Main Content",
            "tabs": [{"file": "extra.md", "title": "Extra"}]
        }));

        process_tabs(&mut ast, &mut meta, &resolver).unwrap();
        let tabs = ast[0]["children"].as_array().unwrap();
        assert_eq!(tabs[0]["title"], "Main Content");
    }

    #[test]
    fn tabs_consumed_from_meta() {
        let tmp = tempfile::tempdir().unwrap();
        let resolver = make_resolver(tmp.path());
        let tab_file = tmp.path().join("t.md");
        fs::write(&tab_file, "# T").unwrap();

        let mut ast = vec![];
        let mut meta = Some(json!({
            "title": "X",
            "tabs": [{"file": "t.md", "title": "T"}],
            "tab_default": "Main"
        }));

        process_tabs(&mut ast, &mut meta, &resolver).unwrap();
        let m = meta.unwrap();
        assert!(m.get("tabs").is_none());
        assert!(m.get("tab_default").is_none());
        assert_eq!(m["has_tabs"], true);
    }

    #[test]
    fn tab_frontmatter_merges_into_parent() {
        let tmp = tempfile::tempdir().unwrap();
        let resolver = make_resolver(tmp.path());
        let tab_file = tmp.path().join("tagged.md");
        fs::write(
            &tab_file,
            "---\ntags:\n  - extra\nauthor: TabAuthor\n---\n# Tagged\n",
        )
        .unwrap();

        let mut ast = vec![];
        let mut meta = Some(json!({
            "title": "Parent",
            "tags": ["main"],
            "tabs": [{"file": "tagged.md", "title": "Tagged"}]
        }));

        process_tabs(&mut ast, &mut meta, &resolver).unwrap();
        let m = meta.unwrap();
        assert_eq!(m["title"], "Parent");
        assert_eq!(m["author"], "TabAuthor");
        let tags = m["tags"].as_array().unwrap();
        assert!(tags.contains(&json!("main")));
        assert!(tags.contains(&json!("extra")));
    }

    #[test]
    fn missing_tab_file_errors() {
        let tmp = tempfile::tempdir().unwrap();
        let resolver = make_resolver(tmp.path());
        let mut ast = vec![];
        let mut meta = Some(json!({
            "tabs": [{"file": "nonexistent.md", "title": "Missing"}]
        }));
        let result = process_tabs(&mut ast, &mut meta, &resolver);
        assert!(result.is_err());
    }

    #[test]
    fn tab_missing_file_key_errors() {
        let tmp = tempfile::tempdir().unwrap();
        let resolver = make_resolver(tmp.path());
        let mut ast = vec![];
        let mut meta = Some(json!({
            "tabs": [{"title": "NoFile"}]
        }));
        let result = process_tabs(&mut ast, &mut meta, &resolver);
        assert!(matches!(result, Err(IncludeError::MissingSrc)));
    }

    #[test]
    fn empty_tabs_array_is_noop() {
        let tmp = tempfile::tempdir().unwrap();
        let resolver = make_resolver(tmp.path());
        let mut ast = vec![json!({"t": "paragraph"})];
        let mut meta = Some(json!({"tabs": []}));
        process_tabs(&mut ast, &mut meta, &resolver).unwrap();
        assert_eq!(ast.len(), 1);
        assert_eq!(ast[0]["t"], "paragraph");
    }
}
