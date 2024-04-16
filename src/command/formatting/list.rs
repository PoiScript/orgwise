use std::iter::once;

use orgize::{
    ast::ListItem,
    rowan::{ast::AstNode, TextRange, TextSize},
    SyntaxNode,
};

pub fn format(node: &SyntaxNode, indent_level: usize, edits: &mut Vec<(TextRange, String)>) {
    let mut items = node.children().filter_map(ListItem::cast);

    let Some(first_item) = items.next() else {
        return;
    };

    match first_item.bullet().trim_end() {
        expected_bullet @ ("-" | "+" | "*") => {
            if first_item.indent() != 3 * indent_level {
                edits.push((
                    TextRange::at(
                        first_item.start(),
                        TextSize::new(first_item.indent() as u32),
                    ),
                    " ".repeat(3 * indent_level),
                ));
            }

            for item in items {
                if item.indent() != 3 * indent_level {
                    edits.push((
                        TextRange::at(item.start(), TextSize::new(item.indent() as u32)),
                        " ".repeat(3 * indent_level),
                    ));
                }

                let bullet = item.bullet();
                let s = bullet.trim_end();
                if s != expected_bullet {
                    edits.push((
                        TextRange::at(bullet.start(), TextSize::new(s.len() as u32)),
                        expected_bullet.to_string(),
                    ));
                }
            }
        }
        b => {
            let c = if b.ends_with(')') { ')' } else { '.' };

            for (index, item) in once(first_item).chain(items).enumerate() {
                if item.indent() != 3 * indent_level {
                    edits.push((
                        TextRange::at(item.start(), TextSize::new(item.indent() as u32)),
                        " ".repeat(3 * indent_level),
                    ));
                }

                let expected_bullet = format!("{}{c}", index + 1);
                let bullet = item.bullet();
                let s = bullet.trim_end();
                if s != expected_bullet {
                    edits.push((
                        TextRange::at(bullet.start(), TextSize::new(s.len() as u32)),
                        expected_bullet,
                    ));
                }
            }
        }
    }
}

#[test]
fn test() {
    use crate::test_case;
    use orgize::ast::List;

    let format0 = |node: &SyntaxNode, edits: &mut Vec<(TextRange, String)>| format(node, 0, edits);

    let format2 = |node: &SyntaxNode, edits: &mut Vec<(TextRange, String)>| format(node, 2, edits);

    test_case!(List, "1.    item", format0, "1.    item");

    test_case!(
        List,
        "0. item\n- item\n+ item",
        format0,
        "1. item\n2. item\n3. item"
    );

    test_case!(
        List,
        " + item\n - item\n 1. item",
        format0,
        "+ item\n+ item\n+ item"
    );

    test_case!(
        List,
        " + item\n - item\n 1. item",
        format2,
        "      + item\n      + item\n      + item"
    );
}
