use clap::Args;
use std::path::PathBuf;

use crate::command::{Executable, SrcBlockDetangleAll, SrcBlockExecuteAll, SrcBlockTangleAll};

use super::environment::CliServer;

#[derive(Debug, Args)]
pub struct DetangleCommand {
    path: Vec<PathBuf>,

    #[arg(short, long)]
    dry_run: bool,
}

impl DetangleCommand {
    pub async fn run(self) -> anyhow::Result<()> {
        let base = CliServer::new(self.dry_run);

        for path in self.path {
            if let Some(url) = base.load_org_file(&path) {
                SrcBlockDetangleAll { url }.execute(&base).await?;
            }
        }

        Ok(())
    }
}

#[derive(Debug, Args)]
pub struct ExecuteCommand {
    path: Vec<PathBuf>,

    #[arg(short, long)]
    dry_run: bool,
}

impl ExecuteCommand {
    pub async fn run(self) -> anyhow::Result<()> {
        let base = CliServer::new(self.dry_run);
        for path in self.path {
            if let Some(url) = base.load_org_file(&path) {
                SrcBlockExecuteAll { url }.execute(&base).await?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Args)]
pub struct TangleCommand {
    path: Vec<PathBuf>,

    #[arg(short, long)]
    dry_run: bool,
}

impl TangleCommand {
    pub async fn run(self) -> anyhow::Result<()> {
        let base = CliServer::new(self.dry_run);
        for path in self.path {
            if let Some(url) = base.load_org_file(&path) {
                SrcBlockTangleAll { url }.execute(&base).await?;
            }
        }
        Ok(())
    }
}
