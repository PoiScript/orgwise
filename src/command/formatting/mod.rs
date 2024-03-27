use orgize::{
    export::{from_fn, Container, Event},
    rowan::{ast::AstNode, TextRange},
    Org,
};

mod blank_lines;
mod list;
mod rule;

pub fn formatting(org: &Org) -> Vec<(TextRange, String)> {
    let mut indent_level = 0;
    let mut edits: Vec<(TextRange, String)> = vec![];

    org.traverse(&mut from_fn(|event| match event {
        Event::Rule(rule) => {
            rule::format(rule.syntax(), &mut edits);
            blank_lines::format(rule.syntax(), &mut edits);
        }
        Event::Clock(clock) => {
            blank_lines::format(clock.syntax(), &mut edits);
        }

        Event::Enter(Container::Document(document)) => {
            blank_lines::format(document.syntax(), &mut edits);
        }
        Event::Enter(Container::Paragraph(paragraph)) => {
            blank_lines::format(paragraph.syntax(), &mut edits);
        }
        Event::Enter(Container::List(list)) => {
            list::format(list.syntax(), indent_level, &mut edits);
            blank_lines::format(list.syntax(), &mut edits);
            indent_level += 1;
        }
        Event::Leave(Container::List(_)) => {
            indent_level -= 1;
        }
        Event::Enter(Container::OrgTable(table)) => {
            blank_lines::format(table.syntax(), &mut edits);
        }
        Event::Enter(Container::SpecialBlock(block)) => {
            blank_lines::format(block.syntax(), &mut edits);
        }
        Event::Enter(Container::QuoteBlock(block)) => {
            blank_lines::format(block.syntax(), &mut edits);
        }
        Event::Enter(Container::CenterBlock(block)) => {
            blank_lines::format(block.syntax(), &mut edits);
        }
        Event::Enter(Container::VerseBlock(block)) => {
            blank_lines::format(block.syntax(), &mut edits);
        }
        Event::Enter(Container::CommentBlock(block)) => {
            blank_lines::format(block.syntax(), &mut edits);
        }
        Event::Enter(Container::ExampleBlock(block)) => {
            blank_lines::format(block.syntax(), &mut edits);
        }
        Event::Enter(Container::ExportBlock(block)) => {
            blank_lines::format(block.syntax(), &mut edits);
        }
        Event::Enter(Container::SourceBlock(block)) => {
            blank_lines::format(block.syntax(), &mut edits);
        }
        _ => {}
    }));

    edits
}

#[cfg(test)]
#[macro_export]
macro_rules! test_case {
    (
        $n:tt,
        $input:expr,
        $fn:expr,
        $expected:expr
    ) => {{
        use orgize::rowan::ast::AstNode;

        let org = orgize::Org::parse($input);
        let node = org.first_node::<$n>().unwrap();
        let node = node.syntax();

        let mut patches = vec![];

        $fn(&node, &mut patches);

        let input = node.to_string();

        patches.sort_by(|a, b| a.0.start().cmp(&b.0.start()));

        let mut i = 0;
        let mut output = String::new();
        for (range, text) in patches {
            let start = range.start().into();
            let end = range.end().into();
            output.push_str(&input[i..start]);
            output.push_str(&text);
            i = end;
        }
        output.push_str(&input[i..]);

        assert_eq!(output, $expected);
    }};
}
