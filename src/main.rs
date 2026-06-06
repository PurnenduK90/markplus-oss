mod core_cmd;

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
    /// MarkPlus Core CLI operations
    Core(core_cmd::CoreArgs),
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Core(args) => core_cmd::run(args),
    }
}
