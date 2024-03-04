use lsp_types::*;

use super::{FileSystem, LanguageClient, LanguageServerBase, Process};

impl<E> LanguageServerBase<E>
where
    E: FileSystem + LanguageClient + Process,
{
    pub fn formatting(&self, params: DocumentFormattingParams) -> Option<Vec<TextEdit>> {
        let doc = self.documents.get(&params.text_document.uri)?;

        let edits = crate::common::formatting(&doc.org)
            .into_iter()
            .map(|(start, end, content)| TextEdit {
                range: doc.range_of2(start as u32, end as u32),
                new_text: content,
            })
            .collect::<Vec<_>>();

        Some(edits)
    }
}
