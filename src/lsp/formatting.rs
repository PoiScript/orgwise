use crate::backend::Backend;
use lsp_types::*;

pub fn formatting<B: Backend>(
    backend: &B,
    params: DocumentFormattingParams,
) -> Option<Vec<TextEdit>> {
    let doc = backend.documents().get(&params.text_document.uri)?;

    let edits = crate::command::formatting::formatting(&doc.org)
        .into_iter()
        .map(|(range, content)| TextEdit {
            range: doc.range_of(range),
            new_text: content,
        })
        .collect::<Vec<_>>();

    Some(edits)
}
