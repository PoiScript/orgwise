use std::collections::HashMap;

use lsp_types::Url;
use orgize::rowan::TextRange;

use crate::backend::{Backend, Documents};

#[derive(Default)]
pub struct TestBackend {
    documents: Documents,
}

impl TestBackend {
    pub fn get(&self, url: &Url) -> String {
        self.documents.get_map(url, |d| d.org.to_org()).unwrap()
    }
}

impl Backend for TestBackend {
    fn documents(&self) -> &Documents {
        &self.documents
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
                .documents()
                .get_map(url, |d| d.org.to_org())
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

            self.documents.insert(url.clone(), &output)
        }

        Ok(())
    }

    async fn read_to_string(&self, url: &Url) -> anyhow::Result<String> {
        Ok(self
            .documents()
            .get_map(url, |d| d.org.to_org())
            .unwrap_or_default())
    }

    async fn write(&self, url: &Url, content: &str) -> anyhow::Result<()> {
        self.documents.insert(url.clone(), content);
        Ok(())
    }
}
