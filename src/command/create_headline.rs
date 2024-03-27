use lsp_types::{MessageType, Url};
use orgize::rowan::ast::AstNode;
use orgize::rowan::TextRange;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::base::Server;

use super::Executable;

#[derive(Deserialize, Serialize, Debug)]
pub struct CreateHeadline {
    pub url: Url,
    pub priority: Option<String>,
    pub keyword: Option<String>,
    pub title: Option<String>,
    pub tags: Option<Vec<String>>,
    pub section: Option<String>,
}

impl Executable for CreateHeadline {
    const NAME: &'static str = "create-headline";

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

        let end = doc.org.document().syntax().text_range().end();

        drop(doc);

        server
            .apply_edit(self.url, s, TextRange::new(end, end))
            .await?;

        Ok(true.into())
    }
}
