use orgize::{rowan::TextRange, SyntaxKind, SyntaxNode};

pub fn format(node: &SyntaxNode, edits: &mut Vec<(TextRange, String)>) {
    let mut blank_lines = node
        .children_with_tokens()
        .filter_map(|e| e.into_token())
        .filter(|n| n.kind() == SyntaxKind::BLANK_LINE);

    let Some(first_line) = blank_lines.next() else {
        return;
    };

    if first_line.text() != "\n" {
        edits.push((first_line.text_range(), "\n".into()));
    }

    match (blank_lines.next(), blank_lines.last()) {
        (Some(first), Some(last)) => {
            edits.push((
                TextRange::new(first.text_range().start(), last.text_range().end()),
                "".into(),
            ));
        }
        (Some(first), None) => {
            edits.push((first.text_range(), "".into()));
        }
        _ => {}
    }
}

#[test]
fn test() {
    use crate::test_case;
    use orgize::ast::SourceBlock;

    test_case!(
        SourceBlock,
        "#+begin_src\n#+end_src\n\r\n\n\r",
        format,
        "#+begin_src\n#+end_src\n\n"
    );

    test_case!(
        SourceBlock,
        "#+begin_src\n#+end_src\n",
        format,
        "#+begin_src\n#+end_src\n"
    );

    test_case!(
        SourceBlock,
        "#+begin_src\n#+end_src",
        format,
        "#+begin_src\n#+end_src"
    );
}
