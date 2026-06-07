use clap::Args;
use markplus_core::parse_document;
use std::{fs, process};

/// Arguments for the `core` subcommand.
///
/// Parses a Markdown file and emits the full MarkPlus AST as JSON to stdout.
///
/// # Examples
///
/// ```bash
/// markplus-oss core document.md
/// markplus-oss core --pretty document.md
/// ```

#[derive(Args)]
pub struct CoreArgs {
    /// Input Markdown file
    pub file: String,

    /// Emit AST JSON (pretty)
    #[arg(short, long)]
    pub pretty: bool,
}

pub fn run(args: &CoreArgs) {
    let raw = fs::read_to_string(&args.file).unwrap_or_else(|e| {
        eprintln!("markplus core: cannot read {}: {e}", args.file);
        process::exit(1);
    });

    let asset = parse_document(&raw).unwrap_or_else(|e| {
        eprintln!("markplus core: parse error: {e}");
        process::exit(1);
    });

    let json = if args.pretty {
        asset.to_json_pretty()
    } else {
        asset.to_json()
    }
    .unwrap_or_else(|e| {
        eprintln!("markplus core: serialisation error: {e}");
        process::exit(1);
    });

    println!("{json}");
}
