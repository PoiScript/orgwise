use clap::Args;
use std::path::PathBuf;

use super::environment::CliServer;
use crate::base::Server;
use crate::command::formatting;

#[derive(Debug, Args)]
pub struct Command {
    path: Vec<PathBuf>,

    #[arg(short, long)]
    dry_run: bool,
}

impl Command {
    pub async fn run(self) -> anyhow::Result<()> {
        let base = CliServer::new(self.dry_run);

        for path in self.path {
            if let Some(url) = base.load_org_file(&path) {
                let doc = base.documents().get(&url).unwrap();

                let edits = formatting::formatting(&doc.org);

                base.apply_edits(
                    edits
                        .into_iter()
                        .map(|(range, content)| (url.clone(), content, range)),
                )
                .await?;
            }
        }

        Ok(())
    }
}
