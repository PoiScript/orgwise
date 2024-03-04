use lsp_types::*;
use orgize::{
    export::{Container, Event, TraversalContext, Traverser},
    rowan::ast::AstNode,
    SyntaxKind,
};
use std::collections::HashMap;
use std::fmt::Write;

use super::{FileSystem, LanguageClient, LanguageServerBase, Process};

impl<E> LanguageServerBase<E>
where
    E: FileSystem + LanguageClient + Process,
{
    pub async fn headline_toc(&self, url: Url, headline_offset: u32) {
        let Some(doc) = self.documents.get(&url) else {
            return;
        };

        let mut toc = Toc {
            indent: 0,
            output: String::new(),

            headline_offset,
            edit_range: None,
        };

        doc.traverse(&mut toc);

        if let Some((start, end)) = toc.edit_range {
            let mut changes = HashMap::new();

            let range = doc.range_of2(start, end);

            changes.insert(
                url,
                vec![TextEdit {
                    new_text: toc.output,
                    range,
                }],
            );

            let _ = self
                .env
                .apply_edit(WorkspaceEdit {
                    changes: Some(changes),
                    ..Default::default()
                })
                .await;
        }
    }
}

struct Toc {
    output: String,
    indent: usize,

    headline_offset: u32,

    edit_range: Option<(u32, u32)>,
}

impl Traverser for Toc {
    fn event(&mut self, event: Event, ctx: &mut TraversalContext) {
        match event {
            Event::Enter(Container::Headline(headline)) => {
                if headline.begin() == self.headline_offset {
                    let start = headline
                        .syntax()
                        .children_with_tokens()
                        .find(|n| n.kind() == SyntaxKind::NEW_LINE)
                        .map(|n| n.text_range().end().into())
                        .unwrap_or(headline.end());

                    let end = headline.end();

                    self.edit_range = Some((start, end));
                } else {
                    let title = headline.title().map(|e| e.to_string()).collect::<String>();

                    let slug = crate::common::headline_slug(&headline);

                    let _ = writeln!(
                        &mut self.output,
                        "{: >i$}- [[#{slug}][{title}]]",
                        "",
                        i = self.indent
                    );
                }

                self.indent += 2;
            }
            Event::Leave(Container::Headline(_)) => self.indent -= 2,

            Event::Enter(Container::Section(_)) => ctx.skip(),
            Event::Enter(Container::Document(_)) => self.output += "#+begin_quote\n",
            Event::Leave(Container::Document(_)) => self.output += "#+end_quote\n\n",
            _ => {}
        }
    }
}
