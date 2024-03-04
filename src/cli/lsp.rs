use lsp_types::{notification::Notification, request::Request};
use serde_json::Value;
use tower_lsp::{jsonrpc::Result, lsp_types::*, Client, LanguageServer, LspService, Server};

use crate::lsp::{FileSystem, LanguageClient, LanguageServerBase, Process};

pub struct OrgServer(LanguageServerBase<TowerEnvironment>);

struct TowerEnvironment {
    client: Client,
}

impl FileSystem for TowerEnvironment {
    type Location = Url;

    async fn write(&self, path: &Url, content: &str) -> anyhow::Result<()> {
        let path = path.to_file_path().unwrap();
        tokio::fs::write(path, content).await?;
        Ok(())
    }

    async fn read_to_string(&self, path: &Url) -> anyhow::Result<String> {
        let path = path.to_file_path().unwrap();
        if path.exists() {
            Ok(tokio::fs::read_to_string(path).await?)
        } else {
            Ok(String::new())
        }
    }

    fn resolve_in(&self, path: &str, base: &Url) -> anyhow::Result<Url> {
        if path.starts_with("~/") {
            if let Some(home) = dirs::home_dir() {
                return Ok(Url::parse(&format!(
                    "file://{}{}",
                    home.display(),
                    &path[1..]
                ))?);
            }
        }

        let options = Url::options().base_url(Some(base));
        Ok(options.parse(path)?)
    }

    fn display(&self, path: &Url) -> impl std::fmt::Display {
        path.to_string()
    }
}

impl LanguageClient for TowerEnvironment {
    async fn send_request<R: Request>(&self, params: R::Params) -> anyhow::Result<R::Result> {
        Ok(self.client.send_request::<R>(params).await?)
    }

    async fn send_notification<N: Notification>(&self, params: N::Params) {
        self.client.send_notification::<N>(params).await;
    }
}

impl Process for TowerEnvironment {
    async fn execute(&self, executable: &str, content: &str) -> anyhow::Result<String> {
        let dir = tempfile::tempdir()?;

        let path = dir.path().join(".orgwise");

        tokio::fs::write(&path, content).await?;

        let output = tokio::process::Command::new(executable)
            .arg(&path)
            .output()
            .await?;

        let output = String::from_utf8_lossy(&output.stdout);

        Ok(output.to_string())
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for OrgServer {
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        Ok(self.0.initialize(params).await)
    }

    async fn initialized(&self, _: InitializedParams) {
        self.0.initialized().await;
    }

    async fn shutdown(&self) -> Result<()> {
        self.0
            .env
            .show_message(MessageType::INFO, "Orgize LSP shutdown".into())
            .await;
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.0.did_open(params);
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        self.0.did_change(params).await
    }

    async fn did_save(&self, _: DidSaveTextDocumentParams) {}

    async fn did_close(&self, _: DidCloseTextDocumentParams) {}

    async fn did_change_configuration(&self, params: DidChangeConfigurationParams) {
        self.0.did_change_configuration(params);
    }

    async fn did_change_workspace_folders(&self, _: DidChangeWorkspaceFoldersParams) {}

    async fn did_change_watched_files(&self, _: DidChangeWatchedFilesParams) {}

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        Ok(self.0.completion(params))
    }

    async fn completion_resolve(&self, params: CompletionItem) -> Result<CompletionItem> {
        Ok(self.0.completion_resolve(params))
    }

    async fn semantic_tokens_full(
        &self,
        params: SemanticTokensParams,
    ) -> Result<Option<SemanticTokensResult>> {
        Ok(self.0.semantic_tokens_full(params))
    }

    async fn semantic_tokens_range(
        &self,
        params: SemanticTokensRangeParams,
    ) -> Result<Option<SemanticTokensRangeResult>> {
        Ok(self.0.semantic_tokens_range(params))
    }

    async fn document_link(&self, params: DocumentLinkParams) -> Result<Option<Vec<DocumentLink>>> {
        Ok(self.0.document_link(params))
    }

    async fn document_link_resolve(&self, params: DocumentLink) -> Result<DocumentLink> {
        Ok(self.0.document_link_resolve(params))
    }

    async fn folding_range(&self, params: FoldingRangeParams) -> Result<Option<Vec<FoldingRange>>> {
        Ok(self.0.folding_range(params))
    }

    async fn code_lens(&self, params: CodeLensParams) -> Result<Option<Vec<CodeLens>>> {
        Ok(self.0.code_lens(params))
    }

    async fn code_lens_resolve(&self, params: CodeLens) -> Result<CodeLens> {
        Ok(self.0.code_lens_resolve(params))
    }

    async fn code_action(&self, params: CodeActionParams) -> Result<Option<CodeActionResponse>> {
        Ok(self.0.code_action(params))
    }

    async fn formatting(&self, params: DocumentFormattingParams) -> Result<Option<Vec<TextEdit>>> {
        Ok(self.0.formatting(params))
    }

    async fn execute_command(&self, params: ExecuteCommandParams) -> Result<Option<Value>> {
        Ok(self.0.execute_command(params).await)
    }

    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> Result<Option<DocumentSymbolResponse>> {
        Ok(self.0.document_symbol(params))
    }

    async fn references(&self, params: ReferenceParams) -> Result<Option<Vec<Location>>> {
        Ok(self.0.references(params))
    }
}

pub async fn start() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) =
        LspService::build(|client| OrgServer(LanguageServerBase::new(TowerEnvironment { client })))
            .finish();

    Server::new(stdin, stdout, socket).serve(service).await;
}
