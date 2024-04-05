use lsp_types::{MessageType, Url};
use orgize::rowan::ast::AstNode;
use serde::{Deserialize, Serialize};

use crate::base::Server;

use crate::command::Executable;
use crate::utils::headline::find_headline;

#[derive(Deserialize, Serialize, Debug)]
pub struct HeadlineRemove {
    pub url: Url,
    pub line: u32,
}

impl Executable for HeadlineRemove {
    const NAME: &'static str = "headline-remove";

    type Result = bool;

    async fn execute<S: Server>(self, server: &S) -> anyhow::Result<bool> {
        let Some(doc) = server.documents().get(&self.url) else {
            server
                .log_message(
                    MessageType::WARNING,
                    format!("cannot find document with url {}", self.url),
                )
                .await;

            return Ok(false);
        };

        let Some(headline) = find_headline(&doc, self.line) else {
            server
                .log_message(
                    MessageType::WARNING,
                    format!("cannot find headline in line {}", self.line),
                )
                .await;

            return Ok(false);
        };

        drop(doc);

        let text_range = (move || headline.syntax().text_range())();

        server
            .apply_edit(self.url, String::new(), text_range)
            .await?;

        Ok(true)
    }
}

#[cfg(test)]
#[tokio::test]
async fn test() {
    use crate::test::TestServer;

    let server = TestServer::default();
    let url = Url::parse("test://test.org").unwrap();
    server.add_doc(url.clone(), "** \n* ".into());

    HeadlineRemove {
        line: 1,
        url: url.clone(),
    }
    .execute(&server)
    .await
    .unwrap();
    assert_eq!(server.get(&url), "* ");

    HeadlineRemove {
        line: 1,
        url: url.clone(),
    }
    .execute(&server)
    .await
    .unwrap();
    assert_eq!(server.get(&url), "");
}
