use clap::builder::styling::{AnsiColor, Color, Style};
use dashmap::{DashMap, RwLock};
use lsp_types::{MessageType, Url};
use orgize::{rowan::TextRange, ParseConfig};
use std::{collections::HashMap, fs, path::Path};

use crate::base::{OrgDocument, Server};

pub struct CliServer {
    dry_run: bool,
    documents: DashMap<Url, OrgDocument>,
    parse_config: RwLock<ParseConfig>,
}

impl CliServer {
    pub fn new(dry_run: bool) -> Self {
        CliServer {
            documents: DashMap::new(),
            dry_run,
            parse_config: RwLock::new(ParseConfig::default()),
        }
    }

    pub fn load_org_file(&self, path: &Path) -> Option<Url> {
        if !path.exists() {
            log::error!("{} is not existed", path.display());
            return None;
        }

        let path = match fs::canonicalize(path) {
            Ok(path) => path,
            Err(err) => {
                log::error!("failed to resolve {}: {err:?}", path.display());
                return None;
            }
        };

        let Ok(url) = Url::from_file_path(&path) else {
            log::error!("failed to parse {}", path.display());
            return None;
        };

        let content = match fs::read_to_string(&path) {
            Ok(content) => content,
            Err(err) => {
                log::error!("failed to read {}: {err:?}", path.display());
                return None;
            }
        };

        self.add_doc(url.clone(), content);

        Some(url)
    }
}

impl Server for CliServer {
    fn home_dir(&self) -> Option<Url> {
        dirs::home_dir().and_then(|d| Url::from_file_path(d).ok())
    }

    async fn log_message(&self, typ: MessageType, message: String) {
        self.show_message(typ, message).await
    }

    async fn show_message(&self, typ: MessageType, message: String) {
        match typ {
            MessageType::ERROR => log::error!("{}", message),
            MessageType::WARNING => log::warn!("{}", message),
            MessageType::INFO => log::info!("{}", message),
            MessageType::LOG => log::debug!("{}", message),
            _ => {}
        }
    }

    async fn apply_edits(
        &self,
        items: impl Iterator<Item = (Url, String, TextRange)>,
    ) -> anyhow::Result<()> {
        let mut changes: HashMap<Url, Vec<(TextRange, String)>> = HashMap::new();

        for (url, new_text, text_range) in items {
            if let Some(edits) = changes.get_mut(&url) {
                edits.push((text_range, new_text))
            } else {
                changes.insert(url.clone(), vec![(text_range, new_text)]);
            }
        }

        for (url, edits) in changes.iter_mut() {
            let Ok(path) = url.to_file_path() else {
                anyhow::bail!("Cannot convert Url to PathBuf")
            };

            edits.sort_by(|a, b| a.0.start().cmp(&b.0.start()));

            let input = tokio::fs::read_to_string(&path).await?;
            let mut output = String::with_capacity(input.len());
            let mut off = 0;

            for (range, content) in edits {
                let start = range.start().into();
                let end = range.end().into();

                if self.dry_run {
                    print!("{}", &input[off..start]);

                    if &input[start..end] != content {
                        let style = Style::new().fg_color(Color::Ansi(AnsiColor::Cyan).into());
                        print!("{}{}{}", style.render(), &content, style.render_reset());
                    } else {
                        print!("{}", &content);
                    }
                } else {
                    output += &input[off..start];
                    output += &content;
                }

                off = end;
            }

            if self.dry_run {
                print!("{}", &input[off..]);
            } else {
                output += &input[off..];
                tokio::fs::write(&path, &output).await?;
                self.update_doc(url.clone(), None, output);
            }
        }

        Ok(())
    }

    async fn write(&self, url: &Url, content: &str) -> anyhow::Result<()> {
        if let Ok(path) = url.to_file_path() {
            tokio::fs::write(path, content).await?;
            Ok(())
        } else {
            anyhow::bail!("Cannot convert Url to PathBuf")
        }
    }

    async fn read_to_string(&self, url: &Url) -> anyhow::Result<String> {
        if let Ok(path) = url.to_file_path() {
            Ok(tokio::fs::read_to_string(path).await?)
        } else {
            anyhow::bail!("Cannot convert Url to PathBuf")
        }
    }

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

    fn documents(&self) -> &DashMap<Url, OrgDocument> {
        &self.documents
    }

    fn default_parse_config(&self) -> ParseConfig {
        self.parse_config.read().clone()
    }

    fn set_default_parse_config(&self, config: ParseConfig) {
        *self.parse_config.write() = config;
    }
}
