use chrono::{DateTime, TimeZone, Utc};
use lsp_types::Url;
use orgize::{
    export::{from_fn_with_ctx, Container, Event},
    rowan::ast::AstNode,
    SyntaxKind,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::base::Server;

use super::Executable;

#[derive(Deserialize, Debug, Serialize)]
pub struct SearchHeadline {
    pub url: Option<Url>,
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
}

impl Executable for SearchHeadline {
    const NAME: &'static str = "search-headline";

    async fn execute<S: Server>(self, server: &S) -> anyhow::Result<Value> {
        let mut results = vec![];

        let iter = server.documents().iter().filter(|doc| {
            !matches!(
                &self.url,
                Some(url) if url != doc.key()
            )
        });

        for item in iter {
            let doc = item.value();

            doc.traverse(&mut from_fn_with_ctx(|event, ctx| {
                if let Event::Enter(Container::Section(_)) = event {
                    return ctx.skip();
                }

                let Event::Enter(Container::Headline(headline)) = event else {
                    return;
                };

                let ts = headline
                    .planning()
                    .and_then(|p| p.closed())
                    .into_iter()
                    .chain(headline.planning().and_then(|p| p.scheduled()))
                    .chain(headline.planning().and_then(|p| p.deadline()))
                    .filter_map(|t| t.start_to_chrono());

                if let Some(from) = self.from {
                    if ts.clone().all(|t| t < from.naive_local()) {
                        return;
                    }
                }

                if let Some(to) = self.to {
                    if ts.clone().all(|t| t > to.naive_local()) {
                        return;
                    }
                }

                results.push(Result {
                    title: headline.title_raw(),
                    url: item.key().clone(),
                    line: doc.line_of(headline.start().into()) + 1,
                    level: headline.level(),
                    priority: headline.priority().map(|t| t.to_string()),
                    tags: headline.tags().map(|t| t.to_string()).collect(),
                    section: headline.section().map(|t| t.raw()),

                    planning: Planning {
                        closed: headline
                            .planning()
                            .and_then(|t| t.closed())
                            .and_then(|t| t.start_to_chrono())
                            .map(|t| Utc.from_utc_datetime(&t)),
                        deadline: headline
                            .planning()
                            .and_then(|t| t.deadline())
                            .and_then(|t| t.start_to_chrono())
                            .map(|t| Utc.from_utc_datetime(&t)),
                        scheduled: headline
                            .planning()
                            .and_then(|t| t.scheduled())
                            .and_then(|t| t.start_to_chrono())
                            .map(|t| Utc.from_utc_datetime(&t)),
                    },

                    keyword: headline
                        .syntax()
                        .children_with_tokens()
                        .flat_map(|elem| elem.into_token())
                        .find_map(|token| match token.kind() {
                            SyntaxKind::HEADLINE_KEYWORD_TODO => Some(Keyword {
                                value: token.to_string(),
                                kind: "TODO",
                            }),
                            SyntaxKind::HEADLINE_KEYWORD_DONE => Some(Keyword {
                                value: token.to_string(),
                                kind: "DONE",
                            }),
                            _ => None,
                        }),
                })
            }));
        }

        Ok(serde_json::to_value(results)?)
    }
}

#[derive(Serialize)]
struct Result {
    title: String,
    url: Url,
    // zero-based
    line: u32,
    level: usize,
    priority: Option<String>,
    tags: Vec<String>,
    keyword: Option<Keyword>,
    planning: Planning,
    section: Option<String>,
}

#[derive(Serialize)]
struct Planning {
    deadline: Option<DateTime<Utc>>,
    scheduled: Option<DateTime<Utc>>,
    closed: Option<DateTime<Utc>>,
}

#[derive(Serialize)]
pub struct Keyword {
    value: String,
    #[serde(rename = "type")]
    kind: &'static str,
}
