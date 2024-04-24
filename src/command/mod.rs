pub mod clocking;
pub mod formatting;
pub mod headline;
pub mod src_block;

use lsp_types::*;
use orgize::rowan::ast::AstNode;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;

use crate::backend::Backend;

pub trait Executable: DeserializeOwned {
    const NAME: &'static str;

    const TITLE: Option<&'static str> = None;

    type Result: Serialize;

    async fn execute<B: Backend>(self, backend: &B) -> anyhow::Result<Self::Result>;
}

macro_rules! command {
    ($( $i:ident, )*) => {
        #[derive(Deserialize)]
        #[serde(tag = "command", content = "argument", rename_all = "kebab-case")]
        pub enum OrgwiseCommand {
            $( $i($i) ),*
        }

        impl OrgwiseCommand {
            pub async fn execute<B: Backend>(self, backend: &B) -> anyhow::Result<Value> {
                match self {
                    $(
                        OrgwiseCommand::$i(i) => Ok(serde_json::to_value(i.execute(backend).await?)?)
                    ),*
                }
            }

            #[cfg(feature="tower")]
            pub async fn execute_response<B: Backend>(self, backend: &B) -> anyhow::Result<axum::response::Response> {
                use axum::{response::IntoResponse, Json};
                match self {
                    $(
                        OrgwiseCommand::$i(i) => Ok(Json(i.execute(backend).await?).into_response())
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

            pub fn from_value(name: &str, argument: Value) -> Option<Self> {
                match name {
                    $(
                        $i::NAME => {
                            Some(OrgwiseCommand::$i(
                                serde_json::from_value(argument).ok()?
                            ))
                        }
                    ),*
                    _ => None
                }
            }
        }

        $(
            impl Into<Command> for $i {
                fn into(self) -> Command {
                    Command {
                        title: $i::TITLE.unwrap_or($i::NAME).to_string(),
                        command: format!("orgwise.{}", $i::NAME),
                        arguments: Some(vec![serde_json::to_value(self).unwrap()]),
                    }
                }
            }
        )*
    };
}

#[derive(Deserialize, Serialize)]
pub struct SyntaxTree(Url);

impl Executable for SyntaxTree {
    const NAME: &'static str = "syntax-tree";

    type Result = Option<String>;

    async fn execute<B: Backend>(self, backend: &B) -> anyhow::Result<Option<String>> {
        Ok(backend
            .documents()
            .get_map(&self.0, |doc| format!("{:#?}", doc.org.document().syntax())))
    }
}

#[derive(Deserialize, Serialize)]
pub struct PreviewHtml(Url);

impl Executable for PreviewHtml {
    const NAME: &'static str = "preview-html";

    type Result = Option<String>;

    async fn execute<B: Backend>(self, backend: &B) -> anyhow::Result<Option<String>> {
        Ok(backend
            .documents()
            .get_map(&self.0, |doc| doc.org.to_html()))
    }
}

pub use clocking::{ClockingStart, ClockingStatus, ClockingStop};
pub use headline::{
    HeadlineCreate, HeadlineDuplicate, HeadlineGenerateToc, HeadlineRemove, HeadlineSearch,
    HeadlineUpdate,
};
pub use src_block::{
    SrcBlockDetangle, SrcBlockDetangleAll, SrcBlockExecute, SrcBlockExecuteAll, SrcBlockTangle,
    SrcBlockTangleAll,
};

command!(
    PreviewHtml,
    SyntaxTree,
    ClockingStart,
    ClockingStatus,
    ClockingStop,
    HeadlineCreate,
    HeadlineDuplicate,
    HeadlineGenerateToc,
    HeadlineRemove,
    HeadlineSearch,
    HeadlineUpdate,
    SrcBlockDetangle,
    SrcBlockDetangleAll,
    SrcBlockExecute,
    SrcBlockExecuteAll,
    SrcBlockTangle,
    SrcBlockTangleAll,
);
