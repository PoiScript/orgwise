use chrono::NaiveDateTime;
use lsp_types::{MessageType, Url};
use memchr::memchr2;
use orgize::{
    ast::{Drawer, Headline},
    rowan::ast::AstNode,
    rowan::TextRange,
    SyntaxKind,
};
use serde::{Deserialize, Serialize};

use crate::{backend::Backend, utils::timestamp::FormatActiveTimestamp};

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
    pub scheduled: Option<NaiveDateTime>,
    pub deadline: Option<NaiveDateTime>,
}

impl Executable for HeadlineUpdate {
    const NAME: &'static str = "headline-update";

    type Result = bool;

    async fn execute<B: Backend>(self, backend: &B) -> anyhow::Result<bool> {
        let Some(Some(headline)) = backend
            .documents()
            .get_map(&self.url, |doc| find_headline(&doc, self.line))
        else {
            backend
                .log_message(
                    MessageType::WARNING,
                    format!("cannot find document with url {}", self.url),
                )
                .await;

            return Ok(false);
        };

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
            .chain(self.edit_keyword(&headline))
            .chain(self.edit_priority(&headline))
            .chain(self.edit_tags(&headline))
            .chain(self.edit_planning(&headline))
            .chain(self.edit_section(&headline))
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
                    .map(|t| TextRange::empty(t.text_range().start()))
                    .unwrap_or_else(|| TextRange::empty(headline.end()));

                Some((format!(" {title}"), text_range))
            }

            (None, true) => None,
        }
    }

    fn edit_section(&self, headline: &Headline) -> Option<(String, TextRange)> {
        let mut section = self.section.as_ref()?.trim().to_string();

        if let Some(s) = headline.section() {
            for (index, drawer) in s.syntax().children().filter_map(Drawer::cast).enumerate() {
                if index == 0 {
                    section.push('\n');
                }
                let drawer = drawer.syntax().to_string();
                section.push_str(&drawer);
                if !drawer.ends_with(['\n', '\r']) {
                    section.push('\n');
                }
            }
        }

        let to_replace = headline.section().map(|s| s.text_range());

        match (to_replace, section.is_empty()) {
            (Some(old), false) => Some((format!("{section}\n"), old)),

            (Some(old), true) => Some((String::new(), old)),

            (None, false) => headline
                .syntax()
                .children_with_tokens()
                .find(|t| t.kind() == SyntaxKind::NEW_LINE)
                .map(|t| {
                    Some((
                        format!("{section}\n"),
                        TextRange::empty(t.text_range().end()),
                    ))
                })
                .unwrap_or_else(|| {
                    Some((format!("\n{section}\n"), TextRange::empty(headline.end())))
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
                if let Some(kw) = headline.todo_keyword() {
                    Some((format!(" [#{priority}]"), TextRange::empty(kw.end())))
                } else {
                    let s = headline
                        .syntax()
                        .children_with_tokens()
                        // the second element must be a whitespace
                        .nth(1)
                        .unwrap()
                        .text_range()
                        .end();

                    Some((format!("[#{priority}] "), TextRange::empty(s)))
                }
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

                Some((format!(" :{}:", tags.join(":")), TextRange::empty(position)))
            }

            (None, true) => None,
        }
    }

    fn edit_planning(&self, headline: &Headline) -> Option<(String, TextRange)> {
        let to_replace = headline
            .syntax()
            .children_with_tokens()
            .find(|tk| tk.kind() == SyntaxKind::PLANNING);

        let planning = match (self.scheduled, self.deadline) {
            (Some(scheduled), Some(deadline)) => Some(format!(
                "SCHEDULED: {} DEADLINE: {}\n",
                FormatActiveTimestamp(scheduled),
                FormatActiveTimestamp(deadline)
            )),

            (Some(scheduled), None) => {
                Some(format!("SCHEDULED: {}\n", FormatActiveTimestamp(scheduled)))
            }

            (None, Some(deadline)) => {
                Some(format!("DEADLINE: {}\n", FormatActiveTimestamp(deadline)))
            }

            _ => None,
        };

        match (to_replace, planning) {
            (Some(old), Some(planning)) => Some((planning, old.text_range())),

            (Some(old), None) => Some((String::new(), old.text_range())),

            (None, Some(mut planning)) => {
                if let Some(new_line) = headline
                    .syntax()
                    .children_with_tokens()
                    .find(|t| t.kind() == SyntaxKind::NEW_LINE)
                {
                    Some((planning, TextRange::empty(new_line.text_range().end())))
                } else {
                    planning.insert_str(0, "\n");
                    Some((planning, TextRange::empty(headline.end())))
                }
            }

            _ => None,
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
                deadline: None,
                scheduled: None,
            }
        }
    }

    let backend = TestBackend::default();
    let url = Url::parse("test://test.org").unwrap();
    backend.documents().insert(url.clone(), "* ");

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

    // update nest headline
    backend
        .documents()
        .insert(url.clone(), "* abc\nsection\n** abc");
    HeadlineUpdate {
        title: Some("mon".into()),
        section: Some("section".into()),
        line: 3,
        keyword: Some("TODO".into()),
        priority: Some("A".into()),
        ..Default::default()
    }
    .execute(&backend)
    .await
    .unwrap();
    assert_eq!(
        backend.get(&url),
        "* abc\nsection\n** TODO [#A] mon\nsection\n"
    );
}
