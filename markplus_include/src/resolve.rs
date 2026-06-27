//! Path resolution for include directives.
//!
//! Supports three address formats:
//! - **Relative** — resolved from the article file's directory
//! - **Absolute** — used as-is
//! - **`@/` prefix** — `@` replaced by the config root

use std::path::{Path, PathBuf};

use crate::error::IncludeError;

/// Resolves include paths relative to the current article and an optional
/// project root.
#[derive(Debug, Clone)]
pub struct PathResolver {
    /// Directory containing the article currently being processed.
    article_dir: PathBuf,
    /// Project root supplied via `--root` flag or config file.
    config_root: Option<PathBuf>,
}

impl PathResolver {
    /// Create a resolver from the article's file path and an optional root.
    ///
    /// `article_path` should point to the `.md` file; only its parent
    /// directory is retained.
    pub fn new(article_path: &Path, config_root: Option<&Path>) -> Self {
        let article_dir = article_path
            .parent()
            .unwrap_or(Path::new("."))
            .to_path_buf();
        Self {
            article_dir,
            config_root: config_root.map(Path::to_path_buf),
        }
    }

    /// Resolve a raw path string from an include directive to an absolute
    /// filesystem path.
    pub fn resolve(&self, raw_path: &str) -> Result<PathBuf, IncludeError> {
        let joined = if let Some(rest) = raw_path.strip_prefix("@/") {
            let root = self
                .config_root
                .as_ref()
                .ok_or(IncludeError::NoConfigRoot)?;
            root.join(rest)
        } else {
            let p = Path::new(raw_path);
            if p.is_absolute() {
                p.to_path_buf()
            } else {
                self.article_dir.join(p)
            }
        };

        // Canonicalize to catch symlinks and normalise `..` segments.
        let canonical = joined.canonicalize().map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                IncludeError::FileNotFound(joined.clone())
            } else {
                IncludeError::Io {
                    path: joined.clone(),
                    source: e,
                }
            }
        })?;

        Ok(canonical)
    }

    /// The directory of the article being processed.
    pub fn article_dir(&self) -> &Path {
        &self.article_dir
    }

    /// The project config root, if one was provided.
    pub fn config_root(&self) -> Option<&Path> {
        self.config_root.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    /// Helper: create a temp directory tree for resolver tests.
    fn setup_temp_dir() -> (tempfile::TempDir, PathBuf, PathBuf) {
        let tmp = tempfile::tempdir().unwrap();
        let article = tmp.path().join("docs/article.md");
        let root = tmp.path().join("project");

        fs::create_dir_all(article.parent().unwrap()).unwrap();
        fs::write(&article, "# test").unwrap();
        fs::create_dir_all(&root).unwrap();

        (tmp, article, root)
    }

    #[test]
    fn resolve_relative_path() {
        let (_tmp, article, _root) = setup_temp_dir();
        let sibling = article.parent().unwrap().join("other.md");
        fs::write(&sibling, "# other").unwrap();

        let resolver = PathResolver::new(&article, None);
        let resolved = resolver.resolve("other.md").unwrap();
        assert_eq!(resolved, sibling.canonicalize().unwrap());
    }

    #[test]
    fn resolve_absolute_path() {
        let (_tmp, article, _root) = setup_temp_dir();
        let target = article.parent().unwrap().join("abs.md");
        fs::write(&target, "# abs").unwrap();

        let resolver = PathResolver::new(&article, None);
        let abs_str = target.to_str().unwrap();
        let resolved = resolver.resolve(abs_str).unwrap();
        assert_eq!(resolved, target.canonicalize().unwrap());
    }

    #[test]
    fn resolve_at_prefix() {
        let (_tmp, article, root) = setup_temp_dir();
        let target = root.join("shared.md");
        fs::write(&target, "# shared").unwrap();

        let resolver = PathResolver::new(&article, Some(&root));
        let resolved = resolver.resolve("@/shared.md").unwrap();
        assert_eq!(resolved, target.canonicalize().unwrap());
    }

    #[test]
    fn resolve_at_prefix_without_root_errors() {
        let (_tmp, article, _root) = setup_temp_dir();
        let resolver = PathResolver::new(&article, None);
        let result = resolver.resolve("@/shared.md");
        assert!(matches!(result, Err(IncludeError::NoConfigRoot)));
    }

    #[test]
    fn resolve_missing_file_errors() {
        let (_tmp, article, _root) = setup_temp_dir();
        let resolver = PathResolver::new(&article, None);
        let result = resolver.resolve("nonexistent.md");
        assert!(matches!(result, Err(IncludeError::FileNotFound(_))));
    }

    #[test]
    fn article_dir_accessor() {
        let (_tmp, article, _root) = setup_temp_dir();
        let resolver = PathResolver::new(&article, None);
        assert_eq!(resolver.article_dir(), article.parent().unwrap());
    }

    #[test]
    fn config_root_accessor() {
        let (_tmp, article, root) = setup_temp_dir();
        let resolver = PathResolver::new(&article, Some(&root));
        assert_eq!(resolver.config_root(), Some(root.as_path()));
    }

    #[test]
    fn config_root_none_when_not_set() {
        let (_tmp, article, _root) = setup_temp_dir();
        let resolver = PathResolver::new(&article, None);
        assert!(resolver.config_root().is_none());
    }

    #[test]
    fn resolve_parent_relative() {
        let tmp = tempfile::tempdir().unwrap();
        let sub = tmp.path().join("sub");
        fs::create_dir_all(&sub).unwrap();
        let article = sub.join("article.md");
        fs::write(&article, "# a").unwrap();
        let target = tmp.path().join("root_file.md");
        fs::write(&target, "# root").unwrap();

        let resolver = PathResolver::new(&article, None);
        let resolved = resolver.resolve("../root_file.md").unwrap();
        assert_eq!(resolved, target.canonicalize().unwrap());
    }
}
