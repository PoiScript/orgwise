#![allow(async_fn_in_trait)]
#![allow(dead_code)]

mod base;
#[cfg(feature = "tower")]
mod cli;
mod command;
mod lsp;
mod utils;

#[cfg(test)]
mod test;

use std::collections::HashMap;

use crate::base::Server;
use base::OrgDocument;
use dashmap::{DashMap, RwLock};
use lsp_types::{
    notification::*, request::*, ApplyWorkspaceEditParams, LogMessageParams, MessageType,
    ShowMessageParams, TextEdit, Url, WorkspaceEdit,
};
use orgize::{rowan::TextRange, ParseConfig};
use serde::Serialize;
use wasm_bindgen::prelude::*;

const SERIALIZER: serde_wasm_bindgen::Serializer =
    serde_wasm_bindgen::Serializer::new().serialize_maps_as_objects(true);

#[wasm_bindgen]
extern "C" {
    pub type Client;

    #[wasm_bindgen(method, js_name = "homeDir")]
    pub fn home_dir(this: &Client) -> JsValue;

    #[wasm_bindgen(method, js_name = "sendRequest", catch)]
    pub async fn send_request(
        this: &Client,
        method: &str,
        params: JsValue,
    ) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(method, js_name = "readToString", catch)]
    pub async fn read_to_string(this: &Client, path: &str) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(method, js_name = "write", catch)]
    pub async fn write(this: &Client, path: &str, content: &str) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(method, js_name = "sendNotification")]
    pub async fn send_notification(this: &Client, method: &str, params: JsValue);
}

#[wasm_bindgen]
pub struct WasmLspServer {
    client: Client,
    documents: DashMap<Url, OrgDocument>,
    parse_config: RwLock<ParseConfig>,
}

impl WasmLspServer {
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

impl Server for WasmLspServer {
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
            if let Some(doc) = self.documents.get(&url) {
                let edit = TextEdit {
                    new_text,
                    range: doc.range_of(text_range),
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

        self.send_request::<ApplyWorkspaceEdit>(edit).await?;

        Ok(())
    }

    fn documents(&self) -> &DashMap<Url, OrgDocument> {
        &self.documents
    }

    fn default_parse_config(&self) -> ParseConfig {
        self.parse_config.read().clone()
    }

    fn set_default_parse_config(&self, config: ParseConfig) {
        *self.parse_config.write() = config;
    }
}

#[wasm_bindgen]
impl WasmLspServer {
    #[wasm_bindgen(constructor)]
    pub fn new(client: Client) -> WasmLspServer {
        console_error_panic_hook::set_once();

        WasmLspServer {
            client,
            documents: DashMap::new(),
            parse_config: RwLock::new(ParseConfig::default()),
        }
    }

    #[allow(unused_variables)]
    #[wasm_bindgen(js_class = Server, js_name = onRequest)]
    pub async fn on_request(&mut self, method: &str, params: JsValue) -> JsValue {
        fn r<R: Request>(
            server: &WasmLspServer,
            params: JsValue,
            f: impl FnOnce(&WasmLspServer, R::Params) -> R::Result,
        ) -> JsValue {
            let params = serde_wasm_bindgen::from_value(params).unwrap();
            let result = f(server, params);
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
    #[wasm_bindgen(js_class = Server, js_name = onNotification)]
    pub async fn on_notification(&mut self, method: &str, params: JsValue) {
        fn n<N: Notification>(
            server: &WasmLspServer,
            params: JsValue,
            f: impl FnOnce(&WasmLspServer, N::Params),
        ) {
            let params = serde_wasm_bindgen::from_value(params).unwrap();
            f(server, params)
        }

        match method {
            Initialized::METHOD => {
                lsp::initialized(self).await;
            }
            DidOpenTextDocument::METHOD => n::<DidOpenTextDocument>(self, params, lsp::did_open),
            DidChangeTextDocument::METHOD => {
                let params = serde_wasm_bindgen::from_value(params).unwrap();
                lsp::did_change(self, params).await;
            }
            _ => {}
        }
    }
}
