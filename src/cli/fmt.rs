use clap::Args;
use orgize::Org;
use std::path::PathBuf;

use crate::cli::diff;

#[derive(Debug, Args)]
pub struct Command {
    path: Vec<PathBuf>,

    #[arg(short, long)]
    dry_run: bool,
}

impl Command {
    pub async fn run(self) -> anyhow::Result<()> {
        for path in self.path {
            if !path.exists() {
                tracing::error!("{:?} is not existed", path);

                let input = tokio::fs::read_to_string(&path).await?;

                let org = Org::parse(&input);

                let patches = crate::common::formatting(&org);

                if self.dry_run {
                    diff::print(&input, patches);
                } else {
                    diff::write_to_file(&input, patches, path)?;
                }
            }
        }

        Ok(())
    }
}
