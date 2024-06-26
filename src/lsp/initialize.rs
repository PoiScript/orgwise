use lsp_types::*;
use orgize::ParseConfig;
use serde::{Deserialize, Serialize};

use super::semantic_token;
use crate::backend::Backend;
use crate::command::OrgwiseCommand;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InitializationOptions {
    #[serde(default)]
    pub todo_keywords: Vec<String>,
    #[serde(default)]
    pub done_keywords: Vec<String>,
}

pub async fn initialize<B: Backend>(backend: &B, params: InitializeParams) -> InitializeResult {
    if let Some(initialization_options) = params
        .initialization_options
        .and_then(|o| serde_json::from_value::<InitializationOptions>(o).ok())
    {
        backend
            .log_message(
                MessageType::INFO,
                format!(
                    "Initialization options: {}",
                    serde_json::to_string(&initialization_options).unwrap_or_default()
                ),
            )
            .await;

        backend.documents().set_default_parse_config(ParseConfig {
            todo_keywords: (
                initialization_options.todo_keywords,
                initialization_options.done_keywords,
            ),
            ..Default::default()
        });
    }

    InitializeResult {
        server_info: None,
        offset_encoding: None,
        capabilities: ServerCapabilities {
            text_document_sync: Some(TextDocumentSyncCapability::Kind(
                TextDocumentSyncKind::INCREMENTAL,
            )),
            execute_command_provider: Some(ExecuteCommandOptions {
                commands: OrgwiseCommand::all(),
                work_done_progress_options: Default::default(),
            }),
            workspace: Some(WorkspaceServerCapabilities {
                workspace_folders: Some(WorkspaceFoldersServerCapabilities {
                    supported: Some(true),
                    change_notifications: Some(OneOf::Left(true)),
                }),
                file_operations: None,
            }),
            references_provider: Some(OneOf::Right(ReferencesOptions {
                work_done_progress_options: WorkDoneProgressOptions::default(),
            })),
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
                trigger_characters: Some(super::completion::trigger_characters()),
                ..Default::default()
            }),
            ..ServerCapabilities::default()
        },
    }
}
