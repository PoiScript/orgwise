use chrono::NaiveDateTime;
use lsp_types::{MessageType, Url};
use orgize::rowan::TextRange;
use serde::{Deserialize, Serialize};
use std::fmt::Write;

use crate::backend::Backend;
use crate::command::Executable;
use crate::utils::timestamp::FormatActiveTimestamp;

#[derive(Deserialize, Serialize, Debug)]
pub struct HeadlineCreate {
    pub url: Url,
    pub priority: Option<String>,
    pub keyword: Option<String>,
    pub title: Option<String>,
    pub tags: Option<Vec<String>>,
    pub section: Option<String>,
    pub scheduled: Option<NaiveDateTime>,
    pub deadline: Option<NaiveDateTime>,
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

        if let Some(keyword) = self.keyword.filter(|t| !t.is_empty()) {
            s.push(' ');
            s.push_str(&keyword);
        }

        if let Some(priority) = self.priority.filter(|t| !t.is_empty()) {
            s.push_str(" [#");
            s.push_str(&priority);
            s.push(']');
        }

        s.push(' ');
        if let Some(title) = self.title {
            s.push_str(&title);
        }

        if let Some(tags) = self.tags.filter(|t| !t.is_empty()) {
            s.push_str(" :");
            for tag in tags {
                s.push_str(&tag);
                s.push(':');
            }
        }

        s.push('\n');

        match (self.scheduled, self.deadline) {
            (Some(scheduled), Some(deadline)) => {
                let _ = writeln!(
                    &mut s,
                    "SCHEDULED: {} DEADLINE: {}",
                    FormatActiveTimestamp(scheduled),
                    FormatActiveTimestamp(deadline)
                );
            }

            (Some(scheduled), None) => {
                let _ = writeln!(&mut s, "SCHEDULED: {}", FormatActiveTimestamp(scheduled));
            }

            (None, Some(deadline)) => {
                let _ = writeln!(&mut s, "DEADLINE: {}", FormatActiveTimestamp(deadline));
            }

            _ => {}
        };

        if let Some(section) = self.section.filter(|t| !t.is_empty()) {
            s.push_str(&section);
            s.push('\n');
        }

        backend
            .apply_edit(self.url, s, TextRange::empty(end))
            .await?;

        Ok(true)
    }
}
