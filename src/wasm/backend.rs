use lsp_types::{MessageType, Url};
use orgize::rowan::TextRange;
use orgize::ParseConfig;
use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;
use wasm_bindgen::prelude::*;

use super::SERIALIZER;
use crate::backend::{Backend, Documents};
use crate::command::OrgwiseCommand;
use crate::lsp;

#[wasm_bindgen]
extern "C" {
    pub type WasmMethods;

    #[wasm_bindgen(method, js_name = "homeDir")]
    pub fn home_dir(this: &WasmMethods) -> JsValue;

    #[wasm_bindgen(method, js_name = "readToString", catch)]
    pub async fn read_to_string(this: &WasmMethods, path: &str) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(method, js_name = "write", catch)]
    pub async fn write(this: &WasmMethods, path: &str, content: &str) -> Result<JsValue, JsValue>;
}

#[wasm_bindgen(js_name = "Backend")]
pub struct WasmBackend {
    methods: WasmMethods,
    documents: Documents,
}

impl Backend for WasmBackend {
    fn home_dir(&self) -> Option<Url> {
        self.methods
            .home_dir()
            .as_string()
            .and_then(|s| Url::parse(&s).ok())
    }

    async fn write(&self, path: &Url, content: &str) -> anyhow::Result<()> {
        self.methods
            .write(path.as_ref(), content)
            .await
            .map_err(|err| anyhow::anyhow!("JS Error: {err:?}"))?;
        Ok(())
    }

    async fn read_to_string(&self, path: &Url) -> anyhow::Result<String> {
        let value = self
            .methods
            .read_to_string(path.as_ref())
            .await
            .map_err(|err| anyhow::anyhow!("JS Error: {err:?}"))?;

        Ok(value.as_string().unwrap_or_default())
    }

    async fn log_message(&self, typ: MessageType, message: String) {
        match typ {
            MessageType::ERROR => web_sys::console::error_1(&JsValue::from_str(&message)),
            MessageType::WARNING => web_sys::console::warn_1(&JsValue::from_str(&message)),
            MessageType::INFO => web_sys::console::info_1(&JsValue::from_str(&message)),
            MessageType::LOG | _ => web_sys::console::log_1(&JsValue::from_str(&message)),
        };
    }

    async fn show_message(&self, typ: MessageType, message: String) {
        self.log_message(typ, message).await
    }

    async fn apply_edits(
        &self,
        items: impl Iterator<Item = (Url, String, TextRange)>,
    ) -> anyhow::Result<()> {
        let mut changes: HashMap<Url, Vec<(TextRange, String)>> = HashMap::new();

        for (url, new_text, text_range) in items {
            if let Some(edits) = changes.get_mut(&url) {
                edits.push((text_range, new_text))
            } else {
                changes.insert(url.clone(), vec![(text_range, new_text)]);
            }
        }

        for (url, edits) in changes.iter_mut() {
            edits.sort_by_key(|edit| (edit.0.start(), edit.0.end()));

            let input = self
                .methods
                .read_to_string(&url.to_string())
                .await
                .unwrap()
                .as_string()
                .unwrap();

            let mut output = String::with_capacity(input.len());
            let mut off = 0;

            for (range, content) in edits {
                let start = range.start().into();
                let end = range.end().into();

                output += &input[off..start];
                output += &content;

                off = end;
            }

            output += &input[off..];

            self.write(&url, &output).await?;
            self.documents.update(url.clone(), None, &output);
        }

        Ok(())
    }

    fn documents(&self) -> &Documents {
        &self.documents
    }
}

#[wasm_bindgen(js_class = "Backend")]
impl WasmBackend {
    #[wasm_bindgen(constructor)]
    pub fn new(methods: WasmMethods) -> WasmBackend {
        console_error_panic_hook::set_once();

        WasmBackend {
            methods,
            documents: Documents::default(),
        }
    }

    #[wasm_bindgen(js_name = "setOptions")]
    pub fn set_options(&mut self, options: JsValue) {
        let options: lsp::InitializationOptions = serde_wasm_bindgen::from_value(options).unwrap();
        self.documents().set_default_parse_config(ParseConfig {
            todo_keywords: (options.todo_keywords, options.done_keywords),
            ..Default::default()
        });
    }

    #[wasm_bindgen(js_name = "addOrgFile")]
    pub fn add_org_file(&mut self, url: String, text: &str) {
        self.documents.insert(Url::parse(&url).unwrap(), text);
    }

    #[wasm_bindgen(js_name = "updateOrgFile")]
    pub fn update_org_file(&mut self, url: String, text: &str) {
        self.documents.update(Url::parse(&url).unwrap(), None, text);
    }

    #[wasm_bindgen(js_name = "executeCommand")]
    pub async fn execute_command(&mut self, name: &str, argument: JsValue) -> JsValue {
        let argument: Value = serde_wasm_bindgen::from_value(argument).unwrap();

        let Some(cmd) = OrgwiseCommand::from_value(name, argument) else {
            self.log_message(MessageType::WARNING, format!("Unknown command {name:?}"))
                .await;

            return JsValue::NULL;
        };

        match cmd.execute(self).await {
            Ok(value) => value.serialize(&SERIALIZER).unwrap(),
            Err(err) => {
                self.log_message(
                    MessageType::ERROR,
                    format!("Failed to execute {name:?}: {err}"),
                )
                .await;

                JsValue::NULL
            }
        }
    }
}
