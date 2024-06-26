use lsp_types::*;
use orgize::rowan::TextSize;
use orgize::{ast::Headline, rowan::TextRange, SyntaxKind};
use orgize::{ast::SourceBlock, rowan::ast::AstNode};
use serde::{Deserialize, Serialize};

use crate::backend::Backend;

use crate::command::Executable;
use crate::utils::src_block::{
    collect_src_blocks, header_argument, language_comments, property_drawer, property_keyword,
};

#[derive(Deserialize, Serialize)]
pub struct SrcBlockDetangle {
    pub url: Url,
    #[serde(with = "crate::utils::text_size")]
    pub block_offset: TextSize,
}

impl Executable for SrcBlockDetangle {
    const NAME: &'static str = "src-block-detangle";

    const TITLE: Option<&'static str> = Some("Detangle");

    type Result = bool;

    async fn execute<B: Backend>(self, backend: &B) -> anyhow::Result<bool> {
        let Some(block) = backend
            .documents()
            .get_and_then(&self.url, |doc| doc.org.node_at_offset(self.block_offset))
        else {
            return Ok(false);
        };

        let Some(option) = DetangleOptions::new(block, &self.url, backend) else {
            backend
                .log_message(
                    MessageType::WARNING,
                    "Code block can't be detangled.".into(),
                )
                .await;

            return Ok(false);
        };

        let (text_range, new_text) = option.run(backend).await?;

        backend.apply_edit(self.url, new_text, text_range).await?;

        Ok(true)
    }
}

#[derive(Deserialize, Serialize)]
pub struct SrcBlockDetangleAll {
    pub url: Url,
}

impl Executable for SrcBlockDetangleAll {
    const NAME: &'static str = "src-block-detangle-all";

    const TITLE: Option<&'static str> = Some("Detangle all source blocks");

    type Result = bool;

    async fn execute<B: Backend>(self, backend: &B) -> anyhow::Result<bool> {
        let Some(blocks) = backend
            .documents()
            .get_map(&self.url, |doc| collect_src_blocks(&doc.org))
        else {
            return Ok(false);
        };

        let options: Vec<_> = blocks
            .into_iter()
            .filter_map(|block| DetangleOptions::new(block, &self.url, backend))
            .collect();

        let mut edits = Vec::with_capacity(options.len());

        for option in options {
            let (range, content) = option.run(backend).await?;
            edits.push((self.url.clone(), content, range));
        }

        backend.apply_edits(edits.into_iter()).await?;

        Ok(true)
    }
}

pub struct DetangleOptions {
    destination: Url,
    comment_link: Option<(String, String)>,
    text_range: TextRange,
}

impl DetangleOptions {
    pub fn new<B: Backend>(block: SourceBlock, base: &Url, backend: &B) -> Option<Self> {
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

        let destination = backend.resolve_in(tangle, base).ok()?;

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

    pub async fn run<B: Backend>(self, backend: &B) -> anyhow::Result<(TextRange, String)> {
        let content = backend.read_to_string(&self.destination).await?;

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
