#![allow(deprecated)]

use lsp_types::*;
use orgize::{
    export::{from_fn_with_ctx, Container, Event},
    rowan::{ast::AstNode, TextSize},
    SyntaxKind,
};

use crate::backend::Backend;

pub fn document_symbol<B: Backend>(
    backend: &B,
    params: DocumentSymbolParams,
) -> Option<DocumentSymbolResponse> {
    backend
        .documents()
        .get_map(&params.text_document.uri, |doc| {
            let mut stack: Vec<usize> = vec![];
            let mut symbols: Vec<DocumentSymbol> = vec![];

            let mut handler = from_fn_with_ctx(|event, ctx| match event {
                Event::Enter(Container::Headline(headline)) => {
                    let mut s = &mut symbols;
                    for &i in &stack {
                        s = s[i].children.get_or_insert(vec![]);
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

                    let start = headline.start();
                    let end = headline.end() - TextSize::new(1);

                    stack.push(s.len());
                    s.push(DocumentSymbol {
                        children: None,
                        name,
                        detail: None,
                        kind: SymbolKind::STRING,
                        tags: Some(vec![]),
                        range: doc.range_of2(start, end),
                        selection_range: doc.range_of2(start, end),
                        deprecated: None,
                    });
                }
                Event::Leave(Container::Headline(_)) => {
                    stack.pop();
                }
                Event::Enter(Container::Section(_)) => ctx.skip(),
                _ => {}
            });
            doc.traverse(&mut handler);

            DocumentSymbolResponse::Nested(symbols)
        })
}
