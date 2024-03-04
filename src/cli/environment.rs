use std::path::PathBuf;

use crate::lsp::{FileSystem, Process};

pub struct TokioEnvironment;

impl FileSystem for TokioEnvironment {
    type Location = PathBuf;

    async fn write(&self, path: &PathBuf, content: &str) -> anyhow::Result<()> {
        tokio::fs::write(path, content).await?;
        Ok(())
    }

    async fn read_to_string(&self, path: &PathBuf) -> anyhow::Result<String> {
        let content = tokio::fs::read_to_string(path).await?;
        Ok(content)
    }

    fn display(&self, path: &Self::Location) -> impl std::fmt::Display {
        path.display()
    }

    fn resolve_in(&self, path: &str, base: &Self::Location) -> anyhow::Result<Self::Location> {
        todo!()
    }
}

impl Process for TokioEnvironment {
    async fn execute(&self, executable: &str, content: &str) -> anyhow::Result<String> {
        let dir = tempfile::tempdir()?;

        let path = dir.path().join(".orgize");

        tokio::fs::write(&path, content).await?;

        let output = tokio::process::Command::new(executable)
            .arg(&path)
            .output()
            .await?;

        let output = String::from_utf8_lossy(&output.stdout);

        Ok(output.to_string())
    }
}
