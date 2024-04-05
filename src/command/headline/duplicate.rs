use lsp_types::{MessageType, Url};
use orgize::rowan::TextRange;
use serde::{Deserialize, Serialize};

use crate::base::Server;

use crate::command::Executable;
use crate::utils::headline::find_headline;

#[derive(Deserialize, Serialize)]
pub struct HeadlineDuplicate {
    pub url: Url,
    pub line: u32,
}

impl Executable for HeadlineDuplicate {
    const NAME: &'static str = "headline-duplicate";

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

        let (new_text, range) = (move || {
            let end = headline.end();
            (headline.raw(), TextRange::new(end, end))
        })();

        drop(doc);

        server.apply_edit(self.url, new_text, range).await?;

        Ok(true)
    }
}

#[cfg(test)]
#[tokio::test]
async fn test() {
    use crate::test::TestServer;

    let server = TestServer::default();
    let url = Url::parse("test://test.org").unwrap();
    server.add_doc(url.clone(), "* a\n* b\n * c".into());

    HeadlineDuplicate {
        line: 1,
        url: url.clone(),
    }
    .execute(&server)
    .await
    .unwrap();
    assert_eq!(server.get(&url), "* a\n* a\n* b\n * c");

    HeadlineDuplicate {
        line: 2,
        url: url.clone(),
    }
    .execute(&server)
    .await
    .unwrap();
    assert_eq!(server.get(&url), "* a\n* a\n* a\n* b\n * c");
}