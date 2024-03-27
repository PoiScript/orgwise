mod base;
mod cli;
mod command;
mod lsp;

#[cfg(test)]
mod test;

use clap::{
    builder::styling::{AnsiColor, Color, Style},
    Parser, Subcommand,
};
use clap_verbosity_flag::{InfoLevel, Verbosity};
use log::{Level, LevelFilter, Log, Metadata, Record};

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
    ApiServer(cli::api_server::Command),

    /// Start language server
    #[clap(name = "lsp")]
    LanguageServer,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let parsed = App::parse();

    let level = parsed.verbose.log_level_filter();
    log::set_boxed_logger(Box::new(Logger { level })).unwrap();
    log::set_max_level(level);

    match parsed.command {
        Command::Tangle(cmd) => cmd.run().await,
        Command::Detangle(cmd) => cmd.run().await,
        Command::ExecuteSrcBlock(cmd) => cmd.run().await,
        Command::Format(cmd) => cmd.run().await,
        Command::ApiServer(cmd) => cmd.run().await,
        Command::LanguageServer => cli::lsp_server::start().await,
    }
}

struct Logger {
    level: LevelFilter,
}

impl Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level().to_level_filter() <= self.level
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let color = match record.level() {
                Level::Error => AnsiColor::Red,
                Level::Warn => AnsiColor::Yellow,
                Level::Info => AnsiColor::Cyan,
                Level::Debug => AnsiColor::Green,
                Level::Trace => AnsiColor::White,
            };

            let style = Style::new().fg_color(Color::Ansi(color).into());

            println!(
                "{}{:<5}{} {}",
                style.render(),
                &record.level(),
                style.render_reset(),
                record.args()
            )
        }
    }

    fn flush(&self) {}
}
