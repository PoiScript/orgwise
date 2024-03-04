use lsp_types::*;
use orgize::{
    export::{Container, Event, TraversalContext, Traverser},
    rowan::{ast::AstNode, TextRange, TokenAtOffset},
    Org, SyntaxKind, SyntaxToken,
};

use super::{
    org_document::OrgDocument, FileSystem, LanguageClient, LanguageServerBase, Process,
};

impl<E> LanguageServerBase<E>
where
    E: FileSystem + LanguageClient + Process,
{
    pub fn references(&self, params: ReferenceParams) -> Option<Vec<Location>> {
        let doc = self
            .documents
            .get(&params.text_document_position.text_document.uri)?;

        let offset = doc.offset_of(params.text_document_position.position);

        let symbol = locate_symbol(&doc.org, offset)?;

        let mut locations = vec![];

        for entry in &self.documents {
            let mut traverser = ReferencesTraverser {
                doc: &entry.value(),
                locations: &mut locations,
                symbol: &symbol,
                url: &entry.key(),
            };
            entry.value().traverse(&mut traverser);
        }

        Some(locations)
    }
}

struct ReferencesTraverser<'a> {
    doc: &'a OrgDocument,
    url: &'a Url,
    locations: &'a mut Vec<Location>,
    symbol: &'a Symbol,
}

impl<'a> Traverser for ReferencesTraverser<'a> {
    fn event(&mut self, event: Event, ctx: &mut TraversalContext) {
        match event {
            Event::Enter(Container::Headline(headline)) => {
                for children in headline.syntax().children_with_tokens() {
                    match (children.kind(), &self.symbol) {
                        (SyntaxKind::HEADLINE_KEYWORD_DONE, Symbol::Keyword(keyword))
                        | (SyntaxKind::HEADLINE_KEYWORD_TODO, Symbol::Keyword(keyword)) => {
                            if let Some(token) = children.as_token() {
                                if token.text() == keyword.text() {
                                    self.add_range(token.text_range())
                                }
                            }
                            break;
                        }
                        (SyntaxKind::HEADLINE_PRIORITY, Symbol::Priority(priority)) => {
                            let is_match = children
                                .as_node()
                                .into_iter()
                                .flat_map(|n| n.children_with_tokens())
                                .flat_map(|c| c.into_token())
                                .find(|t| t.kind() == SyntaxKind::TEXT)
                                .and_then(|t| t.text().parse::<char>().ok())
                                .map(|c| c == *priority)
                                .unwrap_or_default();

                            if is_match {
                                self.add_range(children.text_range())
                            }
                            break;
                        }
                        (SyntaxKind::HEADLINE_TAGS, Symbol::Tag(tag)) => {
                            let token = children
                                .as_node()
                                .into_iter()
                                .flat_map(|n| n.children_with_tokens())
                                .flat_map(|c| c.into_token())
                                .filter(|t| t.kind() == SyntaxKind::TEXT)
                                .find(|t| t.text() == tag.text());

                            if let Some(token) = token {
                                self.add_range(token.text_range())
                            }
                            break;
                        }
                        (SyntaxKind::NEW_LINE, _) => break,
                        _ => {}
                    }
                }
            }

            Event::Enter(Container::Section(_)) => ctx.skip(),

            _ => {}
        }
    }
}

impl<'a> ReferencesTraverser<'a> {
    fn add_range(&mut self, range: TextRange) {
        let range = self.doc.range_of(range);

        self.locations.push(Location {
            uri: self.url.clone(),
            range,
        });
    }
}

enum Symbol {
    Keyword(SyntaxToken),
    Priority(char),
    Tag(SyntaxToken),
}

fn locate_symbol(org: &Org, offset: u32) -> Option<Symbol> {
    let (t1, t2) = match org.document().syntax().token_at_offset(offset.into()) {
        TokenAtOffset::None => return None,
        TokenAtOffset::Single(t1) => (t1, None),
        TokenAtOffset::Between(t1, t2) => (t1, Some(t2)),
    };

    if matches!(
        t1.kind(),
        SyntaxKind::HEADLINE_KEYWORD_DONE | SyntaxKind::HEADLINE_KEYWORD_TODO
    ) {
        return Some(Symbol::Keyword(t1));
    }

    let p1 = t1.parent()?;
    let p2 = t2.and_then(|t| t.parent());

    match (p1.kind(), p2.as_ref().map(|p| p.kind())) {
        (_, Some(SyntaxKind::HEADLINE_PRIORITY)) => {
            let c = p2?
                .children_with_tokens()
                .filter_map(|it| it.into_token())
                .find(|it| it.kind() == SyntaxKind::TEXT)?
                .text()
                .parse()
                .ok()?;
            Some(Symbol::Priority(c))
        }
        (SyntaxKind::HEADLINE_PRIORITY, _) => {
            let c = p1
                .children_with_tokens()
                .filter_map(|it| it.into_token())
                .find(|it| it.kind() == SyntaxKind::TEXT)?
                .text()
                .parse()
                .ok()?;
            Some(Symbol::Priority(c))
        }
        (SyntaxKind::HEADLINE_TAGS, _) if t1.kind() == SyntaxKind::TEXT => Some(Symbol::Tag(t1)),
        _ => None,
    }
}

#[test]
fn test() {
    let org = "* TODO [#A] hello :abc: :edf:";
    let org = Org::parse(org);

    for i in 3..=6 {
        let symbol = locate_symbol(&org, i).unwrap();
        assert!(matches!(symbol, Symbol::Keyword(k) if k.text() == "TODO"));
    }

    for i in 7..=11 {
        let symbol = locate_symbol(&org, i).unwrap();
        assert!(matches!(symbol, Symbol::Priority('A')));
    }

    for i in 20..=21 {
        let symbol = locate_symbol(&org, i).unwrap();
        assert!(matches!(symbol, Symbol::Tag(t) if t.text() == "abc"));
    }

    for i in 26..=27 {
        let symbol = locate_symbol(&org, i).unwrap();
        assert!(matches!(symbol, Symbol::Tag(t) if t.text() == "edf"));
    }
}
