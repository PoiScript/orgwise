use lsp_types::{MessageType, Url};
use orgize::rowan::TextRange;
use serde::{Deserialize, Serialize};

use crate::backend::Backend;

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

        let (new_text, range) = (move || (headline.raw(), TextRange::empty(headline.end())))();

        backend.apply_edit(self.url, new_text, range).await?;

        Ok(true)
    }
}

#[cfg(test)]
#[tokio::test]
async fn test() {
    use crate::test::TestBackend;

    let backend = TestBackend::default();
    let url = Url::parse("test://test.org").unwrap();
    backend.documents().insert(url.clone(), "* a\n* b\n * c");

    HeadlineDuplicate {
        line: 1,
        url: url.clone(),
    }
    .execute(&backend)
    .await
    .unwrap();
    assert_eq!(backend.get(&url), "* a\n* a\n* b\n * c");

    HeadlineDuplicate {
        line: 2,
        url: url.clone(),
    }
    .execute(&backend)
    .await
    .unwrap();
    assert_eq!(backend.get(&url), "* a\n* a\n* a\n* b\n * c");
}
