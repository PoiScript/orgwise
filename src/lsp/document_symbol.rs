#![allow(deprecated)]

use lsp_types::*;
use orgize::{
    export::{Container, Event, TraversalContext, Traverser},
    rowan::ast::AstNode,
    SyntaxKind,
};

use super::{
    org_document::OrgDocument, FileSystem, LanguageClient, LanguageServerBase, Process,
};

impl<E> LanguageServerBase<E>
where
    E: FileSystem + LanguageClient + Process,
{
    pub fn document_symbol(&self, params: DocumentSymbolParams) -> Option<DocumentSymbolResponse> {
        let doc = self.documents.get(&params.text_document.uri)?;

        let mut t = DocumentSymbolTraverser {
            doc: &doc,
            stack: vec![],
            symbols: vec![],
        };

        doc.traverse(&mut t);
        Some(DocumentSymbolResponse::Nested(t.symbols))
    }
}

struct DocumentSymbolTraverser<'a> {
    doc: &'a OrgDocument,
    stack: Vec<usize>,
    symbols: Vec<DocumentSymbol>,
}

impl<'a> Traverser for DocumentSymbolTraverser<'a> {
    fn event(&mut self, event: Event, ctx: &mut TraversalContext) {
        match event {
            Event::Enter(Container::Headline(headline)) => {
                let mut symbols = &mut self.symbols;
                for &i in &self.stack {
                    symbols = symbols[i].children.get_or_insert(vec![]);
                }

                let name = headline
                    .syntax()
                    .children_with_tokens()
                    .filter(|n| {
                        n.kind() != SyntaxKind::HEADLINE_KEYWORD_DONE
                            && n.kind() != SyntaxKind::HEADLINE_KEYWORD_TODO
                            && n.kind() != SyntaxKind::HEADLINE_PRIORITY
                            && n.kind() != SyntaxKind::HEADLINE_TAGS
                    })
                    .take_while(|n| n.kind() != SyntaxKind::NEW_LINE)
                    .map(|n| n.to_string())
                    .collect::<String>();

                let start = headline.begin();
                let end = headline.end() - 1;

                self.stack.push(symbols.len());
                symbols.push(DocumentSymbol {
                    children: None,
                    name,
                    detail: None,
                    kind: SymbolKind::STRING,
                    tags: Some(vec![]),
                    range: self.doc.range_of2(start, end),
                    selection_range: self.doc.range_of2(start, end),
                    deprecated: None,
                });
            }
            Event::Leave(Container::Headline(_)) => {
                self.stack.pop();
            }
            Event::Enter(Container::Section(_)) => ctx.skip(),
            _ => {}
        }
    }
}
