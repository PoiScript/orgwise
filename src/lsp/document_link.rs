use lsp_types::*;
use orgize::{
    ast::{Link, SourceBlock},
    export::{Container, Event, TraversalContext, Traverser},
    rowan::ast::{support, AstNode},
    SyntaxKind,
};
use serde_json::Value;

use crate::backend::{Backend, OrgDocument};
use crate::utils::headline::headline_slug;
use crate::utils::src_block::header_argument;

pub fn document_link<B: Backend>(
    backend: &B,
    params: DocumentLinkParams,
) -> Option<Vec<DocumentLink>> {
    backend
        .documents()
        .get_map(&params.text_document.uri.clone(), |doc| {
            let mut traverser = DocumentLinkTraverser {
                doc: &doc,
                links: vec![],
                base: params.text_document.uri,
            };

            doc.traverse(&mut traverser);

            traverser.links
        })
}

pub fn document_link_resolve<B: Backend>(backend: &B, mut params: DocumentLink) -> DocumentLink {
    if params.target.is_some() {
        return params;
    }

    if let Some(data) = params.data.take() {
        params.target = resolve(backend, data);
    }

    params
}

fn resolve<B: Backend>(backend: &B, data: Value) -> Option<Url> {
    let (typ, url, id): (String, Url, String) = serde_json::from_value(data).ok()?;

    match (typ.as_str(), url, id) {
        ("headline-id", mut url, id) => {
            backend.documents().get_map(&url.clone(), |doc| {
                let mut h = HeadlineIdTraverser {
                    id: id.to_string(),
                    offset: None,
                };

                doc.traverse(&mut h);

                if let Some(offset) = h.offset.take() {
                    let line = doc.line_of(offset);
                    // results is zero-based
                    url.set_fragment(Some(&(line + 1).to_string()));
                }

                url
            })
        }
        ("resolve", base, path) => backend.resolve_in(&path, &base).ok(),
        _ => None,
    }
}

struct DocumentLinkTraverser<'a> {
    doc: &'a OrgDocument,
    links: Vec<DocumentLink>,
    base: Url,
}

impl<'a> Traverser for DocumentLinkTraverser<'a> {
    fn event(&mut self, event: Event, ctx: &mut TraversalContext) {
        match event {
            Event::Enter(Container::Link(link)) => {
                if let Some(link) = self.link_path(link) {
                    self.links.push(link);
                }
                ctx.skip();
            }
            Event::Enter(Container::SourceBlock(block)) => {
                if let Some(link) = self.block_tangle(block) {
                    self.links.push(link);
                }
                ctx.skip();
            }

            _ => {}
        }
    }
}

impl<'a> DocumentLinkTraverser<'a> {
    fn link_path(&self, link: Link) -> Option<DocumentLink> {
        let path = support::token(link.syntax(), SyntaxKind::LINK_PATH)
            .or_else(|| support::token(link.syntax(), SyntaxKind::TEXT))?;

        let path_str = path.text();

        let (target, data) = if let Some(file) = path_str.strip_prefix("file:") {
            (
                None,
                serde_json::to_value(("resolve", &self.base, file)).ok(),
            )
        } else if path_str.starts_with('/')
            || path_str.starts_with("./")
            || path_str.starts_with("~/")
        {
            (
                None,
                serde_json::to_value(("resolve", &self.base, path_str)).ok(),
            )
        } else if path_str.starts_with("http://") || path_str.starts_with("https://") {
            (Some(Url::parse(path_str).ok()?), None)
        } else if let Some(id) = path_str.strip_prefix('#') {
            (
                None,
                serde_json::to_value(("headline-id", &self.base, id)).ok(),
            )
        } else {
            return None;
        };

        Some(DocumentLink {
            range: self.doc.range_of(path.text_range()),
            tooltip: Some("Jump to link".into()),
            target,
            data,
        })
    }

    fn block_tangle(&self, block: SourceBlock) -> Option<DocumentLink> {
        let parameters = block
            .syntax()
            .children()
            .find(|e| e.kind() == SyntaxKind::BLOCK_BEGIN)
            .into_iter()
            .flat_map(|n| n.children_with_tokens())
            .filter_map(|n| n.into_token())
            .find(|n| n.kind() == SyntaxKind::SRC_BLOCK_PARAMETERS)?;

        let tangle = header_argument(parameters.text(), "", "", ":tangle", "no");

        if tangle == "no" {
            return None;
        }

        let start: u32 = parameters.text_range().start().into();

        let index = parameters.text().find(tangle).unwrap_or_default() as u32;

        let len = tangle.len() as u32;

        Some(DocumentLink {
            range: self.doc.range_of2(start + index, start + index + len),
            tooltip: Some("Jump to tangle destination".into()),
            target: None,
            data: serde_json::to_value(("resolve", &self.base, tangle)).ok(),
        })
    }
}

struct HeadlineIdTraverser {
    id: String,
    offset: Option<u32>,
}

impl Traverser for HeadlineIdTraverser {
    fn event(&mut self, event: Event, ctx: &mut TraversalContext) {
        match event {
            Event::Enter(Container::Headline(headline)) if headline_slug(&headline) == self.id => {
                self.offset = Some(headline.start().into());
                ctx.stop()
            }
            Event::Enter(Container::Section(_)) => ctx.skip(),
            _ => {}
        }
    }
}
