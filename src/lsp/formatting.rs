use crate::backend::Backend;
use lsp_types::*;

pub fn formatting<B: Backend>(
    backend: &B,
    params: DocumentFormattingParams,
) -> Option<Vec<TextEdit>> {
    backend
        .documents()
        .get_map(&params.text_document.uri, |doc| {
            crate::command::formatting::formatting(&doc.org)
                .into_iter()
                .map(|(range, content)| TextEdit {
                    range: doc.range_of(range),
                    new_text: content,
                })
                .collect::<Vec<_>>()
        })
}
