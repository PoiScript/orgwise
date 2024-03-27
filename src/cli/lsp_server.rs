use dashmap::{DashMap, RwLock};
use lsp_types::notification::ShowMessage;
use lsp_types::{notification::LogMessage, request::ApplyWorkspaceEdit};
use orgize::rowan::TextRange;
use orgize::ParseConfig;
use serde_json::Value;
use std::collections::HashMap;
use tower_lsp::{jsonrpc::Result, lsp_types::*, Client, LanguageServer, LspService, Server};

use crate::base::OrgDocument;

use crate::base::Server as BaseS;
use crate::lsp;

struct TowerLspServer {
    client: Client,
    documents: DashMap<Url, OrgDocument>,
    parse_config: RwLock<ParseConfig>,
}

impl BaseS for TowerLspServer {
    fn home_dir(&self) -> Option<Url> {
        dirs::home_dir().and_then(|d| Url::from_file_path(d).ok())
    }

    async fn write(&self, url: &Url, content: &str) -> anyhow::Result<()> {
        if let Ok(path) = url.to_file_path() {
            tokio::fs::write(path, content).await?;
            Ok(())
        } else {
            anyhow::bail!("Cannot convert Url to PathBuf")
        }
    }

    async fn read_to_string(&self, url: &Url) -> anyhow::Result<String> {
        if let Ok(path) = url.to_file_path() {
            Ok(tokio::fs::read_to_string(path).await?)
        } else {
            anyhow::bail!("Cannot convert Url to PathBuf")
        }
    }

    async fn execute(&self, executable: &str, content: &str) -> anyhow::Result<String> {
        let dir = tempfile::tempdir()?;

        let path = dir.path().join(".orgize");

        tokio::fs::write(&path, content).await?;

        let output = tokio::process::Command::new(executable)
            .arg(&path)
            .output()
            .await?;

        let output = String::from_utf8_lossy(&output.stdout);

        Ok(output.to_string())
    }

    async fn log_message(&self, typ: MessageType, message: String) {
        self.client
            .send_notification::<LogMessage>(LogMessageParams { typ, message })
            .await;
    }

    async fn show_message(&self, typ: MessageType, message: String) {
        self.client
            .send_notification::<ShowMessage>(ShowMessageParams { typ, message })
            .await;
    }

    async fn apply_edits(
        &self,
        items: impl Iterator<Item = (Url, String, TextRange)>,
    ) -> anyhow::Result<()> {
        let mut changes: HashMap<Url, Vec<TextEdit>> = HashMap::new();

        for (url, new_text, range) in items {
            if let Some(doc) = self.documents.get(&url) {
                let edit = TextEdit {
                    new_text,
                    range: doc.range_of(range),
                };
                changes
                    .entry(url.clone())
                    .and_modify(|edits| edits.push(edit.clone()))
                    .or_insert_with(|| vec![edit]);
            }
        }

        let edit = ApplyWorkspaceEditParams {
            label: None,
            edit: WorkspaceEdit {
                changes: Some(changes),
                ..Default::default()
            },
        };

        self.client.send_request::<ApplyWorkspaceEdit>(edit).await?;

        Ok(())
    }

    fn documents(&self) -> &DashMap<Url, OrgDocument> {
        &self.documents
    }

    fn default_parse_config(&self) -> ParseConfig {
        let lock = self.parse_config.read();
        let config = lock.clone();
        drop(lock);
        config
    }

    fn set_default_parse_config(&self, config: ParseConfig) {
        *self.parse_config.write() = config;
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for TowerLspServer {
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        Ok(lsp::initialize(self, params).await)
    }

    async fn initialized(&self, _: InitializedParams) {
        lsp::initialized(self).await;
    }

    async fn shutdown(&self) -> Result<()> {
        self.log_message(MessageType::INFO, "Orgize LSP shutdown".into())
            .await;
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        lsp::did_open(self, params);
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        lsp::did_change(self, params).await
    }

    async fn did_save(&self, _: DidSaveTextDocumentParams) {}

    async fn did_close(&self, _: DidCloseTextDocumentParams) {}

    async fn did_change_configuration(&self, params: DidChangeConfigurationParams) {
        lsp::did_change_configuration(self, params);
    }

    async fn did_change_workspace_folders(&self, _: DidChangeWorkspaceFoldersParams) {}

    async fn did_change_watched_files(&self, _: DidChangeWatchedFilesParams) {}

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        Ok(lsp::completion(self, params))
    }

    async fn completion_resolve(&self, params: CompletionItem) -> Result<CompletionItem> {
        Ok(lsp::completion_resolve(self, params))
    }

    async fn semantic_tokens_full(
        &self,
        params: SemanticTokensParams,
    ) -> Result<Option<SemanticTokensResult>> {
        Ok(lsp::semantic_tokens_full(self, params))
    }

    async fn semantic_tokens_range(
        &self,
        params: SemanticTokensRangeParams,
    ) -> Result<Option<SemanticTokensRangeResult>> {
        Ok(lsp::semantic_tokens_range(self, params))
    }

    async fn document_link(&self, params: DocumentLinkParams) -> Result<Option<Vec<DocumentLink>>> {
        Ok(lsp::document_link(self, params))
    }

    async fn document_link_resolve(&self, params: DocumentLink) -> Result<DocumentLink> {
        Ok(lsp::document_link_resolve(self, params))
    }

    async fn folding_range(&self, params: FoldingRangeParams) -> Result<Option<Vec<FoldingRange>>> {
        Ok(lsp::folding_range(self, params))
    }

    async fn code_lens(&self, params: CodeLensParams) -> Result<Option<Vec<CodeLens>>> {
        Ok(lsp::code_lens(self, params))
    }

    async fn code_lens_resolve(&self, params: CodeLens) -> Result<CodeLens> {
        Ok(lsp::code_lens_resolve(self, params))
    }

    async fn code_action(&self, params: CodeActionParams) -> Result<Option<CodeActionResponse>> {
        Ok(lsp::code_action(self, params))
    }

    async fn formatting(&self, params: DocumentFormattingParams) -> Result<Option<Vec<TextEdit>>> {
        Ok(lsp::formatting(self, params))
    }

    async fn execute_command(&self, params: ExecuteCommandParams) -> Result<Option<Value>> {
        Ok(lsp::execute_command(self, params).await)
    }

    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> Result<Option<DocumentSymbolResponse>> {
        Ok(lsp::document_symbol(self, params))
    }

    async fn references(&self, params: ReferenceParams) -> Result<Option<Vec<Location>>> {
        Ok(lsp::references(self, params))
    }
}

pub async fn start() -> anyhow::Result<()> {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::build(|client| TowerLspServer {
        client,
        documents: DashMap::new(),
        parse_config: RwLock::new(ParseConfig::default()),
    })
    .finish();

    Server::new(stdin, stdout, socket).serve(service).await;

    Ok(())
}
