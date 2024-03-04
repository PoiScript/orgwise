use dashmap::{DashMap, RwLock};
use lsp_types::{
    notification::{LogMessage, Notification, ShowMessage},
    request::{ApplyWorkspaceEdit, Request, WorkspaceConfiguration},
    *,
};
use orgize::{rowan::TextRange, ParseConfig};
use serde::{Deserialize, Serialize};
use std::path::Path;

use super::{commands::OrgizeCommand, completion, org_document::OrgDocument, semantic_token};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InitializationOptions {
    #[serde(default)]
    todo_keywords: Vec<String>,
    #[serde(default)]
    done_keywords: Vec<String>,
}

pub struct LanguageServerBase<E: FileSystem + LanguageClient + Process> {
    pub documents: DashMap<String, OrgDocument>,
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
    M: FileSystem + LanguageClient + Process,
{
    pub async fn initialize(&self, params: InitializeParams) -> InitializeResult {
        if let Some(initialization_options) = params
            .initialization_options
            .and_then(|o| serde_json::from_value::<InitializationOptions>(o).ok())
        {
            self.env
                .log_message(
                    MessageType::WARNING,
                    format!("Options: {:?}", initialization_options),
                )
                .await;

            self.parse_config.write().todo_keywords = (
                initialization_options.todo_keywords,
                initialization_options.done_keywords,
            );
        }

        InitializeResult {
            server_info: None,
            offset_encoding: None,
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::INCREMENTAL,
                )),
                execute_command_provider: Some(ExecuteCommandOptions {
                    commands: OrgizeCommand::all(),
                    work_done_progress_options: Default::default(),
                }),
                workspace: Some(WorkspaceServerCapabilities {
                    workspace_folders: Some(WorkspaceFoldersServerCapabilities {
                        supported: Some(true),
                        change_notifications: Some(OneOf::Left(true)),
                    }),
                    file_operations: None,
                }),
                semantic_tokens_provider: Some(
                    SemanticTokensServerCapabilities::SemanticTokensRegistrationOptions(
                        SemanticTokensRegistrationOptions {
                            text_document_registration_options: {
                                TextDocumentRegistrationOptions {
                                    document_selector: Some(vec![DocumentFilter {
                                        language: Some("org".to_string()),
                                        scheme: Some("file".to_string()),
                                        pattern: None,
                                    }]),
                                }
                            },
                            semantic_tokens_options: SemanticTokensOptions {
                                work_done_progress_options: WorkDoneProgressOptions::default(),
                                legend: SemanticTokensLegend {
                                    token_types: semantic_token::TYPES.into(),
                                    token_modifiers: semantic_token::MODIFIERS.into(),
                                },
                                range: Some(true),
                                full: Some(SemanticTokensFullOptions::Bool(true)),
                            },
                            static_registration_options: StaticRegistrationOptions::default(),
                        },
                    ),
                ),
                code_lens_provider: Some(CodeLensOptions {
                    resolve_provider: Some(true),
                }),
                folding_range_provider: Some(FoldingRangeProviderCapability::Simple(true)),
                document_link_provider: Some(DocumentLinkOptions {
                    resolve_provider: Some(true),
                    work_done_progress_options: WorkDoneProgressOptions::default(),
                }),
                document_formatting_provider: Some(OneOf::Left(true)),
                document_symbol_provider: Some(OneOf::Left(true)),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(false),
                    trigger_characters: Some(completion::trigger_characters()),
                    ..Default::default()
                }),
                ..ServerCapabilities::default()
            },
        }
    }

    pub async fn initialized(&self) {
        self.env
            .log_message(MessageType::WARNING, "Orgwise initialized".into())
            .await;
    }

    pub async fn did_change_configuration(&self, r: DidChangeConfigurationParams) {
        self.env
            .log_message(
                MessageType::INFO,
                format!("did_change_configuration: {r:?}"),
            )
            .await;
    }

    pub async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let r = self
            .env
            .send_request::<WorkspaceConfiguration>(ConfigurationParams { items: vec![] })
            .await;

        self.env
            .log_message(MessageType::INFO, format!("Did open configuration: {r:?}"))
            .await;

        let url = params.text_document.uri.to_string();

        self.documents.insert(
            url.clone(),
            OrgDocument::new(params.text_document.text, self.parse_config.read().clone()),
        );
    }

    pub fn did_change(&self, params: DidChangeTextDocumentParams) {
        let url = params.text_document.uri.to_string();

        for change in params.content_changes {
            if let (Some(mut doc), Some(range)) = (self.documents.get_mut(&url), change.range) {
                let start = doc.offset_of(range.start);
                let end = doc.offset_of(range.end);
                doc.update(start, end, &change.text);
            } else {
                self.documents.insert(
                    url.clone(),
                    OrgDocument::new(change.text, self.parse_config.read().clone()),
                );
            }
        }
    }

    pub fn completion_resolve(&self, params: CompletionItem) -> CompletionItem {
        params
    }

    pub fn code_action(&self, _: CodeActionParams) -> Option<CodeActionResponse> {
        None
    }
}

pub trait FileSystem {
    async fn write(&self, path: &Path, content: &str) -> anyhow::Result<()>;

    async fn write_range(&self, path: &Path, range: TextRange, content: &str)
        -> anyhow::Result<()>;

    async fn read_to_string(&self, path: &Path) -> anyhow::Result<String>;
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
