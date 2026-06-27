//! MarkPlus CLI Workspace
//!
//! This crate provides the main command-line interface for compiling, validating, and rendering
//! MarkPlus assets.
mod compile_cmd;
mod core_cmd;
mod render_cmd;

use clap::{Parser, Subcommand};

/// MarkPlus Community CLI
///
/// The community version of everything MarkPlus has to offer.
#[derive(Parser)]
#[command(name = "markplus-oss")]
#[command(about = "MarkPlus Community CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Parse a Markdown file and emit its AST as JSON
    Core(core_cmd::CoreArgs),
    /// Render a Markdown file to HTML, Typst source, or PDF
    Render(render_cmd::RenderArgs),
    /// Parse and render in one step — output format inferred from file extension
    /// (.json → AST, .html → HTML, .typ → Typst source, .pdf → PDF)
    Compile(compile_cmd::CompileArgs),
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Core(args) => core_cmd::run(args),
        Commands::Render(args) => render_cmd::run(args),
        Commands::Compile(args) => compile_cmd::run(args),
    }
}
