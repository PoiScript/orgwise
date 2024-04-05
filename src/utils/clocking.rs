use orgize::{
    ast::{Drawer, Headline, Section},
    rowan::ast::AstNode,
};

pub fn find_logbook(headline: &Headline) -> Option<Drawer> {
    headline
        .syntax()
        .children()
        .flat_map(Section::cast)
        .flat_map(|x| x.syntax().children().filter_map(Drawer::cast))
        .find(|d| d.name().eq_ignore_ascii_case("LOGBOOK"))
}
