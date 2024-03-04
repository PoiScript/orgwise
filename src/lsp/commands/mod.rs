mod headline_toc;
pub mod src_block_detangle;
pub mod src_block_execute;
pub mod src_block_tangle;

use lsp_types::*;
use orgize::rowan::ast::AstNode;
use serde_json::{json, Value};

use super::{FileSystem, LanguageClient, LanguageServerBase, Process};

pub enum OrgwiseCommand {
    SrcBlockExecute { url: Url, block_offset: u32 },

    SrcBlockTangle { url: Url, block_offset: u32 },

    SrcBlockDetangle { url: Url, block_offset: u32 },

    HeadlineToc { url: Url, heading_offset: u32 },
}

impl From<OrgwiseCommand> for Command {
    fn from(val: OrgwiseCommand) -> Self {
        match val {
            OrgwiseCommand::SrcBlockExecute { url, block_offset } => Command {
                command: "orgwise.src-block.execute".into(),
                arguments: Some(vec![json!(url), json!(block_offset)]),
                title: "Execute".into(),
            },
            OrgwiseCommand::SrcBlockTangle { url, block_offset } => Command {
                command: "orgwise.src-block.tangle".into(),
                arguments: Some(vec![json!(url), json!(block_offset)]),
                title: "Tangle".into(),
            },
            OrgwiseCommand::SrcBlockDetangle { url, block_offset } => Command {
                command: "orgwise.src-block.detangle".into(),
                arguments: Some(vec![json!(url), json!(block_offset)]),
                title: "Detangle".into(),
            },
            OrgwiseCommand::HeadlineToc {
                url,
                heading_offset,
            } => Command {
                command: "orgwise.headline.toc".into(),
                arguments: Some(vec![json!(url), json!(heading_offset)]),
                title: "Generate TOC".into(),
            },
        }
    }
}

impl OrgwiseCommand {
    pub fn all() -> Vec<String> {
        vec![
            "orgwise.src-block.execute".into(),
            "orgwise.src-block.tangle".into(),
            "orgwise.src-block.detangle".into(),
            "orgwise.headline.toc".into(),
        ]
    }
}

impl<E> LanguageServerBase<E>
where
    E: FileSystem<Location = Url> + LanguageClient + Process,
{
    pub async fn execute_command(&self, params: ExecuteCommandParams) -> Option<Value> {
        match self
            .execute_command_inner(&params.command, params.arguments)
            .await
        {
            Ok(value) => Some(value),
            Err(err) => {
                self.env
                    .show_message(
                        MessageType::ERROR,
                        format!("Failed to execute `{}`: {}", params.command, err),
                    )
                    .await;
                None
            }
        }
    }

    async fn execute_command_inner(
        &self,
        command: &str,
        mut arguments: Vec<Value>,
    ) -> anyhow::Result<Value> {
        if command == "orgwise.search-headline" {
            if let Some(value) = arguments.pop() {
                let mut results = vec![];
                let option = serde_json::from_value(value)?;
                for item in self.documents.iter() {
                    results.append(&mut crate::common::search_heading::search(
                        &option,
                        &item.value().org,
                    ));
                }
                return Ok(serde_json::to_value(results)?);
            }
        }

        let Some(url) = arguments
            .pop()
            .and_then(|x| x.as_str().and_then(|s| Url::parse(s).ok()))
        else {
            return Ok(Value::Bool(false));
        };

        match (command, arguments.pop()) {
            ("orgwise.src-block.tangle", Some(n)) => {
                let offset: u32 = serde_json::from_value(n)?;
                self.src_block_tangle(url, offset).await?;
                Ok(Value::Bool(true))
            }
            ("orgwise.src-block.detangle", Some(n)) => {
                let offset: u32 = serde_json::from_value(n)?;
                self.src_block_detangle(url, offset).await?;
                Ok(Value::Bool(true))
            }
            ("orgwise.src-block.execute", Some(n)) => {
                let offset: u32 = serde_json::from_value(n)?;
                self.src_block_execute(url, offset).await?;
                Ok(Value::Bool(true))
            }
            ("orgwise.headline.toc", Some(n)) => {
                let offset: u32 = serde_json::from_value(n)?;
                self.headline_toc(url, offset).await;
                Ok(Value::Bool(true))
            }
            ("orgwise.syntax-tree", _) => {
                if let Some(doc) = self.documents.get(&url) {
                    Ok(Value::String(format!("{:#?}", doc.org.document().syntax())))
                } else {
                    Ok(Value::Bool(true))
                }
            }
            ("orgwise.preview-html", _) => {
                if let Some(doc) = self.documents.get(&url) {
                    Ok(Value::String(format!("{}", doc.org.to_html())))
                } else {
                    Ok(Value::Bool(true))
                }
            }

            _ => Ok(Value::Bool(false)),
        }
    }
}
