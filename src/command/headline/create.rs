use lsp_types::{MessageType, Url};
use orgize::rowan::TextRange;
use serde::{Deserialize, Serialize};

use crate::backend::Backend;

use crate::command::Executable;

#[derive(Deserialize, Serialize, Debug)]
pub struct HeadlineCreate {
    pub url: Url,
    pub priority: Option<String>,
    pub keyword: Option<String>,
    pub title: Option<String>,
    pub tags: Option<Vec<String>>,
    pub section: Option<String>,
}

impl Executable for HeadlineCreate {
    const NAME: &'static str = "headline-create";

    type Result = bool;

    async fn execute<B: Backend>(self, backend: &B) -> anyhow::Result<bool> {
        let Some(end) = backend
            .documents()
            .get_map(&self.url, |doc| doc.org.document().end())
        else {
            backend
                .log_message(
                    MessageType::WARNING,
                    format!("cannot find document with url {}", self.url),
                )
                .await;

            return Ok(false);
        };

        let mut s = "\n*".to_string();

        if let Some(priority) = self.priority {
            s.push_str(" [#");
            s.push_str(&priority);
            s.push(']');
        }

        if let Some(keyword) = self.keyword {
            s.push(' ');
            s.push_str(&keyword);
        }

        s.push(' ');
        if let Some(title) = self.title {
            s.push_str(&title);
        }

        if let Some(tags) = self.tags {
            s.push(':');
            for tag in tags {
                s.push_str(&tag);
                s.push(':');
            }
        }

        s.push('\n');

        if let Some(section) = self.section {
            s.push_str(&section);
        }

        s.push('\n');

        backend
            .apply_edit(self.url, s, TextRange::new(end, end))
            .await?;

        Ok(true)
    }
}
