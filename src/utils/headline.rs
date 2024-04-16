use lsp_types::Position;
use orgize::ast::Headline;
use orgize::rowan::ast::AstNode;

use crate::backend::OrgDocument;

pub fn find_headline(doc: &OrgDocument, line: u32) -> Option<Headline> {
    let offset = doc.offset_of(Position {
        line: line - 1,
        character: 0,
    });

    let mut node = doc.org.document().syntax().clone();

    'l: loop {
        for hdl in node.children().filter_map(Headline::cast) {
            if hdl.start() == offset.into() {
                return Some(hdl);
            } else if hdl.start() < offset.into() && hdl.end() > offset.into() {
                node = hdl.syntax().clone();
                continue 'l;
            }
        }
        return None;
    }
}

pub fn headline_slug(headline: &Headline) -> String {
    headline.title().fold(String::new(), |mut acc, elem| {
        for ch in elem.to_string().chars().filter(|c| c.is_ascii_graphic()) {
            acc.push(ch);
        }
        acc
    })
}
