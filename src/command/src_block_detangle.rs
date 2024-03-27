use lsp_types::*;
use orgize::rowan::TextSize;
use orgize::{ast::Headline, rowan::TextRange, SyntaxKind};
use orgize::{ast::SourceBlock, rowan::ast::AstNode};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::base::Server;
use crate::command::utils::collect_src_blocks;

use super::utils::{
    find_block, header_argument, language_comments, property_drawer, property_keyword,
};
use super::Executable;

#[derive(Deserialize, Serialize)]
pub struct SrcBlockDetangle {
    pub url: Url,
    #[serde(with = "crate::command::utils::text_size")]
    pub block_offset: TextSize,
}

impl Executable for SrcBlockDetangle {
    const NAME: &'static str = "src-block-detangle";

    const TITLE: Option<&'static str> = Some("Detangle");

    async fn execute<S: Server>(self, server: &S) -> anyhow::Result<Value> {
        let Some(doc) = server.documents().get(&self.url) else {
            return Ok(Value::Null);
        };

        let Some(block) = find_block(&doc, self.block_offset) else {
            return Ok(Value::Null);
        };

        let Some(option) = DetangleOptions::new(block, &self.url, server) else {
            server
                .log_message(
                    MessageType::WARNING,
                    "Code block can't be detangled.".into(),
                )
                .await;

            return Ok(Value::Null);
        };

        let (text_range, new_text) = option.run(server).await?;

        drop(doc);

        server.apply_edit(self.url, new_text, text_range).await?;

        Ok(Value::Bool(true))
    }
}

#[derive(Deserialize, Serialize)]
pub struct SrcBlockDetangleAll {
    pub url: Url,
}

impl Executable for SrcBlockDetangleAll {
    const NAME: &'static str = "src-block-detangle-all";

    const TITLE: Option<&'static str> = Some("Detangle all source blocks");

    async fn execute<S: Server>(self, server: &S) -> anyhow::Result<Value> {
        let Some(doc) = server.documents().get(&self.url) else {
            return Ok(Value::Null);
        };

        let blocks = collect_src_blocks(&doc.org);
        let options: Vec<_> = blocks
            .into_iter()
            .filter_map(|block| DetangleOptions::new(block, &self.url, server))
            .collect();

        let mut edits = Vec::with_capacity(options.len());

        for option in options {
            let (range, content) = option.run(server).await?;
            edits.push((self.url.clone(), content, range));
        }

        drop(doc);

        server.apply_edits(edits.into_iter()).await?;

        Ok(Value::Bool(true))
    }
}

pub struct DetangleOptions {
    destination: Url,
    comment_link: Option<(String, String)>,
    text_range: TextRange,
}

impl DetangleOptions {
    pub fn new<S: Server>(block: SourceBlock, base: &Url, server: &S) -> Option<Self> {
        let arg1 = block.parameters().unwrap_or_default();
        let arg2 = property_drawer(block.syntax()).unwrap_or_default();
        let arg3 = property_keyword(block.syntax()).unwrap_or_default();
        let language = block.language().unwrap_or_default();

        let tangle = header_argument(&arg1, &arg2, &arg3, ":tangle", "no");

        if tangle == "no" {
            return None;
        }

        let text_range = block
            .syntax()
            .children()
            .find(|n| n.kind() == SyntaxKind::BLOCK_CONTENT)
            .unwrap()
            .text_range();

        let comments = header_argument(&arg1, &arg2, &arg3, ":comments", "no");

        let destination = server.resolve_in(tangle, base).ok()?;

        let mut comment_link = None;
        if comments == "yes" || comments == "link" || comments == "noweb" || comments == "both" {
            let parent = block
                .syntax()
                .ancestors()
                .find(|n| n.kind() == SyntaxKind::HEADLINE || n.kind() == SyntaxKind::DOCUMENT);

            let nth = parent
                .as_ref()
                .and_then(|n| n.children().position(|c| &c == block.syntax()))
                .unwrap_or(1);

            let title = parent
                .and_then(Headline::cast)
                .map(|headline| headline.title_raw())
                .unwrap_or_else(|| "No heading".to_string());

            if let Some((l, r)) = language_comments(&language) {
                comment_link = Some((
                    format!("{l} [[{}::*{title}][{title}:{nth}]] {r}", destination),
                    format!("{l} {title}:{nth} ends here {r}"),
                ));
            }
        }

        Some(DetangleOptions {
            destination,
            comment_link,
            text_range,
        })
    }

    pub async fn run<S: Server>(self, server: &S) -> anyhow::Result<(TextRange, String)> {
        let content = server.read_to_string(&self.destination).await?;

        if let Some((begin, end)) = &self.comment_link {
            let mut block_content = String::new();

            for line in content
                .lines()
                .skip_while(|line| line.trim() != begin)
                .skip(1)
            {
                if line.trim() == end {
                    break;
                } else {
                    block_content += line;
                    block_content += "\n";
                }
            }

            Ok((self.text_range, block_content))
        } else {
            Ok((self.text_range, content))
        }
    }
}
