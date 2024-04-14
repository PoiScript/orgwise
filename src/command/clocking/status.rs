use chrono::NaiveDateTime;
use lsp_types::Url;
use orgize::{
    ast::Timestamp,
    export::{from_fn_with_ctx, Container, Event},
    rowan::ast::AstNode,
};
use serde::{Deserialize, Serialize};

use crate::backend::Backend;
use crate::command::Executable;

#[derive(Deserialize, Serialize)]
pub struct ClockingStatus {}

#[derive(Serialize, PartialEq, Debug)]
pub struct Result {
    running: Option<ClockingStatusResult>,
}

#[derive(Serialize, PartialEq, Debug)]
struct ClockingStatusResult {
    url: Url,
    line: u32,
    start: NaiveDateTime,
    title: String,
}

impl Executable for ClockingStatus {
    const NAME: &'static str = "clocking-status";

    type Result = Result;

    async fn execute<B: Backend>(self, backend: &B) -> anyhow::Result<Result> {
        let mut running: Option<ClockingStatusResult> = None;

        backend.documents().for_each(|url, doc| {
            doc.traverse(&mut from_fn_with_ctx(|event, ctx| match event {
                Event::Enter(Container::Headline(hdl)) => {
                    for clock in hdl.clocks() {
                        let Some(start) = clock
                            .syntax()
                            .children()
                            .find_map(Timestamp::cast)
                            .and_then(|ts| ts.start_to_chrono())
                        else {
                            continue;
                        };

                        if clock.is_running() && !matches!(&running, Some(r) if r.start >= start) {
                            running = Some(ClockingStatusResult {
                                url: url.clone(),
                                line: doc.line_of(hdl.start().into()) + 1,
                                start,
                                title: hdl.title_raw(),
                            });
                        }
                    }
                }

                Event::Enter(Container::Section(_)) => ctx.skip(),

                _ => {}
            }));
        });

        Ok(Result { running })
    }
}

#[cfg(test)]
#[tokio::test]
async fn test() {
    use chrono::NaiveDate;

    use crate::test::TestBackend;

    let backend = TestBackend::default();
    let url = Url::parse("test://test.org").unwrap();

    backend.documents().insert(
        url.clone(),
        &format!(
            r#"
* a
:LOGBOOK:
CLOCK: [2000-01-01 Web 00:00]--[2000-01-01 Web 01:00] => 01:00
CLOCK: [2000-01-02 Web 00:00]--[2000-01-03 Web 01:00] => 01:00
CLOCK: [2000-01-03 Web 00:00]--[2000-01-04 Web 01:00] => 01:00
CLOCK: [2000-01-04 Web 00:00]--[2000-01-05 Web 01:00] => 01:00
CLOCK: [2000-01-06 Web 00:00]
:END:
* b
:LOGBOOK:
CLOCK: [2000-01-05 Web 00:00]--[2000-01-06 Web 01:00] => 01:00
CLOCK: [2000-01-03 Web 00:00]--[2000-01-04 Web 01:00] => 01:00
CLOCK: [2000-01-07 Web 00:00]
:END:
"#,
        ),
    );

    let r = |day: u32, title: &str, line: u32| ClockingStatusResult {
        line,
        start: NaiveDate::from_ymd_opt(2000, 1, day)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap(),
        title: title.into(),
        url: url.clone(),
    };

    assert_eq!(
        ClockingStatus {}.execute(&backend).await.unwrap(),
        Result {
            running: Some(r(7, "b", 10)),
        }
    );
}
