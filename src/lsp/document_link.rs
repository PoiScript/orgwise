use lsp_types::*;
use orgize::{
    ast::{Link, SourceBlock},
    export::{Container, Event, TraversalContext, Traverser},
    rowan::ast::{support, AstNode},
    SyntaxKind,
};
use crate::common::header_argument;
use serde_json::Value;

use super::{org_document::OrgDocument, FileSystem, LanguageClient, LanguageServerBase, Process};

impl<E> LanguageServerBase<E>
where
    E: FileSystem<Location = Url> + LanguageClient + Process,
{
    pub fn document_link(&self, params: DocumentLinkParams) -> Option<Vec<DocumentLink>> {
        let doc = self.documents.get(&params.text_document.uri)?;

        let mut traverser = DocumentLinkTraverser {
            doc: &doc,
            links: vec![],
            base: params.text_document.uri,
        };

        doc.traverse(&mut traverser);

        Some(traverser.links)
    }

    pub fn document_link_resolve(&self, mut params: DocumentLink) -> DocumentLink {
        if params.target.is_some() {
            return params;
        }

        if let Some(data) = params.data.take() {
            params.target = self.resolve(data);
        }

        return params;
    }

    fn resolve(&self, data: Value) -> Option<Url> {
        let (typ, url, id): (String, Url, String) = serde_json::from_value(data).ok()?;

        match (typ.as_str(), url, id) {
            ("headline-id", mut url, id) => {
                let doc = self.documents.get(&url)?;

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

                Some(url)
            }
            ("resolve", base, path) => self.env.resolve_in(&path, &base).ok(),
            _ => None,
        }
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
            range: self.doc.range_of2(
                path.text_range().start().into(),
                path.text_range().end().into(),
            ),
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
            Event::Enter(Container::Headline(headline))
                if crate::common::headline_slug(&headline) == self.id =>
            {
                self.offset = Some(headline.begin());
                return ctx.stop();
            }
            Event::Enter(Container::Section(_)) => ctx.skip(),
            _ => {}
        }
    }
}
