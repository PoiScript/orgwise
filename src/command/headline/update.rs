use lsp_types::{MessageType, Url};
use memchr::memchr2;
use orgize::rowan::ast::AstNode;
use orgize::SyntaxKind;
use orgize::{ast::Headline, rowan::TextRange};
use serde::{Deserialize, Serialize};

use crate::backend::Backend;

use crate::command::Executable;
use crate::utils::headline::find_headline;

#[derive(Deserialize, Serialize, Debug)]
pub struct HeadlineUpdate {
    pub url: Url,
    pub line: u32,
    pub keyword: Option<String>,
    pub priority: Option<String>,
    pub title: Option<String>,
    pub section: Option<String>,
    pub tags: Option<Vec<String>>,
}

impl Executable for HeadlineUpdate {
    const NAME: &'static str = "headline-update";

    type Result = bool;

    async fn execute<B: Backend>(self, backend: &B) -> anyhow::Result<bool> {
        let Some(doc) = backend.documents().get(&self.url) else {
            backend
                .log_message(
                    MessageType::WARNING,
                    format!("cannot find document with url {}", self.url),
                )
                .await;

            return Ok(false);
        };

        let Some(headline) = find_headline(&doc, self.line) else {
            backend
                .log_message(
                    MessageType::WARNING,
                    format!("cannot find headline in line {}", self.line),
                )
                .await;

            return Ok(false);
        };

        drop(doc);

        let edits = self.edit(headline);

        let edits: Vec<_> = edits
            .into_iter()
            .map(|(new_text, text_range)| (self.url.clone(), new_text, text_range))
            .collect();

        backend.apply_edits(edits.into_iter()).await?;

        Ok(true)
    }
}

impl HeadlineUpdate {
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
                        let s = headline.end();
                        TextRange::new(s, s)
                    });

                Some((format!(" {title}"), text_range))
            }

            (None, true) => None,
        }
    }

    fn edit_section(&self, headline: &Headline) -> Option<(String, TextRange)> {
        let section = self.section.as_ref()?.trim();

        let to_replace = headline.section().map(|s| s.text_range());

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
                    let s = headline.end();
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
                    .unwrap_or_else(|| headline.end());

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
    use crate::test::TestBackend;

    impl Default for HeadlineUpdate {
        fn default() -> Self {
            HeadlineUpdate {
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

    let backend = TestBackend::default();
    let url = Url::parse("test://test.org").unwrap();
    backend.add_doc(url.clone(), "* ".into());

    // keyword
    {
        HeadlineUpdate {
            keyword: Some("DONE".into()),
            ..Default::default()
        }
        .execute(&backend)
        .await
        .unwrap();
        assert_eq!(backend.get(&url), "* DONE ");

        HeadlineUpdate {
            keyword: Some("TODO".into()),
            ..Default::default()
        }
        .execute(&backend)
        .await
        .unwrap();
        assert_eq!(backend.get(&url), "* TODO ");

        HeadlineUpdate {
            keyword: Some("".into()),
            ..Default::default()
        }
        .execute(&backend)
        .await
        .unwrap();
        assert_eq!(backend.get(&url), "*  ");
    }

    // title
    {
        HeadlineUpdate {
            title: Some("title".into()),
            ..Default::default()
        }
        .execute(&backend)
        .await
        .unwrap();
        assert_eq!(backend.get(&url), "*   title");

        HeadlineUpdate {
            title: Some("hello world".into()),
            ..Default::default()
        }
        .execute(&backend)
        .await
        .unwrap();
        assert_eq!(backend.get(&url), "*   hello world");

        HeadlineUpdate {
            title: Some("".into()),
            ..Default::default()
        }
        .execute(&backend)
        .await
        .unwrap();
        assert_eq!(backend.get(&url), "*   ");
    }

    // priority
    {
        HeadlineUpdate {
            priority: Some("A".into()),
            ..Default::default()
        }
        .execute(&backend)
        .await
        .unwrap();
        assert_eq!(backend.get(&url), "*   [#A] ");

        HeadlineUpdate {
            priority: Some("B".into()),
            ..Default::default()
        }
        .execute(&backend)
        .await
        .unwrap();
        assert_eq!(backend.get(&url), "*   [#B] ");

        HeadlineUpdate {
            priority: Some("".into()),
            ..Default::default()
        }
        .execute(&backend)
        .await
        .unwrap();
        assert_eq!(backend.get(&url), "*    ");
    }

    // tags
    {
        HeadlineUpdate {
            tags: Some(vec!["a".into(), "b".into()]),
            ..Default::default()
        }
        .execute(&backend)
        .await
        .unwrap();
        assert_eq!(backend.get(&url), "*     :a:b:");

        HeadlineUpdate {
            tags: Some(vec!["foo".into(), "bar".into()]),
            ..Default::default()
        }
        .execute(&backend)
        .await
        .unwrap();
        assert_eq!(backend.get(&url), "*     :foo:bar:");

        HeadlineUpdate {
            tags: Some(vec![]),
            ..Default::default()
        }
        .execute(&backend)
        .await
        .unwrap();
        assert_eq!(backend.get(&url), "*     ");
    }

    // section
    {
        HeadlineUpdate {
            section: Some("section".into()),
            ..Default::default()
        }
        .execute(&backend)
        .await
        .unwrap();
        assert_eq!(backend.get(&url), "*     \nsection\n");

        HeadlineUpdate {
            section: Some("long \n \n section".into()),
            ..Default::default()
        }
        .execute(&backend)
        .await
        .unwrap();
        assert_eq!(backend.get(&url), "*     \nlong \n \n section\n");

        HeadlineUpdate {
            section: Some("".into()),
            ..Default::default()
        }
        .execute(&backend)
        .await
        .unwrap();
        assert_eq!(backend.get(&url), "*     \n");
    }
}
