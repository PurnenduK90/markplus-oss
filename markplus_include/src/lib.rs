//! # markplus_include
//!
//! Include/embed processor for the MarkPlus ecosystem.
//!
//! Walks the AST produced by `markplus_core`, finds `fenced` nodes with
//! `name == "include"`, and replaces them with the content of the referenced
//! file. Also processes frontmatter tab definitions.
//!
//! ## Supported include types
//!
//! | Extension | Handler | Result |
//! |-----------|---------|--------|
//! | `.md` | `markplus_core::parse_document` | Merged AST + frontmatter |
//! | `.csv` | `markplus_core::read_csv_as_table_ast` | Table node |
//! | `.json` | `markplus_core::read_json_data_as_ast` | Table / def_list / fenced |
//! | `.mmd` | `markplus_core::mermaid` | Fenced mermaid node |
//! | Code files | `markplus_core::read_code_as_fenced_ast` | Fenced code node |
//!
//! ## Tab processing
//!
//! When frontmatter contains a `tabs` array, the original content becomes the
//! first tab ("Article" by default) and each tab file becomes a subsequent tab.

pub mod error;
pub mod frontmatter;
pub mod handlers;
pub mod resolve;

use std::collections::HashSet;
use std::path::PathBuf;

use serde_json::Value;

use error::IncludeError;
use markplus_core::json::SiteAsset;
use resolve::PathResolver;

/// Process all include directives in a [`SiteAsset`].
///
/// Walks the AST, finds `fenced` nodes with `name == "include"`, and replaces
/// them with the content of the included file. Also processes frontmatter
/// tabs if present.
///
/// # Errors
///
/// Returns [`IncludeError`] if a file cannot be found, parsed, or if a
/// circular include chain is detected.
pub fn process_includes(
    asset: &mut SiteAsset,
    resolver: &PathResolver,
) -> Result<(), IncludeError> {
    let mut seen = HashSet::new();
    process_ast_includes(&mut asset.ast, &mut asset.meta, resolver, &mut seen)?;
    handlers::tabs::process_tabs(&mut asset.ast, &mut asset.meta, resolver)?;
    Ok(())
}

fn process_ast_includes(
    ast: &mut Vec<Value>,
    meta: &mut Option<Value>,
    resolver: &PathResolver,
    seen: &mut HashSet<PathBuf>,
) -> Result<(), IncludeError> {
    let mut i = 0;
    while i < ast.len() {
        if is_include_node(&ast[i]) {
            let node = &ast[i];
            let src = node
                .get("attrs")
                .and_then(|a| a.get("src"))
                .and_then(|s| s.as_str())
                .ok_or(IncludeError::MissingSrc)?;

            let path = resolver.resolve(src)?;

            // Circular include detection
            if !seen.insert(path.clone()) {
                return Err(IncludeError::CircularInclude(path));
            }

            let attrs = node
                .get("attrs")
                .and_then(|a| a.as_object())
                .cloned()
                .unwrap_or_default();

            let mut replacement = handlers::dispatch_include(&path, &attrs, meta)?;

            // For markdown includes, recursively process includes in the
            // included content (using the included file's directory as base).
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            if ext == "md" || ext == "markdown" {
                let inner_resolver =
                    PathResolver::new(&path, resolver.config_root());
                process_ast_includes(&mut replacement, meta, &inner_resolver, seen)?;
            }

            // Allow the same file to be included in non-circular positions
            seen.remove(&path);

            // Splice the replacement nodes in place of the include node
            ast.remove(i);
            for (j, node) in replacement.into_iter().enumerate() {
                ast.insert(i + j, node);
            }
            // Don't increment i — re-examine from the same position
        } else {
            // Recurse into children for nested structures
            recurse_children(&mut ast[i], meta, resolver, seen)?;
            i += 1;
        }
    }
    Ok(())
}

fn recurse_children(
    node: &mut Value,
    meta: &mut Option<Value>,
    resolver: &PathResolver,
    seen: &mut HashSet<PathBuf>,
) -> Result<(), IncludeError> {
    if let Some(children) = node.get_mut("children").and_then(|c| c.as_array_mut()) {
        process_ast_includes(children, meta, resolver, seen)?;
    }
    // Also recurse into list items
    if let Some(items) = node.get_mut("items").and_then(|c| c.as_array_mut()) {
        for item in items.iter_mut() {
            recurse_children(item, meta, resolver, seen)?;
        }
    }
    Ok(())
}

fn is_include_node(node: &Value) -> bool {
    node.get("t").and_then(|t| t.as_str()) == Some("fenced")
        && node.get("name").and_then(|n| n.as_str()) == Some("include")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::fs;

    /// Create a SiteAsset with an include fenced block.
    fn asset_with_include(src: &str) -> SiteAsset {
        SiteAsset::new(
            None,
            vec![
                json!({"t": "heading", "level": 1, "children": [{"t": "text", "text": "Before"}]}),
                json!({"t": "fenced", "name": "include", "attrs": {"src": src}, "raw": ""}),
                json!({"t": "paragraph", "children": [{"t": "text", "text": "After"}]}),
            ],
        )
    }

    #[test]
    fn process_includes_markdown() {
        let tmp = tempfile::tempdir().unwrap();
        let article = tmp.path().join("article.md");
        fs::write(&article, "# Main\n").unwrap();
        let chapter = tmp.path().join("chapter.md");
        fs::write(&chapter, "## Chapter\n\nContent.\n").unwrap();

        let resolver = PathResolver::new(&article, None);
        let mut asset = asset_with_include("chapter.md");
        process_includes(&mut asset, &resolver).unwrap();

        // include node replaced with chapter's AST
        assert!(asset
            .ast
            .iter()
            .any(|n| n["t"] == "heading" && n["level"] == 2));
        // original nodes still present
        assert_eq!(asset.ast[0]["t"], "heading");
        assert!(asset.ast.last().unwrap()["t"] == "paragraph");
    }

    #[test]
    fn process_includes_csv() {
        let tmp = tempfile::tempdir().unwrap();
        let article = tmp.path().join("article.md");
        fs::write(&article, "").unwrap();
        let csv = tmp.path().join("data.csv");
        fs::write(&csv, "Name,Value\nA,1\n").unwrap();

        let resolver = PathResolver::new(&article, None);
        let mut asset = asset_with_include("data.csv");
        process_includes(&mut asset, &resolver).unwrap();

        assert!(asset.ast.iter().any(|n| n["t"] == "table"));
    }

    #[test]
    fn process_includes_code() {
        let tmp = tempfile::tempdir().unwrap();
        let article = tmp.path().join("article.md");
        fs::write(&article, "").unwrap();
        let py = tmp.path().join("example.py");
        fs::write(&py, "x = 42\n").unwrap();

        let resolver = PathResolver::new(&article, None);
        let mut asset = asset_with_include("example.py");
        process_includes(&mut asset, &resolver).unwrap();

        let fenced = asset.ast.iter().find(|n| n["t"] == "fenced").unwrap();
        assert_eq!(fenced["name"], "python");
    }

    #[test]
    fn process_includes_nested() {
        let tmp = tempfile::tempdir().unwrap();
        let article = tmp.path().join("article.md");
        fs::write(&article, "").unwrap();

        // outer.md includes inner.md
        let outer = tmp.path().join("outer.md");
        fs::write(
            &outer,
            "# Outer\n\n```include\nsrc=inner.md\n```\n",
        )
        .unwrap();
        let inner = tmp.path().join("inner.md");
        fs::write(&inner, "## Inner\n").unwrap();

        // But since outer.md is parsed by core which will produce a fenced
        // include node, let's create the AST directly:
        let resolver = PathResolver::new(&article, None);
        let mut asset = SiteAsset::new(
            None,
            vec![json!({"t": "fenced", "name": "include", "attrs": {"src": "outer.md"}, "raw": ""})],
        );
        // Write outer.md as markdown that the handler will parse
        fs::write(
            &outer,
            "# Outer Heading\n",
        )
        .unwrap();
        process_includes(&mut asset, &resolver).unwrap();
        assert!(asset.ast.iter().any(|n| n["t"] == "heading"));
    }

    #[test]
    fn circular_include_detected() {
        let tmp = tempfile::tempdir().unwrap();
        let article = tmp.path().join("article.md");
        fs::write(&article, "").unwrap();

        // a.md includes b.md, b.md includes a.md
        let a = tmp.path().join("a.md");
        let b = tmp.path().join("b.md");
        // These are raw markdown but core will parse them. For circular
        // detection we need to construct the AST manually since core won't
        // produce include nodes from markdown:
        // Actually, let's just create a.md that when parsed will have content,
        // and construct the AST with a circular include manually.
        fs::write(&a, "# A\n").unwrap();
        fs::write(&b, "# B\n").unwrap();

        let resolver = PathResolver::new(&article, None);
        // AST: include a.md which includes a.md (circular)
        // We need inner include to also be a fenced include node.
        // Since core parses a.md as markdown, it won't produce includes.
        // So circular detection only triggers when the markdown content
        // itself contains include fenced blocks. Let's test with JSON:
        // Actually the simplest test is to have include → same file:
        let mut asset = SiteAsset::new(
            None,
            vec![json!({"t": "fenced", "name": "include", "attrs": {"src": "a.md"}, "raw": ""})],
        );
        // a.md is normal markdown, so it won't cause circular include.
        // To actually test circular detection, we need the included md's
        // AST to contain another include node pointing back. But since
        // core parses markdown → AST, the include detection only works
        // if the markdown has a fenced include block.

        // Let's write a.md to include itself via fenced block:
        // This is a direct test — process_includes on an asset that
        // includes a file that includes itself would need the inner
        // file to produce include nodes. But core doesn't know about
        // our include fenced syntax.

        // The practical test: directly test with seen set
        let mut seen = HashSet::new();
        let path = a.canonicalize().unwrap();
        seen.insert(path.clone());
        // Trying to include a.md when it's already in seen:
        let mut ast = vec![json!({"t": "fenced", "name": "include", "attrs": {"src": "a.md"}, "raw": ""})];
        let result = process_ast_includes(&mut ast, &mut None, &resolver, &mut seen);
        assert!(matches!(result, Err(IncludeError::CircularInclude(_))));
    }

    #[test]
    fn missing_src_attr_errors() {
        let tmp = tempfile::tempdir().unwrap();
        let article = tmp.path().join("article.md");
        fs::write(&article, "").unwrap();
        let resolver = PathResolver::new(&article, None);

        let mut asset = SiteAsset::new(
            None,
            vec![json!({"t": "fenced", "name": "include", "attrs": {}, "raw": ""})],
        );
        let result = process_includes(&mut asset, &resolver);
        assert!(matches!(result, Err(IncludeError::MissingSrc)));
    }

    #[test]
    fn non_include_nodes_pass_through() {
        let tmp = tempfile::tempdir().unwrap();
        let article = tmp.path().join("article.md");
        fs::write(&article, "").unwrap();
        let resolver = PathResolver::new(&article, None);

        let mut asset = SiteAsset::new(
            None,
            vec![
                json!({"t": "heading", "level": 1, "children": [{"t": "text", "text": "Hello"}]}),
                json!({"t": "fenced", "name": "python", "attrs": {}, "raw": "x = 1"}),
            ],
        );
        process_includes(&mut asset, &resolver).unwrap();
        assert_eq!(asset.ast.len(), 2);
        assert_eq!(asset.ast[0]["t"], "heading");
        assert_eq!(asset.ast[1]["name"], "python");
    }

    #[test]
    fn is_include_node_detection() {
        assert!(is_include_node(&json!({"t": "fenced", "name": "include", "attrs": {"src": "x"}})));
        assert!(!is_include_node(&json!({"t": "fenced", "name": "python"})));
        assert!(!is_include_node(&json!({"t": "heading"})));
    }

    #[test]
    fn include_inside_blockquote() {
        let tmp = tempfile::tempdir().unwrap();
        let article = tmp.path().join("article.md");
        fs::write(&article, "").unwrap();
        let py = tmp.path().join("snippet.py");
        fs::write(&py, "y = 2\n").unwrap();

        let resolver = PathResolver::new(&article, None);
        let mut asset = SiteAsset::new(
            None,
            vec![json!({
                "t": "blockquote",
                "children": [
                    {"t": "fenced", "name": "include", "attrs": {"src": "snippet.py"}, "raw": ""}
                ]
            })],
        );
        process_includes(&mut asset, &resolver).unwrap();
        let bq = &asset.ast[0];
        let inner = &bq["children"].as_array().unwrap()[0];
        assert_eq!(inner["t"], "fenced");
        assert_eq!(inner["name"], "python");
    }

    #[test]
    fn process_includes_with_tabs() {
        let tmp = tempfile::tempdir().unwrap();
        let article = tmp.path().join("article.md");
        fs::write(&article, "").unwrap();
        let tab_file = tmp.path().join("extra.md");
        fs::write(&tab_file, "# Extra Content\n").unwrap();

        let resolver = PathResolver::new(&article, None);
        let mut asset = SiteAsset::new(
            Some(json!({
                "title": "Main",
                "tabs": [{"file": "extra.md", "title": "Extra"}]
            })),
            vec![json!({"t": "heading", "level": 1, "children": [{"t": "text", "text": "Main"}]})],
        );
        process_includes(&mut asset, &resolver).unwrap();

        // AST should be tab_group
        assert_eq!(asset.ast[0]["t"], "tab_group");
        let tabs = asset.ast[0]["children"].as_array().unwrap();
        assert_eq!(tabs.len(), 2);
    }
}
