use lsp_types::{MessageType, Url};
use memchr::memchr2;
use orgize::rowan::ast::AstNode;
use orgize::SyntaxKind;
use orgize::{ast::Headline, rowan::TextRange};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::base::Server;

use super::utils::find_headline;
use super::Executable;

#[derive(Deserialize, Serialize, Debug)]
pub struct UpdateHeadline {
    pub url: Url,
    pub line: u32,
    pub keyword: Option<String>,
    pub priority: Option<String>,
    pub title: Option<String>,
    pub section: Option<String>,
    pub tags: Option<Vec<String>>,
}

impl Executable for UpdateHeadline {
    const NAME: &'static str = "update-headline";

    async fn execute<S: Server>(self, server: &S) -> anyhow::Result<Value> {
        let Some(doc) = server.documents().get(&self.url) else {
            server
                .log_message(
                    MessageType::WARNING,
                    format!("cannot find document with url {}", self.url),
                )
                .await;

            return Ok(Value::Null);
        };

        let Some(headline) = find_headline(&doc, self.line) else {
            server
                .log_message(
                    MessageType::WARNING,
                    format!("cannot find headline in line {}", self.line),
                )
                .await;

            return Ok(Value::Null);
        };

        drop(doc);

        let edits = self.edit(headline);

        let edits: Vec<_> = edits
            .into_iter()
            .map(|(new_text, text_range)| (self.url.clone(), new_text, text_range))
            .collect();

        server.apply_edits(edits.into_iter()).await?;

        Ok(Value::Bool(true))
    }
}

impl UpdateHeadline {
    fn edit(&self, headline: Headline) -> Vec<(String, TextRange)> {
        self.edit_title(&headline)
            .into_iter()
            .chain(self.edit_priority(&headline))
            .chain(self.edit_keyword(&headline))
            .chain(self.edit_section(&headline))
            .chain(self.edit_tags(&headline))
            .collect()
    }

    fn edit_title(&self, headline: &Headline) -> Option<(String, TextRange)> {
        let title = self.title.as_ref()?;

        let to_replace = headline
            .syntax()
            .children_with_tokens()
            .find(|tk| tk.kind() == SyntaxKind::HEADLINE_TITLE);

        let title = match memchr2(b'\n', b'\r', title.as_bytes()) {
            Some(i) => &title[..i],
            None => title.as_str(),
        };

        match (to_replace, title.is_empty()) {
            (Some(old), false) => Some((title.to_string(), old.text_range())),

            (Some(old), true) => Some((String::new(), old.text_range())),

            (None, false) => {
                let text_range = headline
                    .syntax()
                    .children_with_tokens()
                    .find(|t| t.kind() == SyntaxKind::NEW_LINE)
                    .map(|t| {
                        let s = t.text_range().start();
                        TextRange::new(s, s)
                    })
                    .unwrap_or_else(|| {
                        let s = headline.syntax().text_range().end();
                        TextRange::new(s, s)
                    });

                Some((format!(" {title}"), text_range))
            }

            (None, true) => None,
        }
    }

    fn edit_section(&self, headline: &Headline) -> Option<(String, TextRange)> {
        let section = self.section.as_ref()?.trim();

        let to_replace = headline.section().map(|s| s.syntax().text_range());

        match (to_replace, section.is_empty()) {
            (Some(old), false) => Some((format!("{section}\n"), old)),

            (Some(old), true) => Some((String::new(), old)),

            (None, false) => headline
                .syntax()
                .children_with_tokens()
                .find(|t| t.kind() == SyntaxKind::NEW_LINE)
                .map(|t| {
                    let s = t.text_range().end();

                    Some((format!("{section}\n"), TextRange::new(s, s)))
                })
                .unwrap_or_else(|| {
                    let s = headline.syntax().text_range().end();
                    Some((format!("\n{section}\n"), TextRange::new(s, s)))
                }),

            (None, true) => None,
        }
    }

    fn edit_priority(&self, headline: &Headline) -> Option<(String, TextRange)> {
        let to_replace = headline
            .syntax()
            .children_with_tokens()
            .find(|tk| tk.kind() == SyntaxKind::HEADLINE_PRIORITY);

        let priority = self.priority.as_ref()?;

        match (to_replace, priority.is_empty()) {
            (Some(old), false) => Some((format!("[#{priority}]"), old.text_range())),

            (Some(old), true) => Some((String::new(), old.text_range())),

            (None, false) => {
                let s = headline
                    .syntax()
                    .children_with_tokens()
                    // the second element must be a whitespace
                    .nth(1)
                    .unwrap()
                    .text_range()
                    .end();

                Some((format!("[#{priority}] "), TextRange::new(s, s)))
            }

            (None, true) => None,
        }
    }

    fn edit_keyword(&self, headline: &Headline) -> Option<(String, TextRange)> {
        let to_replace = headline.syntax().children_with_tokens().find(|tk| {
            tk.kind() == SyntaxKind::HEADLINE_KEYWORD_TODO
                || tk.kind() == SyntaxKind::HEADLINE_KEYWORD_DONE
        });

        let keyword = self.keyword.as_ref()?;

        match (to_replace, keyword.is_empty()) {
            (Some(old), false) => Some((keyword.to_string(), old.text_range())),

            (Some(old), true) => Some((String::new(), old.text_range())),

            (None, false) => {
                let text_range = headline
                    .syntax()
                    .children_with_tokens()
                    // the second element must be a whitespace
                    .nth(1)
                    .unwrap()
                    .text_range();

                Some((format!(" {keyword} "), text_range))
            }

            (None, true) => None,
        }
    }

    fn edit_tags(&self, headline: &Headline) -> Option<(String, TextRange)> {
        let tags = self.tags.as_ref()?;

        let to_replace = headline
            .syntax()
            .children_with_tokens()
            .find(|tk| tk.kind() == SyntaxKind::HEADLINE_TAGS);

        match (to_replace, tags.is_empty()) {
            (Some(old), false) => Some((format!(":{}:", tags.join(":")), old.text_range())),

            (Some(old), true) => Some((String::new(), old.text_range())),

            (None, false) => {
                let position = headline
                    .syntax()
                    .children_with_tokens()
                    .find(|t| t.kind() == SyntaxKind::NEW_LINE)
                    .map(|t| t.text_range().start())
                    .unwrap_or_else(|| headline.syntax().text_range().end());

                Some((
                    format!(" :{}:", tags.join(":")),
                    TextRange::new(position, position),
                ))
            }

            (None, true) => None,
        }
    }
}

#[cfg(test)]
#[tokio::test]
async fn test() {
    use crate::test::TestServer;

    impl Default for UpdateHeadline {
        fn default() -> Self {
            UpdateHeadline {
                url: Url::parse("test://test.org").unwrap(),
                line: 1,
                keyword: None,
                priority: None,
                title: None,
                section: None,
                tags: None,
            }
        }
    }

    let server = TestServer::default();
    let url = Url::parse("test://test.org").unwrap();
    server.add_doc(url.clone(), "* ".into());

    // keyword
    {
        UpdateHeadline {
            keyword: Some("DONE".into()),
            ..Default::default()
        }
        .execute(&server)
        .await
        .unwrap();
        assert_eq!(server.get(&url), "* DONE ");

        UpdateHeadline {
            keyword: Some("TODO".into()),
            ..Default::default()
        }
        .execute(&server)
        .await
        .unwrap();
        assert_eq!(server.get(&url), "* TODO ");

        UpdateHeadline {
            keyword: Some("".into()),
            ..Default::default()
        }
        .execute(&server)
        .await
        .unwrap();
        assert_eq!(server.get(&url), "*  ");
    }

    // title
    {
        UpdateHeadline {
            title: Some("title".into()),
            ..Default::default()
        }
        .execute(&server)
        .await
        .unwrap();
        assert_eq!(server.get(&url), "*   title");

        UpdateHeadline {
            title: Some("hello world".into()),
            ..Default::default()
        }
        .execute(&server)
        .await
        .unwrap();
        assert_eq!(server.get(&url), "*   hello world");

        UpdateHeadline {
            title: Some("".into()),
            ..Default::default()
        }
        .execute(&server)
        .await
        .unwrap();
        assert_eq!(server.get(&url), "*   ");
    }

    // priority
    {
        UpdateHeadline {
            priority: Some("A".into()),
            ..Default::default()
        }
        .execute(&server)
        .await
        .unwrap();
        assert_eq!(server.get(&url), "*   [#A] ");

        UpdateHeadline {
            priority: Some("B".into()),
            ..Default::default()
        }
        .execute(&server)
        .await
        .unwrap();
        assert_eq!(server.get(&url), "*   [#B] ");

        UpdateHeadline {
            priority: Some("".into()),
            ..Default::default()
        }
        .execute(&server)
        .await
        .unwrap();
        assert_eq!(server.get(&url), "*    ");
    }

    // tags
    {
        UpdateHeadline {
            tags: Some(vec!["a".into(), "b".into()]),
            ..Default::default()
        }
        .execute(&server)
        .await
        .unwrap();
        assert_eq!(server.get(&url), "*     :a:b:");

        UpdateHeadline {
            tags: Some(vec!["foo".into(), "bar".into()]),
            ..Default::default()
        }
        .execute(&server)
        .await
        .unwrap();
        assert_eq!(server.get(&url), "*     :foo:bar:");

        UpdateHeadline {
            tags: Some(vec![]),
            ..Default::default()
        }
        .execute(&server)
        .await
        .unwrap();
        assert_eq!(server.get(&url), "*     ");
    }

    // section
    {
        UpdateHeadline {
            section: Some("section".into()),
            ..Default::default()
        }
        .execute(&server)
        .await
        .unwrap();
        assert_eq!(server.get(&url), "*     \nsection\n");

        UpdateHeadline {
            section: Some("long \n \n section".into()),
            ..Default::default()
        }
        .execute(&server)
        .await
        .unwrap();
        assert_eq!(server.get(&url), "*     \nlong \n \n section\n");

        UpdateHeadline {
            section: Some("".into()),
            ..Default::default()
        }
        .execute(&server)
        .await
        .unwrap();
        assert_eq!(server.get(&url), "*     \n");
    }
}
