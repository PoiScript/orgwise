use chrono::NaiveDateTime;
use lsp_types::Url;
use orgize::{
    export::{from_fn_with_ctx, Container, Event, HtmlExport, MarkdownExport},
    rowan::ast::AstNode,
    SyntaxKind,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::backend::Backend;

use crate::command::Executable;

#[derive(Deserialize, Debug, Serialize)]
pub struct HeadlineSearch {
    pub url: Option<Url>,
    #[serde(default)]
    pub markdown: bool,
    #[serde(default)]
    pub html: bool,
    pub from: Option<NaiveDateTime>,
    pub to: Option<NaiveDateTime>,
}

impl Executable for HeadlineSearch {
    const NAME: &'static str = "headline-search";

    type Result = Vec<Result>;

    async fn execute<B: Backend>(self, backend: &B) -> anyhow::Result<Vec<Result>> {
        let mut results = vec![];

        backend.documents().for_each(|url, doc| {
            if matches!(&self.url, Some(u) if url != u) {
                return;
            }

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
                    if ts.clone().all(|t| t < from) {
                        return;
                    }
                }

                if let Some(to) = self.to {
                    if ts.clone().all(|t| t > to) {
                        return;
                    }
                }

                results.push(Result {
                    title: headline.title_raw(),

                    url: url.clone(),
                    line: doc.line_of(headline.start().into()) + 1,
                    level: headline.level(),
                    priority: headline.priority().map(|t| t.to_string()),
                    tags: headline.tags().map(|t| t.to_string()).collect(),

                    section: headline.section().map(|t| {
                        t.syntax()
                            .children()
                            .filter(|n| n.kind() != SyntaxKind::DRAWER)
                            .fold(String::new(), |acc, node| acc + &node.to_string())
                    }),

                    section_html: headline.section().filter(|_| self.html).map(|section| {
                        let mut html = HtmlExport::default();
                        html.render(section.syntax());
                        html.finish()
                    }),

                    section_markdown: headline.section().filter(|_| self.markdown).map(|section| {
                        let mut md = MarkdownExport::default();
                        md.render(section.syntax());
                        md.finish()
                    }),

                    planning: Planning {
                        closed: headline
                            .planning()
                            .and_then(|t| t.closed())
                            .and_then(|t| t.start_to_chrono()),

                        deadline: headline
                            .planning()
                            .and_then(|t| t.deadline())
                            .and_then(|t| t.start_to_chrono()),

                        scheduled: headline
                            .planning()
                            .and_then(|t| t.scheduled())
                            .and_then(|t| t.start_to_chrono()),
                    },

                    clocking: Clocking {
                        start: headline
                            .clocks()
                            .filter(|x| x.is_running())
                            .filter_map(|x| x.value())
                            .find_map(|x| x.start_to_chrono()),
                        total_minutes: headline
                            .clocks()
                            .filter(|x| x.is_closed())
                            .filter_map(|x| x.value())
                            .filter_map(|x| Some(x.end_to_chrono()? - x.start_to_chrono()?))
                            .map(|x| x.num_minutes())
                            .sum(),
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

                    properties: headline
                        .properties()
                        .into_iter()
                        .flat_map(|p| p.iter().map(|(k, v)| (k.to_string(), v.to_string())))
                        .collect(),
                })
            }));
        });

        Ok(results)
    }
}

#[derive(Serialize)]
pub struct Result {
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
    #[serde(skip_serializing_if = "Option::is_none")]
    section_html: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    section_markdown: Option<String>,
    clocking: Clocking,
    properties: HashMap<String, String>,
}

#[derive(Serialize)]
struct Planning {
    deadline: Option<NaiveDateTime>,
    scheduled: Option<NaiveDateTime>,
    closed: Option<NaiveDateTime>,
}

#[derive(Serialize)]
struct Clocking {
    total_minutes: i64,
    start: Option<NaiveDateTime>,
}

#[derive(Serialize)]
pub struct Keyword {
    value: String,
    #[serde(rename = "type")]
    kind: &'static str,
}
