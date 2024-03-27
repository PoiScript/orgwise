use chrono::NaiveDateTime;
use lsp_types::Url;
use orgize::{
    ast::Timestamp,
    export::{from_fn_with_ctx, Container, Event},
    rowan::ast::AstNode,
};
use serde::{Deserialize, Serialize};

use super::Executable;
use crate::base::Server;

#[derive(Deserialize, Serialize)]
pub struct ClockingStatus {}

#[derive(Serialize)]
struct Result {
    last: Option<ClockingStatusResult>,
    running: Option<ClockingStatusResult>,
}

#[derive(Serialize)]
struct ClockingStatusResult {
    url: Url,
    line: u32,
    start: NaiveDateTime,
    title: String,
}

impl Executable for ClockingStatus {
    const NAME: &'static str = "clocking-status";

    async fn execute<S: Server>(self, server: &S) -> anyhow::Result<serde_json::Value> {
        let mut last: Option<ClockingStatusResult> = None;
        let mut running: Option<ClockingStatusResult> = None;

        for doc in server.documents() {
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
                                url: doc.key().clone(),
                                line: doc.line_of(hdl.syntax().text_range().start().into()),
                                start,
                                title: hdl.title_raw(),
                            });
                        }

                        if !matches!(&last, Some(l) if l.start >= start) {
                            last = Some(ClockingStatusResult {
                                url: doc.key().clone(),
                                line: doc.line_of(hdl.syntax().text_range().start().into()),
                                start,
                                title: hdl.title_raw(),
                            })
                        }
                    }
                }

                Event::Enter(Container::Section(_)) => ctx.skip(),

                _ => {}
            }));
        }

        Ok(serde_json::to_value(Result { last, running })?)
    }
}

#[cfg(test)]
#[tokio::test]
async fn test() {}
