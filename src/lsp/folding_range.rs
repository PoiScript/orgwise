use lsp_types::*;
use orgize::{
    export::{from_fn, Container, Event},
    rowan::ast::AstNode,
    SyntaxKind, SyntaxNode,
};

use crate::backend::Backend;

pub fn folding_range<B: Backend>(
    backend: &B,
    params: FoldingRangeParams,
) -> Option<Vec<FoldingRange>> {
    let doc = backend.documents().get(&params.text_document.uri)?;

    let mut ranges: Vec<FoldingRange> = vec![];

    doc.traverse(&mut from_fn(|event| {
        let syntax = match &event {
            Event::Enter(Container::Headline(i)) => i.syntax(),
            Event::Enter(Container::OrgTable(i)) => i.syntax(),
            Event::Enter(Container::TableEl(i)) => i.syntax(),
            Event::Enter(Container::List(i)) => i.syntax(),
            Event::Enter(Container::Drawer(i)) => i.syntax(),
            Event::Enter(Container::DynBlock(i)) => i.syntax(),
            Event::Enter(Container::SpecialBlock(i)) => i.syntax(),
            Event::Enter(Container::QuoteBlock(i)) => i.syntax(),
            Event::Enter(Container::CenterBlock(i)) => i.syntax(),
            Event::Enter(Container::VerseBlock(i)) => i.syntax(),
            Event::Enter(Container::CommentBlock(i)) => i.syntax(),
            Event::Enter(Container::ExampleBlock(i)) => i.syntax(),
            Event::Enter(Container::ExportBlock(i)) => i.syntax(),
            Event::Enter(Container::SourceBlock(i)) => i.syntax(),
            _ => return,
        };

        let (start, end) = if syntax.kind() == SyntaxKind::HEADLINE {
            let range = syntax.text_range();
            (range.start().into(), range.end().into())
        } else {
            get_block_folding_range(syntax)
        };

        let start_line = doc.line_of(start);
        let end_line = doc.line_of(end - 1);

        if start_line != end_line {
            ranges.push(FoldingRange {
                start_line,
                end_line,
                kind: Some(FoldingRangeKind::Region),
                ..Default::default()
            });
        }
    }));

    Some(ranges)
}

fn get_block_folding_range(syntax: &SyntaxNode) -> (u32, u32) {
    let start: u32 = syntax.text_range().start().into();

    // don't include blank lines in folding range
    let end = syntax
        .children()
        .take_while(|n| n.kind() != SyntaxKind::BLANK_LINE)
        .last();

    let end: u32 = end.map(|n| n.text_range().end().into()).unwrap_or(start);

    (start, end)
}

#[test]
fn test() {
    use crate::test::TestBackend;

    let backend = TestBackend::default();
    let url = Url::parse("test://test.org").unwrap();
    backend.add_doc(url.clone(), "\n* a\n\n* b\n\n".into());

    let ranges = folding_range(
        &backend,
        FoldingRangeParams {
            text_document: TextDocumentIdentifier { uri: url.clone() },
            partial_result_params: PartialResultParams::default(),
            work_done_progress_params: WorkDoneProgressParams::default(),
        },
    )
    .unwrap();
    assert_eq!(ranges[0].start_line, 1);
    assert_eq!(ranges[0].end_line, 2);
    assert_eq!(ranges[1].start_line, 3);
    assert_eq!(ranges[1].end_line, 4);

    backend.add_doc(url.clone(), "\n\r\n#+begin_src\n#+end_src\n\r\r".into());
    let ranges = folding_range(
        &backend,
        FoldingRangeParams {
            text_document: TextDocumentIdentifier { uri: url.clone() },
            partial_result_params: PartialResultParams::default(),
            work_done_progress_params: WorkDoneProgressParams::default(),
        },
    )
    .unwrap();
    assert_eq!(ranges[0].start_line, 2);
    assert_eq!(ranges[0].end_line, 3);
}
