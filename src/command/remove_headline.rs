use lsp_types::{MessageType, Url};
use orgize::rowan::ast::AstNode;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::base::Server;

use super::utils::find_headline;
use super::Executable;

#[derive(Deserialize, Serialize, Debug)]
pub struct RemoveHeadline {
    pub url: Url,
    pub line: u32,
}

impl Executable for RemoveHeadline {
    const NAME: &'static str = "remove-headline";

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

        let text_range = (move || headline.syntax().text_range())();

        server
            .apply_edit(self.url, String::new(), text_range)
            .await?;

        Ok(Value::Bool(true))
    }
}

#[cfg(test)]
#[tokio::test]
async fn test() {
    use crate::test::TestServer;

    let server = TestServer::default();
    let url = Url::parse("test://test.org").unwrap();
    server.add_doc(url.clone(), "** \n* ".into());

    RemoveHeadline {
        line: 1,
        url: url.clone(),
    }
    .execute(&server)
    .await
    .unwrap();
    assert_eq!(server.get(&url), "* ");

    RemoveHeadline {
        line: 1,
        url: url.clone(),
    }
    .execute(&server)
    .await
    .unwrap();
    assert_eq!(server.get(&url), "");
}
