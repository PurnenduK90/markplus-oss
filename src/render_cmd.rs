use clap::Args;
use markplus_core::parse_document;
use markplus_render::{RenderEngine, RenderError};
use std::{path::Path, process};

/// Arguments for the `render` subcommand.
///
/// Parses a Markdown file and renders it to a file. The output format is
/// determined by the extension of `output`:
///
/// | Extension | Output |
/// |-----------|--------|
/// | `.html`   | Rendered HTML via Tera template |
/// | `.typ`    | Typst source via Tera template |
/// | `.pdf`    | Compiled PDF (Typst source compiled in-process) |
///
/// A templates directory must be available (default: `"templates"`).
/// The template name defaults to `default/article.html.tera` for HTML and
/// `default/article.typ.tera` for Typst/PDF output.
///
/// # Examples
///
/// ```bash
/// markplus-oss render document.md out.html
/// markplus-oss render document.md out.pdf --template custom/report.typ.tera
/// markplus-oss render document.md out.html --templates-dir ./my-templates
/// ```

#[derive(Args)]
pub struct RenderArgs {
    /// Input Markdown file
    pub file: String,

    /// Output file (.html, .typ, or .pdf — format is inferred from extension)
    pub output: String,

    /// Tera template name to use (e.g. "default/article.html.tera")
    #[arg(short, long)]
    pub template: Option<String>,

    /// Templates directory (default: "templates")
    #[arg(long, default_value = "templates")]
    pub templates_dir: String,
}

pub fn run(args: &RenderArgs) {
    let raw = std::fs::read_to_string(&args.file).unwrap_or_else(|e| {
        eprintln!("markplus render: cannot read {}: {e}", args.file);
        process::exit(1);
    });

    let asset = parse_document(&raw).unwrap_or_else(|e| {
        eprintln!("markplus render: parse error: {e}");
        process::exit(1);
    });

    let engine = RenderEngine::builder()
        .with_templates(Path::new(&args.templates_dir))
        .build()
        .unwrap_or_else(|e| {
            eprintln!(
                "markplus render: failed to load templates from '{}': {e}",
                args.templates_dir
            );
            process::exit(1);
        });

    let dest = Path::new(&args.output);
    let ext = dest.extension().and_then(|e| e.to_str()).unwrap_or("");

    let template_name = args.template.clone().unwrap_or_else(|| {
        match ext {
            "html" => "default/article.html.tera".into(),
            "typ" | "pdf" => "default/article.typ.tera".into(),
            other => {
                eprintln!("markplus render: unsupported output extension '{other}' — use .html, .typ, or .pdf");
                process::exit(1);
            }
        }
    });

    engine
        .render_to_file(&asset, &template_name, dest)
        .unwrap_or_else(|e| {
            eprintln!("markplus render: {e}");
            if let RenderError::TypstCompile(_) = e {
                process::exit(2);
            }
            process::exit(1);
        });
}
