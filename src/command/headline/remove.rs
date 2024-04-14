use lsp_types::{MessageType, Url};
use serde::{Deserialize, Serialize};

use crate::backend::Backend;

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

    async fn execute<B: Backend>(self, backend: &B) -> anyhow::Result<bool> {
        let Some(headline) = backend
            .documents()
            .get_and_then(&self.url, |doc| find_headline(&doc, self.line))
        else {
            backend
                .log_message(
                    MessageType::WARNING,
                    format!("cannot find document with url {}", self.url),
                )
                .await;

            return Ok(false);
        };

        let text_range = (move || headline.text_range())();

        backend
            .apply_edit(self.url, String::new(), text_range)
            .await?;

        Ok(true)
    }
}

#[cfg(test)]
#[tokio::test]
async fn test() {
    use crate::test::TestBackend;

    let backend = TestBackend::default();
    let url = Url::parse("test://test.org").unwrap();
    backend.documents().insert(url.clone(), "** \n* ");

    HeadlineRemove {
        line: 1,
        url: url.clone(),
    }
    .execute(&backend)
    .await
    .unwrap();
    assert_eq!(backend.get(&url), "* ");

    HeadlineRemove {
        line: 1,
        url: url.clone(),
    }
    .execute(&backend)
    .await
    .unwrap();
    assert_eq!(backend.get(&url), "");
}
