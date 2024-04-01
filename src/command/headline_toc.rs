use lsp_types::*;
use orgize::{
    export::{from_fn_with_ctx, Container, Event},
    rowan::{ast::AstNode, TextRange, TextSize},
    SyntaxKind,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt::Write;

use crate::base::Server;

use super::Executable;

#[derive(Deserialize, Serialize)]
pub struct HeadlineToc {
    pub url: Url,
    #[serde(with = "crate::command::utils::text_size")]
    pub headline_offset: TextSize,
}

impl Executable for HeadlineToc {
    const NAME: &'static str = "headline-toc";

    const TITLE: Option<&'static str> = Some("Generate TOC");

    async fn execute<S: Server>(self, server: &S) -> anyhow::Result<Value> {
        let Some(doc) = server.documents().get(&self.url) else {
            return Ok(Value::Null);
        };

        let mut indent = 0;
        let mut edit_range: Option<TextRange> = None;
        let mut output = String::new();

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

                    let slug = super::utils::headline_slug(&headline);

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

        drop(doc);

        if let Some(text_range) = edit_range {
            server.apply_edit(self.url, output, text_range).await?;
        }

        Ok(Value::Bool(true))
    }
}

#[cfg(test)]
#[tokio::test]
async fn test() {
    use crate::test::TestServer;

    let server = TestServer::default();
    let url = Url::parse("test://test.org").unwrap();
    server.add_doc(
        url.clone(),
        r#"* toc
* a
**** g
* b
* c
*** d
** e
*** f"#
            .into(),
    );

    HeadlineToc {
        headline_offset: 0.into(),
        url: url.clone(),
    }
    .execute(&server)
    .await
    .unwrap();
    assert_eq!(
        server.get(&url),
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
