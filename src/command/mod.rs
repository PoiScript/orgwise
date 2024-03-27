mod clocking;
mod create_headline;
mod duplicate_headline;
pub mod formatting;
mod headline_toc;
mod remove_headline;
mod search_heading;
mod src_block_detangle;
mod src_block_execute;
mod src_block_tangle;
mod update_headline;

pub mod utils;

use lsp_types::*;
use orgize::rowan::ast::AstNode;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;

use crate::base::Server;

pub(crate) trait Executable: DeserializeOwned {
    const NAME: &'static str;

    const TITLE: Option<&'static str> = None;

    async fn execute<S: Server>(self, server: &S) -> anyhow::Result<Value>;
}

macro_rules! command {
    ($( $i:ident, )*) => {
        #[derive(Deserialize)]
        #[serde(tag = "command", content = "argument", rename_all = "kebab-case")]
        pub enum OrgwiseCommand {
            $( $i($i) ),*
        }

        impl OrgwiseCommand {
            pub async fn execute<S: Server>(self, server: &S) -> anyhow::Result<Value> {
                match self {
                    $(
                        OrgwiseCommand::$i(i) => Ok(i.execute(server).await?)
                    ),*
                }
            }

            pub fn all() -> Vec<String> {
                vec![
                    $(
                        format!("orgwise.{}", $i::NAME)
                    ),*
                ]
            }
        }

        impl<'a> TryFrom<(&'a str, Vec<Value>)> for OrgwiseCommand {
            type Error = anyhow::Error;

            fn try_from(value: (&'a str, Vec<Value>)) -> Result<Self, Self::Error> {
                let (name, mut arguments) = value;
                let Some(tag) = name.strip_prefix("orgwise.") else {
                    anyhow::bail!("");
                };
                let Some(argument) = arguments.pop() else {
                    anyhow::bail!("");
                };
                match tag {
                    $(
                        $i::NAME => {
                            Ok(OrgwiseCommand::$i(
                                serde_json::from_value(argument)?
                            ))
                        }
                    ),*
                    _ => {
                        anyhow::bail!("")
                    }
                }
            }
        }

        impl Into<Command> for OrgwiseCommand {
            fn into(self) -> Command {
                match self {
                    $(
                        OrgwiseCommand::$i(i) => {
                            Command {
                                title: $i::TITLE.unwrap_or($i::NAME).to_string(),
                                command: format!("orgwise.{}", $i::NAME),
                                arguments: Some(vec![serde_json::to_value(i).unwrap()]),
                            }
                        }
                    ),*
                }
            }
        }

        $(
            impl From<$i> for OrgwiseCommand {
                fn from(cmd: $i) -> OrgwiseCommand {
                    OrgwiseCommand::$i(cmd)
                }
            }

            impl Into<Command> for $i {
                fn into(self) -> Command {
                    OrgwiseCommand::$i(self).into()
                }
            }
        )*
    };
}

#[derive(Deserialize, Serialize)]
pub struct SyntaxTree(Url);

impl Executable for SyntaxTree {
    const NAME: &'static str = "syntax-tree";

    async fn execute<S: Server>(self, server: &S) -> anyhow::Result<Value> {
        match server.documents().get(&self.0) {
            Some(doc) => Ok(Value::String(format!("{:#?}", doc.org.document().syntax()))),
            None => Ok(Value::Null),
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct PreviewHtml(Url);

impl Executable for PreviewHtml {
    const NAME: &'static str = "preview-html";

    async fn execute<S: Server>(self, server: &S) -> anyhow::Result<Value> {
        match server.documents().get(&self.0) {
            Some(doc) => Ok(Value::String(doc.org.to_html())),
            None => Ok(Value::Null),
        }
    }
}

pub use clocking::ClockingStatus;
pub use create_headline::CreateHeadline;
pub use duplicate_headline::DuplicateHeadline;
pub use headline_toc::HeadlineToc;
pub use remove_headline::RemoveHeadline;
pub use search_heading::SearchHeadline;
pub use src_block_detangle::{SrcBlockDetangle, SrcBlockDetangleAll};
pub use src_block_execute::{SrcBlockExecute, SrcBlockExecuteAll};
pub use src_block_tangle::{SrcBlockTangle, SrcBlockTangleAll};
pub use update_headline::UpdateHeadline;

command!(
    SearchHeadline,
    UpdateHeadline,
    RemoveHeadline,
    CreateHeadline,
    DuplicateHeadline,
    ClockingStatus,
    HeadlineToc,
    PreviewHtml,
    SyntaxTree,
    SrcBlockDetangle,
    SrcBlockDetangleAll,
    SrcBlockExecute,
    SrcBlockExecuteAll,
    SrcBlockTangleAll,
    SrcBlockTangle,
);
