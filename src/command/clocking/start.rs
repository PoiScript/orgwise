use chrono::Local;
use lsp_types::{MessageType, Url};
use orgize::{
    rowan::{ast::AstNode, TextRange},
    SyntaxKind,
};
use serde::{Deserialize, Serialize};

use crate::{
    backend::Backend,
    command::Executable,
    utils::{clocking::find_logbook, headline::find_headline},
};

use super::FormatNativeDateTime;

#[derive(Deserialize, Serialize)]
pub struct ClockingStart {
    pub url: Url,
    pub line: u32,
}

impl Executable for ClockingStart {
    const NAME: &'static str = "clocking-start";

    const TITLE: Option<&'static str> = Some("Start clocking");

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

        let now = Local::now().naive_local();

        let (new_text, text_range) = (move || {
            if let Some(logbook) = find_logbook(&headline) {
                let node = logbook.syntax();
                let s = node
                    .children()
                    .find(|x| x.kind() == SyntaxKind::DRAWER_END)
                    .map(|x| x.text_range().start())
                    .unwrap_or_else(|| node.text_range().start());
                (
                    format!("CLOCK: {}\n", FormatNativeDateTime(now)),
                    TextRange::new(s, s),
                )
            } else {
                (
                    format!("\n:LOGBOOK:\nCLOCK: {}\n:END:\n", FormatNativeDateTime(now)),
                    TextRange::new(headline.end(), headline.end()),
                )
            }
        })();

        backend.apply_edit(self.url, new_text, text_range).await?;

        Ok(true)
    }
}

#[cfg(test)]
#[tokio::test]
async fn test() {
    use std::time::Duration;

    use chrono::TimeDelta;

    use crate::test::TestBackend;

    let backend = TestBackend::default();
    let url = Url::parse("test://test.org").unwrap();

    backend.documents().insert(url.clone(), r#"* a"#);

    let now = Local::now().naive_local();
    let _1h_ago = now - TimeDelta::from_std(Duration::from_secs(60 * 60)).unwrap();

    ClockingStart {
        url: url.clone(),
        line: 1,
    }
    .execute(&backend)
    .await
    .unwrap();

    ClockingStart {
        url: url.clone(),
        line: 1,
    }
    .execute(&backend)
    .await
    .unwrap();

    assert_eq!(
        backend.get(&url),
        format!(
            r#"* a
:LOGBOOK:
CLOCK: {}
CLOCK: {}
:END:
"#,
            FormatNativeDateTime(now),
            FormatNativeDateTime(now),
        )
    );
}
