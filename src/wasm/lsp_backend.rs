use lsp_types::{
    notification::*, request::*, ApplyWorkspaceEditParams, LogMessageParams, MessageType,
    ShowMessageParams, TextEdit, Url, WorkspaceEdit,
};
use orgize::rowan::TextRange;
use serde::Serialize;
use std::collections::HashMap;
use wasm_bindgen::prelude::*;

use super::SERIALIZER;
use crate::backend::{Backend, Documents};
use crate::lsp;

#[wasm_bindgen]
extern "C" {
    pub type LspClient;

    #[wasm_bindgen(method, js_name = "homeDir")]
    pub fn home_dir(this: &LspClient) -> JsValue;

    #[wasm_bindgen(method, js_name = "sendRequest", catch)]
    pub async fn send_request(
        this: &LspClient,
        method: &str,
        params: JsValue,
    ) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(method, js_name = "readToString", catch)]
    pub async fn read_to_string(this: &LspClient, path: &str) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(method, js_name = "write", catch)]
    pub async fn write(this: &LspClient, path: &str, content: &str) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(method, js_name = "sendNotification")]
    pub async fn send_notification(this: &LspClient, method: &str, params: JsValue);
}

#[wasm_bindgen(js_name = "LspBackend")]
pub struct LspBackend {
    client: LspClient,
    documents: Documents,
}

impl LspBackend {
    async fn send_request<R: Request>(&self, params: R::Params) -> anyhow::Result<R::Result> {
        let value = params.serialize(&SERIALIZER).unwrap();
        let result = self
            .client
            .send_request(R::METHOD, value)
            .await
            .map_err(|err| anyhow::anyhow!("{:?}", err))?;
        Ok(serde_wasm_bindgen::from_value(result).unwrap())
    }

    async fn send_notification<N: Notification>(&self, params: N::Params) {
        let value = params.serialize(&SERIALIZER).unwrap();
        self.client.send_notification(N::METHOD, value).await;
    }
}

impl Backend for LspBackend {
    fn home_dir(&self) -> Option<Url> {
        self.client
            .home_dir()
            .as_string()
            .and_then(|s| Url::parse(&s).ok())
    }

    async fn write(&self, path: &Url, content: &str) -> anyhow::Result<()> {
        self.client
            .write(path.as_ref(), content)
            .await
            .map_err(|err| anyhow::anyhow!("JS Error: {err:?}"))?;
        Ok(())
    }

    async fn read_to_string(&self, path: &Url) -> anyhow::Result<String> {
        let value = self
            .client
            .read_to_string(path.as_ref())
            .await
            .map_err(|err| anyhow::anyhow!("JS Error: {err:?}"))?;

        Ok(value.as_string().unwrap_or_default())
    }

    async fn log_message(&self, typ: MessageType, message: String) {
        self.send_notification::<LogMessage>(LogMessageParams { typ, message })
            .await;
    }

    async fn show_message(&self, typ: MessageType, message: String) {
        self.send_notification::<ShowMessage>(ShowMessageParams { typ, message })
            .await;
    }

    async fn apply_edits(
        &self,
        items: impl Iterator<Item = (Url, String, TextRange)>,
    ) -> anyhow::Result<()> {
        let mut changes: HashMap<Url, Vec<TextEdit>> = HashMap::new();

        for (url, new_text, text_range) in items {
            self.documents.with(&url, |doc| {
                let edit = TextEdit {
                    new_text,
                    range: doc.range_of(text_range),
                };
                changes
                    .entry(url.clone())
                    .and_modify(|edits| edits.push(edit.clone()))
                    .or_insert_with(|| vec![edit]);
            });
        }

        let edit = ApplyWorkspaceEditParams {
            label: None,
            edit: WorkspaceEdit {
                changes: Some(changes),
                ..Default::default()
            },
        };

        self.send_request::<ApplyWorkspaceEdit>(edit).await?;

        Ok(())
    }

    fn documents(&self) -> &Documents {
        &self.documents
    }

    // fn default_parse_config(&self) -> ParseConfig {
    //     self.parse_config.clone()
    // }

    // fn set_default_parse_config(&self, config: ParseConfig) {}
}

#[wasm_bindgen(js_class = "LspBackend")]
impl LspBackend {
    #[wasm_bindgen(constructor)]
    pub fn new(client: LspClient) -> LspBackend {
        console_error_panic_hook::set_once();

        LspBackend {
            client,
            documents: Documents::default(),
        }
    }

    #[allow(unused_variables)]
    #[wasm_bindgen(js_name = "onRequest")]
    pub async fn on_request(&mut self, method: &str, params: JsValue) -> JsValue {
        fn r<R: Request>(
            backend: &LspBackend,
            params: JsValue,
            f: impl FnOnce(&LspBackend, R::Params) -> R::Result,
        ) -> JsValue {
            let params = serde_wasm_bindgen::from_value(params).unwrap();
            let result = f(backend, params);
            result.serialize(&SERIALIZER).unwrap()
        }

        match method {
            Initialize::METHOD => {
                let params = serde_wasm_bindgen::from_value(params).unwrap();
                let result = lsp::initialize(self, params).await;
                self.log_message(MessageType::ERROR, format!("{:?}", result))
                    .await;
                result.serialize(&SERIALIZER).unwrap()
            }
            Completion::METHOD => r::<Completion>(self, params, lsp::completion),
            SemanticTokensFullRequest::METHOD => {
                r::<SemanticTokensFullRequest>(self, params, lsp::semantic_tokens_full)
            }
            SemanticTokensRangeRequest::METHOD => {
                r::<SemanticTokensRangeRequest>(self, params, lsp::semantic_tokens_range)
            }
            FoldingRangeRequest::METHOD => {
                r::<FoldingRangeRequest>(self, params, lsp::folding_range)
            }
            CodeLensRequest::METHOD => r::<CodeLensRequest>(self, params, lsp::code_lens),
            References::METHOD => r::<References>(self, params, lsp::references),
            Formatting::METHOD => r::<Formatting>(self, params, lsp::formatting),
            ExecuteCommand::METHOD => {
                let params = serde_wasm_bindgen::from_value(params).unwrap();
                let result = lsp::execute_command(self, params).await;
                result.serialize(&SERIALIZER).unwrap()
            }
            DocumentSymbolRequest::METHOD => {
                r::<DocumentSymbolRequest>(self, params, lsp::document_symbol)
            }
            DocumentLinkRequest::METHOD => {
                r::<DocumentLinkRequest>(self, params, lsp::document_link)
            }
            DocumentLinkResolve::METHOD => {
                r::<DocumentLinkResolve>(self, params, lsp::document_link_resolve)
            }
            _ => JsValue::NULL,
        }
    }

    #[allow(unused_variables)]
    #[wasm_bindgen(js_name = "onNotification")]
    pub async fn on_notification(&mut self, method: &str, params: JsValue) {
        fn n<N: Notification>(
            backend: &LspBackend,
            params: JsValue,
            f: impl FnOnce(&LspBackend, N::Params),
        ) {
            let params = serde_wasm_bindgen::from_value(params).unwrap();
            f(backend, params)
        }

        match method {
            Initialized::METHOD => {
                lsp::initialized(self).await;
            }
            DidOpenTextDocument::METHOD => n::<DidOpenTextDocument>(self, params, lsp::did_open),
            DidChangeTextDocument::METHOD => {
                n::<DidChangeTextDocument>(self, params, lsp::did_change)
            }
            _ => {}
        }
    }
}
