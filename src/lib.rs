#![allow(async_fn_in_trait)]

mod common;
pub mod lsp;

use lsp_types::{notification::*, request::*, MessageType, Url};
use serde::Serialize;
use wasm_bindgen::prelude::*;

use lsp::{FileSystem, LanguageClient, LanguageServerBase, Process};

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

struct WasmEnvironment {
    client: Client,
}

impl FileSystem for WasmEnvironment {
    type Location = Url;

    async fn write(&self, path: &Url, content: &str) -> anyhow::Result<()> {
        self.client
            .write(&path.to_string(), content)
            .await
            .map_err(|err| anyhow::anyhow!("JS Error: {err:?}"))?;
        Ok(())
    }

    async fn read_to_string(&self, path: &Url) -> anyhow::Result<String> {
        let value = self
            .client
            .read_to_string(&path.to_string())
            .await
            .map_err(|err| anyhow::anyhow!("JS Error: {err:?}"))?;

        Ok(value.as_string().unwrap_or_default())
    }

    fn resolve_in(&self, path: &str, base: &Url) -> anyhow::Result<Url> {
        if path.starts_with("~/") {
            if let Some(home) = self.client.home_dir().as_string() {
                return Ok(Url::parse(&format!("file://{home}{}", &path[1..]))?);
            }
        }

        let options = Url::options().base_url(Some(base));
        Ok(options.parse(path)?)
    }

    fn display(&self, path: &Url) -> impl std::fmt::Display {
        path.to_string()
    }
}

impl LanguageClient for WasmEnvironment {
    async fn send_request<R: Request>(&self, params: R::Params) -> anyhow::Result<R::Result> {
        let serializer = serde_wasm_bindgen::Serializer::new().serialize_maps_as_objects(true);
        let value = params.serialize(&serializer).unwrap();
        let result = self
            .client
            .send_request(R::METHOD, value)
            .await
            .map_err(|err| anyhow::anyhow!("{:?}", err))?;
        Ok(serde_wasm_bindgen::from_value(result).unwrap())
    }

    async fn send_notification<N: Notification>(&self, params: N::Params) {
        let serializer = serde_wasm_bindgen::Serializer::new().serialize_maps_as_objects(true);
        let value = params.serialize(&serializer).unwrap();
        self.client.send_notification(N::METHOD, value).await;
    }
}

impl Process for WasmEnvironment {
    async fn execute(&self, _: &str, _: &str) -> anyhow::Result<String> {
        todo!()
    }
}

#[wasm_bindgen]
pub struct Server(LanguageServerBase<WasmEnvironment>);

#[wasm_bindgen]
impl Server {
    #[wasm_bindgen(constructor)]
    pub fn new(client: Client) -> Server {
        console_error_panic_hook::set_once();

        Server(LanguageServerBase::new(WasmEnvironment { client }))
    }

    #[allow(unused_variables)]
    #[wasm_bindgen(js_class = Server, js_name = onRequest)]
    pub async fn on_request(&mut self, method: &str, params: JsValue) -> JsValue {
        const SERIALIZER: serde_wasm_bindgen::Serializer =
            serde_wasm_bindgen::Serializer::new().serialize_maps_as_objects(true);

        fn r<R: Request>(
            server: &Server,
            params: JsValue,
            f: impl FnOnce(&LanguageServerBase<WasmEnvironment>, R::Params) -> R::Result,
        ) -> JsValue {
            let params = serde_wasm_bindgen::from_value(params).unwrap();
            let result = f(&server.0, params);
            result.serialize(&SERIALIZER).unwrap()
        }

        match method {
            Initialize::METHOD => {
                let params = serde_wasm_bindgen::from_value(params).unwrap();
                let result = self.0.initialize(params).await;
                self.0
                    .env
                    .log_message(MessageType::ERROR, format!("{:?}", result))
                    .await;
                result.serialize(&SERIALIZER).unwrap()
            }
            Completion::METHOD => r::<Completion>(self, params, LanguageServerBase::completion),
            SemanticTokensFullRequest::METHOD => r::<SemanticTokensFullRequest>(
                self,
                params,
                LanguageServerBase::semantic_tokens_full,
            ),
            SemanticTokensRangeRequest::METHOD => r::<SemanticTokensRangeRequest>(
                self,
                params,
                LanguageServerBase::semantic_tokens_range,
            ),
            FoldingRangeRequest::METHOD => {
                r::<FoldingRangeRequest>(self, params, LanguageServerBase::folding_range)
            }
            CodeLensRequest::METHOD => {
                r::<CodeLensRequest>(self, params, LanguageServerBase::code_lens)
            }
            References::METHOD => r::<References>(self, params, LanguageServerBase::references),
            Formatting::METHOD => r::<Formatting>(self, params, LanguageServerBase::formatting),
            ExecuteCommand::METHOD => {
                let params = serde_wasm_bindgen::from_value(params).unwrap();
                let result = self.0.execute_command(params).await;
                result.serialize(&SERIALIZER).unwrap()
            }
            DocumentSymbolRequest::METHOD => {
                r::<DocumentSymbolRequest>(self, params, LanguageServerBase::document_symbol)
            }
            DocumentLinkRequest::METHOD => {
                r::<DocumentLinkRequest>(self, params, LanguageServerBase::document_link)
            }
            DocumentLinkResolve::METHOD => {
                r::<DocumentLinkResolve>(self, params, LanguageServerBase::document_link_resolve)
            }
            _ => JsValue::NULL,
        }
    }

    #[allow(unused_variables)]
    #[wasm_bindgen(js_class = Server, js_name = onNotification)]
    pub async fn on_notification(&mut self, method: &str, params: JsValue) {
        fn n<N: Notification>(
            server: &Server,
            params: JsValue,
            f: impl FnOnce(&LanguageServerBase<WasmEnvironment>, N::Params) -> (),
        ) {
            let params = serde_wasm_bindgen::from_value(params).unwrap();
            f(&server.0, params)
        }

        match method {
            Initialized::METHOD => {
                self.0.initialized().await;
            }
            DidOpenTextDocument::METHOD => {
                n::<DidOpenTextDocument>(self, params, LanguageServerBase::did_open)
            }
            DidChangeConfiguration::METHOD => n::<DidChangeConfiguration>(
                self,
                params,
                LanguageServerBase::did_change_configuration,
            ),
            DidChangeTextDocument::METHOD => {
                let params = serde_wasm_bindgen::from_value(params).unwrap();
                self.0.did_change(params).await;
            }
            _ => {}
        }
    }
}
