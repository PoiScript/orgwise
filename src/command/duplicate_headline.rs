use lsp_types::{MessageType, Url};
use orgize::rowan::ast::AstNode;
use orgize::rowan::TextRange;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::base::Server;

use super::utils::find_headline;
use super::Executable;

#[derive(Deserialize, Serialize)]
pub struct DuplicateHeadline {
    pub url: Url,
    pub line: u32,
}

impl Executable for DuplicateHeadline {
    const NAME: &'static str = "duplicate-headline";

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

        let (new_text, range) = (move || {
            let end = headline.syntax().text_range().end();
            let text_range = TextRange::new(end, end);
            let new_text = headline.syntax().to_string();
            (new_text, text_range)
        })();

        drop(doc);

        server.apply_edit(self.url, new_text, range).await?;

        Ok(Value::Bool(true))
    }
}

#[cfg(test)]
#[tokio::test]
async fn test() {
    use crate::test::TestServer;

    let server = TestServer::default();
    let url = Url::parse("test://test.org").unwrap();
    server.add_doc(url.clone(), "* a\n* b\n * c".into());

    DuplicateHeadline {
        line: 1,
        url: url.clone(),
    }
    .execute(&server)
    .await
    .unwrap();
    assert_eq!(server.get(&url), "* a\n* a\n* b\n * c");

    DuplicateHeadline {
        line: 2,
        url: url.clone(),
    }
    .execute(&server)
    .await
    .unwrap();
    assert_eq!(server.get(&url), "* a\n* a\n* a\n* b\n * c");
}