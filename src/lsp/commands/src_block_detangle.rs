use std::collections::HashMap;
use std::marker::PhantomData;

use lsp_types::*;
use orgize::{ast::Headline, rowan::TextRange, SyntaxKind};
use orgize::{ast::SourceBlock, rowan::ast::AstNode};

use crate::common::{
    header_argument::{header_argument, property_drawer, property_keyword},
    utils::language_comments,
};

use super::LanguageServerBase;
use super::{FileSystem, LanguageClient, Process};

impl<E> LanguageServerBase<E>
where
    E: FileSystem<Location = Url> + LanguageClient + Process,
{
    pub async fn src_block_detangle(&self, url: Url, block_offset: u32) -> anyhow::Result<()> {
        let Some(doc) = self.documents.get(&url) else {
            return Ok(());
        };

        let Some(block) = doc
            .org
            .document()
            .syntax()
            .descendants()
            .filter_map(SourceBlock::cast)
            .find(|n| n.begin() == block_offset)
        else {
            return Ok(());
        };

        let Some(option) = DetangleOptions::new(block, &url, &self.env) else {
            self.env
                .show_message(
                    MessageType::WARNING,
                    "Code block can't be detangled.".into(),
                )
                .await;
            return Ok(());
        };

        let (range, new_text) = option.run(&self.env).await?;

        let mut changes = HashMap::new();

        let range = doc.range_of(range);

        changes.insert(url, vec![TextEdit { new_text, range }]);

        let _ = self
            .env
            .apply_edit(WorkspaceEdit {
                changes: Some(changes),
                ..Default::default()
            })
            .await;

        Ok(())
    }
}

pub struct DetangleOptions<E>
where
    E: FileSystem,
{
    destination: E::Location,
    comment_link: Option<(String, String)>,
    text_range: TextRange,

    env: PhantomData<E>,
}

impl<E> DetangleOptions<E>
where
    E: FileSystem,
{
    pub fn new(block: SourceBlock, base: &E::Location, env: &E) -> Option<Self> {
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

        let destination = env.resolve_in(&tangle, base).ok()?;

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
                    format!(
                        "{l} [[{}::*{title}][{title}:{nth}]] {r}",
                        env.display(&destination)
                    ),
                    format!("{l} {title}:{nth} ends here {r}"),
                ));
            }
        }

        Some(DetangleOptions {
            destination,
            comment_link,
            text_range,
            env: PhantomData,
        })
    }

    pub async fn run(self, env: &E) -> anyhow::Result<(TextRange, String)> {
        let content = env.read_to_string(&self.destination).await?;

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
