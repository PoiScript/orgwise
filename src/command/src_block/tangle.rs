// TODO: :noweb support

use lsp_types::*;
use memchr::memchr2_iter;
use orgize::ast::Headline;
use orgize::rowan::{Direction, TextSize};
use orgize::SyntaxKind;
use orgize::{ast::SourceBlock, rowan::ast::AstNode};
use serde::{Deserialize, Serialize};
use std::fmt::Write;

use crate::backend::Backend;

use crate::command::Executable;
use crate::utils::src_block::{
    collect_src_blocks, header_argument, language_comments, property_drawer, property_keyword,
};

#[derive(Serialize, Deserialize)]
pub struct SrcBlockTangle {
    pub url: Url,
    #[serde(with = "crate::utils::text_size")]
    pub block_offset: TextSize,
}

#[derive(Serialize, Deserialize)]
pub struct SrcBlockTangleAll {
    pub url: Url,
}

impl Executable for SrcBlockTangleAll {
    const NAME: &'static str = "src-block-tangle-all";

    const TITLE: Option<&'static str> = Some("Tangle all source blocks");

    type Result = bool;

    async fn execute<B: Backend>(self, backend: &B) -> anyhow::Result<bool> {
        let Some(blocks) = backend
            .documents()
            .get_map(&self.url, |doc| collect_src_blocks(&doc.org))
        else {
            return Ok(false);
        };

        let mut i = 0;

        let options: Vec<_> = blocks
            .into_iter()
            .filter_map(|block| TangleOptions::new(block, &self.url, backend))
            .collect();

        for option in options {
            let (_range, _new_text) = option.run(backend).await?;

            i += 1;

            // TODO:
            // results
            //     .entry(&options.)
            //     .and_modify(|e| {
            //         e.1 += &options.content;
            //     })
            //     .or_insert((options.permission, options.content, options.mkdir));
        }

        if i > 0 {
            backend
                .show_message(
                    MessageType::INFO,
                    format!("Found {} code block from {}", i, self.url),
                )
                .await;
        }

        Ok(true)
    }
}

impl Executable for SrcBlockTangle {
    const NAME: &'static str = "src-block-tangle";

    const TITLE: Option<&'static str> = Some("Tangle");

    type Result = bool;

    async fn execute<B: Backend>(self, backend: &B) -> anyhow::Result<bool> {
        let Some(block) = backend
            .documents()
            .get_and_then(&self.url, |doc| doc.org.node_at_offset(self.block_offset))
        else {
            return Ok(false);
        };

        let Some(options) = TangleOptions::new(block, &self.url, backend) else {
            backend
                .show_message(MessageType::ERROR, "Code block can't be tangled.".into())
                .await;

            return Ok(false);
        };

        let (range, new_text) = options.run(backend).await?;

        let content = backend.read_to_string(&options.destination).await?;

        if let Some((start, end)) = range {
            let new_content = format!("{}{}{}", &content[0..start], new_text, &content[end..]);
            backend.write(&options.destination, &new_content).await?;
        } else {
            let new_content = format!("{}{}", &content, new_text);
            backend.write(&options.destination, &new_content).await?;
        }

        backend
            .show_message(
                MessageType::INFO,
                format!("Write to {}", options.destination),
            )
            .await;

        Ok(true)
    }
}

struct TangleOptions {
    destination: Url,
    _permission: Option<u32>,
    content: String,
    _mkdir: bool,

    padline: bool,
    shebang: Option<String>,
    org_comments: String,
    comment_links: Option<(String, String)>,
}

impl TangleOptions {
    pub fn new<B: Backend>(block: SourceBlock, base: &Url, backend: &B) -> Option<Self> {
        let arg1 = block.parameters().unwrap_or_default();
        let arg2 = property_drawer(block.syntax()).unwrap_or_default();
        let arg3 = property_keyword(block.syntax()).unwrap_or_default();
        let language = block.language().unwrap_or_default();

        let tangle = header_argument(&arg1, &arg2, &arg3, ":tangle", "no");

        if tangle == "no" {
            return None;
        }

        let comments = header_argument(&arg1, &arg2, &arg3, ":comments", "no");
        let padline = header_argument(&arg1, &arg2, &arg3, ":padline", "no");
        let shebang = header_argument(&arg1, &arg2, &arg3, ":shebang", "no");
        let mode = header_argument(
            &arg1,
            &arg2,
            &arg3,
            ":tangle-mode",
            if shebang == "yea" { "o755" } else { "no" },
        );
        let is_mkdir = header_argument(&arg1, &arg2, &arg3, ":mkdir", "no");

        let parent = block
            .syntax()
            .ancestors()
            .find(|n| n.kind() == SyntaxKind::HEADLINE || n.kind() == SyntaxKind::DOCUMENT);

        let nth = parent
            .as_ref()
            .and_then(|n| n.children().position(|c| &c == block.syntax()))
            .unwrap_or(1);

        let headline_title = parent.and_then(Headline::cast).map(|headline| {
            headline
                .title()
                .fold(String::new(), |a, n| a + &n.to_string())
        });

        let destination = backend.resolve_in(tangle, base).ok()?;

        let mut permission = None;
        if mode != "no"
            && mode.len() == 4
            && mode.starts_with('o')
            && mode.bytes().skip(1).all(|b| (b'0'..=b'7').contains(&b))
        {
            permission = u32::from_str_radix(&mode[1..], 8).ok();
        }

        let mut org_comments = String::new();
        if comments == "org" || comments == "both" {
            if let Some((begin, end)) = language_comments(&language) {
                let start = block
                    .syntax()
                    .siblings(Direction::Prev)
                    .skip(1) // skip self
                    .take_while(|n| n.kind() != SyntaxKind::SOURCE_BLOCK)
                    .last();

                for sibling in start
                    .into_iter()
                    .flat_map(|start| start.siblings(Direction::Next))
                    .take_while(|node| node != block.syntax())
                {
                    for line in sibling.to_string().lines() {
                        if line.is_empty() {
                            let _ = writeln!(org_comments);
                        } else {
                            let _ = writeln!(org_comments, "{begin} {line} {end}");
                        }
                    }
                }
            }
        }

        let mut comment_links = None;
        if comments == "yes" || comments == "link" || comments == "noweb" || comments == "both" {
            if let Some((begin, end)) = language_comments(&language) {
                comment_links = Some((
                    format!(
                        "{begin} [[{destination}::*{title}][{title}:{nth}]] {end}",
                        title = headline_title.as_deref().unwrap_or("No heading"),
                        destination = destination,
                    ),
                    format!(
                        "{begin} {title}:{nth} ends here {end}",
                        title = headline_title.as_deref().unwrap_or("No heading"),
                    ),
                ))
            }
        }

        Some(TangleOptions {
            shebang: if shebang != "no" && !shebang.is_empty() {
                Some(shebang.to_string())
            } else {
                None
            },
            destination,
            _permission: permission,
            org_comments,
            content: block.value(),
            _mkdir: is_mkdir != "no",
            padline: padline != "no",
            comment_links,
        })
    }

    pub async fn run<B: Backend>(
        &self,
        backend: &B,
    ) -> anyhow::Result<(Option<(usize, usize)>, String)> {
        let content = backend.read_to_string(&self.destination).await?;

        let mut range = None;
        if let Some((begin, end)) = &self.comment_links {
            let mut offset = 0;

            let bytes = content.as_bytes();

            let mut start_idx = None;
            let mut end_idx = None;

            for i in memchr2_iter(b'\n', b'\r', bytes)
                .filter(|&i| bytes[i] == b'\r' && bytes.get(i + 1) == Some(&b'\n'))
                .map(|i| i + 1)
                .chain(std::iter::once(content.len()))
            {
                let line = &content[offset..i];

                if start_idx.is_none() && line == begin {
                    start_idx = Some(i);
                } else if end_idx.is_none() && line == end {
                    end_idx = Some(offset);
                    break;
                }

                offset = i;
            }

            match (start_idx, end_idx) {
                (Some(s), Some(e)) => {
                    range = Some((s, e));
                }
                _ => {}
            }
        }

        let mut new_text = String::new();

        if range.is_none() {
            if let Some(shebang) = &self.shebang {
                new_text += &shebang;
                new_text += "\n";
            }
            // TODO: update org comments
            new_text += &self.org_comments;
            new_text += "\n";
        }

        if let Some((begin, end)) = &self.comment_links {
            new_text += &begin;
            new_text += "\n";
            new_text += &self.content;
            if self.padline {
                new_text += "\n";
            }
            new_text += &end;
            new_text += "\n";
        } else {
            new_text += &self.content;
            if self.padline {
                new_text += "\n";
            }
        }

        // TODO: set file permission
        // TODO: run mkdir

        Ok((range, new_text))
    }
}

#[cfg(test)]
#[tokio::test]
async fn test() {
    use crate::test::TestBackend;

    let backend = TestBackend::default();
    let url = Url::parse("test://test.org").unwrap();

    backend.documents().insert(
        url.clone(),
        r#"#+begin_src js :tangle ./a.js
console.log('a')
#+end_src
"#,
    );

    SrcBlockTangle {
        url: url.clone(),
        block_offset: 0.into(),
    }
    .execute(&backend)
    .await
    .unwrap();

    assert_eq!(
        backend.get(&Url::parse("test://test.org/a.js").unwrap()),
        "\nconsole.log('a')\n"
    );
}
