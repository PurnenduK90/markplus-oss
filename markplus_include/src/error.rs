//! Error types for the include processor.

use std::path::PathBuf;

/// Errors that can occur while processing include directives.
#[derive(Debug)]
pub enum IncludeError {
    /// The referenced file does not exist.
    FileNotFound(PathBuf),

    /// An I/O error occurred while reading a file.
    Io {
        path: PathBuf,
        source: std::io::Error,
    },

    /// A CSV file could not be parsed.
    CsvParse { path: PathBuf, message: String },

    /// A JSON file could not be parsed.
    JsonParse {
        path: PathBuf,
        source: serde_json::Error,
    },

    /// An included Markdown file failed to parse via `markplus_core`.
    MarkdownParse { path: PathBuf, message: String },

    /// The include fenced block is missing the required `src` attribute.
    MissingSrc,

    /// The file extension is not recognised by any handler.
    UnsupportedExtension(String),

    /// A circular include chain was detected.
    CircularInclude(PathBuf),

    /// The `@/` prefix was used but no config root was provided.
    NoConfigRoot,
}

impl std::fmt::Display for IncludeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FileNotFound(p) => write!(f, "file not found: {}", p.display()),
            Self::Io { path, source } => {
                write!(f, "IO error reading {}: {source}", path.display())
            }
            Self::CsvParse { path, message } => {
                write!(f, "CSV parse error in {}: {message}", path.display())
            }
            Self::JsonParse { path, source } => {
                write!(f, "JSON parse error in {}: {source}", path.display())
            }
            Self::MarkdownParse { path, message } => {
                write!(f, "markdown parse error in {}: {message}", path.display())
            }
            Self::MissingSrc => write!(f, "include node missing 'src' attribute"),
            Self::UnsupportedExtension(ext) => {
                write!(f, "unsupported file extension: {ext}")
            }
            Self::CircularInclude(p) => {
                write!(f, "circular include detected: {}", p.display())
            }
            Self::NoConfigRoot => write!(
                f,
                "@/ prefix used but no config root provided (use --root flag)"
            ),
        }
    }
}

impl std::error::Error for IncludeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            Self::JsonParse { source, .. } => Some(source),
            _ => None,
        }
    }
}
