use std::collections::HashMap;

use dashmap::DashMap;
use lsp_types::Url;
use orgize::{rowan::TextRange, ParseConfig};

use crate::base::{OrgDocument, Server};

#[derive(Default)]
pub struct TestServer {
    documents: DashMap<Url, OrgDocument>,
}

impl TestServer {
    pub fn get(&self, url: &Url) -> String {
        self.documents.get(url).unwrap().org.to_org()
    }
}

impl Server for TestServer {
    fn documents(&self) -> &DashMap<Url, OrgDocument> {
        &self.documents
    }

    fn default_parse_config(&self) -> ParseConfig {
        ParseConfig::default()
    }

    fn set_default_parse_config(&self, _: ParseConfig) {}

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
            edits.sort_by(|a, b| a.0.start().cmp(&b.0.start()));

            let input = self
                .documents()
                .get(url)
                .map(|d| d.org.to_org())
                .unwrap_or_default();
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

            self.add_doc(url.clone(), output)
        }

        Ok(())
    }

    async fn read_to_string(&self, url: &Url) -> anyhow::Result<String> {
        Ok(self
            .documents()
            .get(url)
            .map(|d| d.org.to_org())
            .unwrap_or_default())
    }

    async fn write(&self, url: &Url, content: &str) -> anyhow::Result<()> {
        self.add_doc(url.clone(), content.into());
        Ok(())
    }
}
