use chrono::NaiveDateTime;
use lsp_types::{MessageType, Url};
use orgize::{ast::Timestamp, rowan::ast::AstNode};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    backend::Backend,
    command::Executable,
    utils::{headline::find_headline, timestamp::FormatInactiveTimestamp},
};

#[derive(Deserialize, Serialize)]
pub struct ClockingStop {
    pub url: Url,
    pub line: u32,
}

impl Executable for ClockingStop {
    const NAME: &'static str = "clocking-stop";

    const TITLE: Option<&'static str> = Some("Stop clocking");

    type Result = Value;

    async fn execute<B: Backend>(self, backend: &B) -> anyhow::Result<Value> {
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

            return Ok(Value::Null);
        };

        let now = now();

        let edits: Vec<_> = (move || {
            headline
                .clocks()
                .filter_map(|clock| {
                    if clock.is_closed() {
                        return None;
                    }

                    let start = clock
                        .syntax()
                        .children()
                        .find_map(Timestamp::cast)?
                        .start_to_chrono()?;

                    let duration = now - start;

                    Some((
                        self.url.clone(),
                        format!(
                            "CLOCK: {}--{} => {:0>2}:{:0>2}\n",
                            FormatInactiveTimestamp(start),
                            FormatInactiveTimestamp(now),
                            duration.num_hours(),
                            duration.num_minutes() % 60,
                        ),
                        clock.text_range(),
                    ))
                })
                .collect()
        })();

        backend.apply_edits(edits.into_iter()).await?;

        Ok(Value::Bool(true))
    }
}

#[cfg(not(test))]
#[inline]
fn now() -> NaiveDateTime {
    chrono::Local::now().naive_local()
}

#[cfg(test)]
#[inline]
fn now() -> NaiveDateTime {
    chrono::NaiveDate::from_ymd_opt(2000, 1, 1)
        .unwrap()
        .and_hms_opt(0, 0, 0)
        .unwrap()
}

#[cfg(test)]
#[tokio::test]
async fn test() {
    use std::time::Duration;

    use chrono::TimeDelta;

    use crate::test::TestBackend;

    let backend = TestBackend::default();
    let url = Url::parse("test://test.org").unwrap();

    let now = now();
    let _1h_ago = now - TimeDelta::from_std(Duration::from_secs(60 * 60)).unwrap();

    backend.documents().insert(
        url.clone(),
        format!(
            r#"
* a
:LOGBOOK:
CLOCK: {}
CLOCK: {}
:END:
"#,
            FormatInactiveTimestamp(now),
            FormatInactiveTimestamp(_1h_ago)
        ),
    );

    ClockingStop {
        url: url.clone(),
        line: 2,
    }
    .execute(&backend)
    .await
    .unwrap();

    assert_eq!(
        backend.get(&url),
        format!(
            r#"
* a
:LOGBOOK:
CLOCK: {}--{} => 00:00
CLOCK: {}--{} => 01:00
:END:
"#,
            FormatInactiveTimestamp(now),
            FormatInactiveTimestamp(now),
            FormatInactiveTimestamp(_1h_ago),
            FormatInactiveTimestamp(now),
        )
    );
}
