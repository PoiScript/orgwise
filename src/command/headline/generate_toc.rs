use lsp_types::*;
use orgize::{
    export::{from_fn_with_ctx, Container, Event},
    rowan::{ast::AstNode, TextRange, TextSize},
    SyntaxKind,
};
use serde::{Deserialize, Serialize};
use std::fmt::Write;

use crate::backend::Backend;

use crate::command::Executable;
use crate::utils::headline::headline_slug;

#[derive(Deserialize, Serialize)]
pub struct HeadlineGenerateToc {
    pub url: Url,
    #[serde(with = "crate::utils::text_size")]
    pub headline_offset: TextSize,
}

impl Executable for HeadlineGenerateToc {
    const NAME: &'static str = "headline-toc";

    const TITLE: Option<&'static str> = Some("Generate TOC");

    type Result = bool;

    async fn execute<B: Backend>(self, backend: &B) -> anyhow::Result<bool> {
        let mut edit_range: Option<TextRange> = None;
        let mut output = String::new();
        let mut indent = 0;

        let Some(_) = backend.documents().get_map(&self.url, |doc| {
            doc.traverse(&mut from_fn_with_ctx(|event, ctx| match event {
                Event::Enter(Container::Headline(headline)) => {
                    if headline.start() == self.headline_offset {
                        let start = headline
                            .syntax()
                            .children_with_tokens()
                            .find(|n| n.kind() == SyntaxKind::NEW_LINE)
                            .map(|n| n.text_range().end());

                        let end = headline.end();

                        edit_range = Some(TextRange::new(start.unwrap_or(end), end));
                    } else {
                        let title = headline.title_raw();

                        let slug = headline_slug(&headline);

                        let _ = writeln!(&mut output, "{: >indent$}- [[#{slug}][{title}]]", "",);
                    }

                    indent += 2;
                }
                Event::Leave(Container::Headline(_)) => indent -= 2,
                Event::Enter(Container::Section(_)) => ctx.skip(),
                Event::Enter(Container::Document(_)) => output += "#+begin_quote\n",
                Event::Leave(Container::Document(_)) => output += "#+end_quote\n\n",
                _ => {}
            }));
        }) else {
            return Ok(false);
        };

        if let Some(text_range) = edit_range {
            backend.apply_edit(self.url, output, text_range).await?;
        }

        Ok(true)
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
        r#"* toc
* a
**** g
* b
* c
*** d
** e
*** f"#,
    );

    HeadlineGenerateToc {
        headline_offset: 0.into(),
        url: url.clone(),
    }
    .execute(&backend)
    .await
    .unwrap();
    assert_eq!(
        backend.get(&url),
        r#"* toc
#+begin_quote
- [[#a][a]]
  - [[#g][g]]
- [[#b][b]]
- [[#c][c]]
  - [[#d][d]]
  - [[#e][e]]
    - [[#f][f]]
#+end_quote

* a
**** g
* b
* c
*** d
** e
*** f"#
    );
}
