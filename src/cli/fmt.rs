use clap::Args;
use std::path::PathBuf;

use super::environment::CliBackend;
use crate::backend::Backend;
use crate::command::formatting;

#[derive(Debug, Args)]
pub struct Command {
    path: Vec<PathBuf>,

    #[arg(short, long)]
    dry_run: bool,
}

impl Command {
    pub async fn run(self) -> anyhow::Result<()> {
        let backend = CliBackend::new(self.dry_run);

        for path in self.path {
            if let Some(url) = backend.load_org_file(&path) {
                let doc = backend.documents().get(&url).unwrap();

                let edits = formatting::formatting(&doc.org);

                backend
                    .apply_edits(
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
