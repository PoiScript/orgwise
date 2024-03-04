mod code_lens;
pub mod commands;
pub mod completion;
pub mod document_link;
pub mod document_symbol;
pub mod folding_range;
pub mod formatting;
pub mod initialize;
pub mod org_document;
pub mod references;
pub mod semantic_token;

use std::fmt::Display;

use dashmap::{DashMap, RwLock};
use lsp_types::{
    notification::{LogMessage, Notification, ShowMessage},
    request::{ApplyWorkspaceEdit, Request},
    *,
};
use orgize::ParseConfig;

pub use lsp_types;

use org_document::OrgDocument;

pub struct LanguageServerBase<E: FileSystem + LanguageClient + Process> {
    pub documents: DashMap<Url, OrgDocument>,
    pub parse_config: RwLock<ParseConfig>,
    pub env: E,
}

impl<E> LanguageServerBase<E>
where
    E: FileSystem + LanguageClient + Process,
{
    pub fn new(env: E) -> Self {
        LanguageServerBase {
            documents: DashMap::new(),
            env,
            parse_config: RwLock::new(ParseConfig::default()),
        }
    }
}

impl<M> LanguageServerBase<M>
where
    M: FileSystem<Location = Url> + LanguageClient + Process,
{
    pub async fn initialized(&self) {
        self.env
            .log_message(MessageType::WARNING, "Initialized".into())
            .await;
    }

    pub fn did_change_configuration(&self, _: DidChangeConfigurationParams) {}

    pub fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.documents.insert(
            params.text_document.uri,
            OrgDocument::new(params.text_document.text, self.parse_config.read().clone()),
        );
    }

    pub async fn did_change(&self, params: DidChangeTextDocumentParams) {
        for change in params.content_changes {
            let config = self.parse_config.read().clone();
            if let (Some(mut doc), Some(range)) = (
                self.documents.get_mut(&params.text_document.uri),
                change.range,
            ) {
                let start = doc.offset_of(range.start);
                let end = doc.offset_of(range.end);
                doc.update(start, end, &change.text, config);
            } else {
                self.documents.insert(
                    params.text_document.uri.clone(),
                    OrgDocument::new(change.text, config),
                );
            }
        }
    }

    pub fn code_action(&self, _: CodeActionParams) -> Option<CodeActionResponse> {
        None
    }
}

pub trait FileSystem {
    type Location;

    async fn write(&self, path: &Self::Location, content: &str) -> anyhow::Result<()>;

    async fn read_to_string(&self, path: &Self::Location) -> anyhow::Result<String>;

    fn resolve_in(&self, path: &str, base: &Self::Location) -> anyhow::Result<Self::Location>;

    fn display(&self, path: &Self::Location) -> impl Display;
}

pub trait LanguageClient {
    async fn send_request<R: Request>(&self, params: R::Params) -> anyhow::Result<R::Result>;

    async fn send_notification<N: Notification>(&self, params: N::Params);

    async fn show_message(&self, typ: MessageType, message: String) {
        self.send_notification::<ShowMessage>(ShowMessageParams { typ, message })
            .await;
    }

    async fn log_message(&self, typ: MessageType, message: String) {
        self.send_notification::<LogMessage>(LogMessageParams { typ, message })
            .await;
    }

    async fn apply_edit(&self, edit: WorkspaceEdit) {
        let _ = self
            .send_request::<ApplyWorkspaceEdit>(ApplyWorkspaceEditParams { edit, label: None })
            .await;
    }
}

pub trait Process {
    async fn execute(&self, executable: &str, content: &str) -> anyhow::Result<String>;
}
