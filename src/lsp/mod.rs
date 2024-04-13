pub mod code_lens;
pub mod completion;
pub mod document_link;
pub mod document_symbol;
pub mod execute_command;
pub mod folding_range;
pub mod formatting;
pub mod initialize;
pub mod references;
pub mod semantic_token;

pub use code_lens::*;
pub use completion::*;
pub use document_link::*;
pub use document_symbol::*;
pub use execute_command::*;
pub use folding_range::*;
pub use formatting::*;
pub use initialize::*;
pub use references::*;
pub use semantic_token::*;

use crate::backend::Backend;
use lsp_types::*;

pub async fn initialized<B: Backend>(backend: &B) {
    backend
        .log_message(MessageType::WARNING, "Initialized".into())
        .await;
}

pub fn did_change_configuration<B: Backend>(_: &B, _: DidChangeConfigurationParams) {}

pub fn did_open<B: Backend>(backend: &B, params: DidOpenTextDocumentParams) {
    backend.add_doc(params.text_document.uri, params.text_document.text);
}

pub async fn did_change<B: Backend>(backend: &B, params: DidChangeTextDocumentParams) {
    for change in params.content_changes {
        backend.update_doc(params.text_document.uri.clone(), change.range, change.text);
    }
}

pub fn code_action<B: Backend>(_: &B, _: CodeActionParams) -> Option<CodeActionResponse> {
    None
}
