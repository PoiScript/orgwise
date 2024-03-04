use chrono::{DateTime, TimeZone, Utc};
use orgize::{
    export::{Container, Event, TraversalContext, Traverser},
    rowan::ast::AstNode,
    Org,
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct SearchOption {
    from: Option<DateTime<Utc>>,
    to: Option<DateTime<Utc>>,
}

#[derive(Serialize)]
pub struct Result {
    title: String,
    url: String,
    offset: usize,
    level: usize,
    priority: Option<String>,
    tags: Vec<String>,
    keyword: Option<String>,
    deadline: Option<DateTime<Utc>>,
    scheduled: Option<DateTime<Utc>>,
    closed: Option<DateTime<Utc>>,
}

pub fn search(option: &SearchOption, org: &Org) -> Vec<Result> {
    let mut t = SearchTraverser {
        option,
        results: vec![],
    };

    org.traverse(&mut t);

    t.results
}

struct SearchTraverser<'a> {
    option: &'a SearchOption,
    results: Vec<Result>,
}

impl<'a> Traverser for SearchTraverser<'a> {
    fn event(&mut self, event: Event, ctx: &mut TraversalContext) {
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

        if let Some(from) = self.option.from {
            if ts.clone().all(|t| t < from.naive_local()) {
                return;
            }
        }

        if let Some(to) = self.option.to {
            if ts.clone().all(|t| t > to.naive_local()) {
                return;
            }
        }

        self.results.push(Result {
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

            title: headline.title_raw(),
            url: String::new(),
            offset: headline.syntax().text_range().start().into(),
            level: headline.level(),
            priority: headline.priority().map(|t| t.to_string()),
            tags: headline.tags().map(|t| t.to_string()).collect(),
            keyword: headline.todo_keyword().map(|t| t.to_string()),
        })
    }
}
