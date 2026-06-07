use clap::Args;
use markplus_core::parse_document;
use markplus_render::RenderEngine;
use std::{path::Path, process};

/// Arguments for the `compile` subcommand.
///
/// The unified one-stop command — parses a Markdown file and writes the
/// result to `output` in the format inferred from its file extension:
///
/// | Extension | Output |
/// |-----------|--------|
/// | `.json`   | MarkPlus AST JSON (compact or pretty with `--pretty`) |
/// | `.html`   | Rendered HTML via Tera template |
/// | `.typ`    | Typst source via Tera template |
/// | `.pdf`    | Compiled PDF (Typst source compiled in-process) |
///
/// For render targets (`.html`, `.typ`, `.pdf`) a templates directory must be
/// available. The template name defaults to `default/article.html.tera` for
/// HTML and `default/article.typ.tera` for Typst/PDF output.
///
/// # Examples
///
/// ```bash
/// markplus-oss compile document.md out.json
/// markplus-oss compile --pretty document.md out.json
/// markplus-oss compile document.md out.html
/// markplus-oss compile document.md out.pdf
/// markplus-oss compile document.md out.html --template custom/page.html.tera
/// ```

#[derive(Args)]
pub struct CompileArgs {
    /// Input Markdown file
    pub file: String,

    /// Output file — format inferred from extension:
    /// .json → AST, .html → HTML, .typ → Typst source, .pdf → PDF
    pub output: String,

    /// Tera template name (render targets only; inferred from extension if omitted)
    #[arg(short, long)]
    pub template: Option<String>,

    /// Templates directory (render targets only; default: "templates")
    #[arg(long, default_value = "templates")]
    pub templates_dir: String,

    /// Pretty-print JSON output (AST target only)
    #[arg(short, long)]
    pub pretty: bool,
}

pub fn run(args: &CompileArgs) {
    let raw = std::fs::read_to_string(&args.file).unwrap_or_else(|e| {
        eprintln!("markplus compile: cannot read {}: {e}", args.file);
        process::exit(1);
    });

    let asset = parse_document(&raw).unwrap_or_else(|e| {
        eprintln!("markplus compile: parse error: {e}");
        process::exit(1);
    });

    let dest = Path::new(&args.output);
    let ext = dest.extension().and_then(|e| e.to_str()).unwrap_or("");

    match ext {
        "json" => {
            let json = if args.pretty {
                asset.to_json_pretty()
            } else {
                asset.to_json()
            }
            .unwrap_or_else(|e| {
                eprintln!("markplus compile: serialisation error: {e}");
                process::exit(1);
            });
            std::fs::write(dest, json).unwrap_or_else(|e| {
                eprintln!("markplus compile: cannot write {}: {e}", args.output);
                process::exit(1);
            });
        }
        "html" | "typ" | "pdf" => {
            let template_name = args.template.clone().unwrap_or_else(|| match ext {
                "html" => "default/article.html.tera".into(),
                _ => "default/article.typ.tera".into(),
            });

            let engine = RenderEngine::builder()
                .with_templates(Path::new(&args.templates_dir))
                .build()
                .unwrap_or_else(|e| {
                    eprintln!(
                        "markplus compile: failed to load templates from '{}': {e}",
                        args.templates_dir
                    );
                    process::exit(1);
                });

            engine
                .render_to_file(&asset, &template_name, dest)
                .unwrap_or_else(|e| {
                    eprintln!("markplus compile: {e}");
                    process::exit(1);
                });
        }
        other => {
            eprintln!(
                "markplus compile: unsupported output extension '{other}' — use .json, .html, .typ, or .pdf"
            );
            process::exit(1);
        }
    }
}
