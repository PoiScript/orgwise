mod api;
mod cli;
mod common;
mod lsp;

use std::path::PathBuf;

use clap::{Parser, Subcommand};
use clap_verbosity_flag::{InfoLevel, LevelFilter as CLevelFilter, Verbosity};
use tracing::level_filters::LevelFilter;

/// Command line utility for org-mode files
#[derive(Debug, Parser)]
#[clap(name = "orgwise", version)]
pub struct App {
    #[clap(subcommand)]
    command: Command,

    #[command(flatten)]
    verbose: Verbosity<InfoLevel>,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Tangle source block contents to destination files
    #[clap(name = "tangle")]
    Tangle(cli::src_block::TangleCommand),

    /// Insert tangled file contents back to source files
    #[clap(name = "detangle")]
    Detangle(cli::src_block::DetangleCommand),

    /// Execute source block
    #[clap(name = "execute-src-block")]
    ExecuteSrcBlock(cli::src_block::ExecuteCommand),

    /// Format org-mode files
    #[clap(name = "fmt")]
    Format(cli::fmt::Command),

    /// Start api server
    #[clap(name = "api")]
    ApiServer { path: Vec<PathBuf> },

    /// Start language server
    #[clap(name = "lsp")]
    LanguageServer,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let parsed = App::parse();

    tracing_subscriber::fmt()
        .with_max_level(match parsed.verbose.log_level_filter() {
            CLevelFilter::Off => LevelFilter::OFF,
            CLevelFilter::Error => LevelFilter::ERROR,
            CLevelFilter::Warn => LevelFilter::WARN,
            CLevelFilter::Info => LevelFilter::INFO,
            CLevelFilter::Debug => LevelFilter::DEBUG,
            CLevelFilter::Trace => LevelFilter::TRACE,
        })
        .without_time()
        .with_file(false)
        .with_line_number(false)
        .init();

    match parsed.command {
        Command::Tangle(cmd) => cmd.run().await,
        Command::Detangle(cmd) => cmd.run().await,
        Command::ExecuteSrcBlock(cmd) => cmd.run().await,
        Command::Format(cmd) => cmd.run().await,
        Command::ApiServer { path } => api::start(path).await,
        Command::LanguageServer => {
            cli::lsp::start().await;
            Ok(())
        }
    }
}
