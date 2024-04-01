use lsp_types::*;
use orgize::rowan::TextSize;
use orgize::{ast::AffiliatedKeyword, rowan::TextRange, SyntaxKind};
use orgize::{ast::SourceBlock, rowan::ast::AstNode};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::iter::once;

use super::utils::{header_argument, language_execute_command, property_drawer, property_keyword};
use super::Executable;

use crate::base::Server;
use crate::command::utils::collect_src_blocks;

#[derive(Serialize, Deserialize)]
pub struct SrcBlockExecute {
    pub url: Url,
    #[serde(with = "crate::command::utils::text_size")]
    pub block_offset: TextSize,
}

impl Executable for SrcBlockExecute {
    const NAME: &'static str = "src-block-execute";

    const TITLE: Option<&'static str> = Some("Execute");

    async fn execute<S: Server>(self, server: &S) -> anyhow::Result<Value> {
        let Some(doc) = server.documents().get(&self.url) else {
            return Ok(Value::Null);
        };

        let Some(block) = doc.org.node_at_offset(self.block_offset) else {
            return Ok(Value::Null);
        };

        let Some(options) = ExecuteOptions::new(block) else {
            server
                .log_message(MessageType::ERROR, "Code block can't be executed.".into())
                .await;
            return Ok(Value::Null);
        };

        let new_text = options.run(server).await?;

        drop(doc);

        server.apply_edit(self.url, new_text, options.range).await?;

        Ok(Value::Bool(true))
    }
}

#[derive(Serialize, Deserialize)]
pub struct SrcBlockExecuteAll {
    pub url: Url,
}

impl Executable for SrcBlockExecuteAll {
    const NAME: &'static str = "src-block-execute-all";

    const TITLE: Option<&'static str> = Some("Execute all source blocks");

    async fn execute<S: Server>(self, server: &S) -> anyhow::Result<Value> {
        let Some(doc) = server.documents().get(&self.url) else {
            return Ok(Value::Null);
        };

        let blocks = collect_src_blocks(&doc.org);
        let options: Vec<_> = blocks.into_iter().filter_map(ExecuteOptions::new).collect();

        let mut edits = Vec::with_capacity(options.len());

        for option in options {
            let content = option.run(server).await?;
            edits.push((self.url.clone(), content, option.range));
        }

        drop(doc);

        server.apply_edits(edits.into_iter()).await?;

        Ok(Value::Bool(true))
    }
}

struct ExecuteOptions {
    format: Format,
    executable: String,
    content: String,
    range: TextRange,
}

impl ExecuteOptions {
    pub fn new(block: SourceBlock) -> Option<Self> {
        let arg1 = block.parameters().unwrap_or_default();
        let arg2 = property_drawer(block.syntax()).unwrap_or_default();
        let arg3 = property_keyword(block.syntax()).unwrap_or_default();
        let language = block.language().unwrap_or_default();
        let results = header_argument(&arg1, &arg2, &arg3, ":results", "no");

        if results == "no" {
            return None;
        }

        let mut segs = results.split(&[' ', '\t']).filter(|x| !x.is_empty());

        let format = match (segs.next(), segs.next()) {
            (Some("output"), Some("code")) | (Some("code"), None) => Format::Code,
            (Some("output"), Some("list")) | (Some("list"), None) => Format::List,
            (Some("output"), Some("scalar"))
            | (Some("scalar"), None)
            | (Some("output"), Some("verbatim"))
            | (Some("verbatim"), None) => Format::Verbatim,
            (Some("output"), Some("html")) | (Some("html"), None) => Format::Html,
            (Some("output"), Some("latex")) | (Some("latex"), None) => Format::Latex,
            (Some("output"), Some("raw")) | (Some("raw"), None) => Format::Raw,
            _ => return None,
        };

        let range = find_existing_results(&block).unwrap_or_else(|| {
            let end = block.end();
            TextRange::new(end, end)
        });

        let Some(executable) = language_execute_command(&language) else {
            return None;
        };

        Some(ExecuteOptions {
            executable: executable.to_string(),
            content: block.value(),
            format,
            range,
        })
    }

    pub async fn run<S: Server>(&self, server: &S) -> anyhow::Result<String> {
        let output = server.execute(&self.executable, &self.content).await?;

        let mut output = match self.format {
            Format::Code => once("#+begin_src")
                .chain(output.lines())
                .chain(once("#+end_src"))
                .fold(String::new(), |acc, line| acc + line + "\n"),
            Format::Html => once("#+begin_export html")
                .chain(output.lines())
                .chain(once("#+end_export"))
                .fold(String::new(), |acc, line| acc + line + "\n"),
            Format::Latex => once("#+begin_export latex")
                .chain(output.lines())
                .chain(once("#+end_export"))
                .fold(String::new(), |acc, line| acc + line + "\n"),
            Format::List => output
                .lines()
                .fold(String::new(), |acc, line| acc + "- " + line + "\n"),
            Format::Verbatim => output
                .lines()
                .fold(String::new(), |acc, line| acc + ": " + line + "\n"),
            Format::Raw => output,
        };

        if self.range.start() == self.range.end() {
            output = format!("\n#+RESULTS:\n{output}\n");
        }

        Ok(output)
    }
}

#[derive(Debug)]
pub enum Format {
    Code,
    List,
    Verbatim,
    Html,
    Latex,
    Raw,
}

fn find_existing_results(block: &SourceBlock) -> Option<TextRange> {
    let sibling = block.syntax().next_sibling().filter(|n| {
        n.children()
            .filter_map(AffiliatedKeyword::cast)
            .any(|k| k.key().eq_ignore_ascii_case("results"))
    })?;

    let (first, last) = match sibling.kind() {
        SyntaxKind::SOURCE_BLOCK | SyntaxKind::EXPORT_BLOCK => {
            let begin = sibling
                .children_with_tokens()
                .find(|n| n.kind() == SyntaxKind::BLOCK_BEGIN);

            let end = sibling
                .children_with_tokens()
                .find(|n| n.kind() == SyntaxKind::BLOCK_END);

            (begin, end)
        }
        SyntaxKind::LIST => {
            let mut iter = sibling
                .children_with_tokens()
                .filter(|n| n.kind() == SyntaxKind::LIST_ITEM);

            (iter.next(), iter.last())
        }
        SyntaxKind::ORG_TABLE => {
            let mut iter = sibling.children_with_tokens().filter(|n| {
                n.kind() == SyntaxKind::ORG_TABLE_RULE_ROW
                    || n.kind() == SyntaxKind::ORG_TABLE_STANDARD_ROW
            });

            (iter.next(), iter.last())
        }
        SyntaxKind::FIXED_WIDTH => {
            let mut iter = sibling
                .children_with_tokens()
                .skip_while(|n| n.kind() == SyntaxKind::AFFILIATED_KEYWORD)
                .take_while(|n| n.kind() != SyntaxKind::BLANK_LINE);

            (iter.next(), iter.last())
        }
        _ => return None,
    };

    let start = first.as_ref().map(|n| n.text_range().start())?;

    let end = last.or(first).map(|x| x.text_range().end())?;

    Some(TextRange::new(start, end))
}

#[test]
fn test() {
    use orgize::Org;

    let org = r#"
#+BEGIN_SRC bash :results output code
#+END_SRC

#+RESULTS:
#+begin_src


#+end_src


"#;
    let org = Org::parse(org);
    let block = org.first_node::<SourceBlock>().unwrap();

    assert_eq!(
        find_existing_results(&block).unwrap(),
        TextRange::new(61.into(), 85.into(),),
    );
}
