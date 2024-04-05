use lsp_types::Position;
use orgize::ast::Headline;

use crate::base::OrgDocument;

pub fn find_headline(doc: &OrgDocument, line: u32) -> Option<Headline> {
    let offset = doc.offset_of(Position {
        line: line - 1,
        character: 0,
    });
    doc.org.node_at_offset(offset)
}

pub fn headline_slug(headline: &Headline) -> String {
    headline.title().fold(String::new(), |mut acc, elem| {
        for ch in elem.to_string().chars().filter(|c| c.is_ascii_graphic()) {
            acc.push(ch);
        }
        acc
    })
}
