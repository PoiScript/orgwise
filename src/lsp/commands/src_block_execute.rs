use lsp_types::*;
use orgize::{ast::AffiliatedKeyword, rowan::TextRange, SyntaxKind};
use orgize::{ast::SourceBlock, rowan::ast::AstNode};
use std::collections::HashMap;
use std::iter::once;

use crate::common::{
    header_argument::{header_argument, property_drawer, property_keyword},
    utils::language_execute_command,
};

use super::LanguageServerBase;
use super::{FileSystem, LanguageClient, Process};

impl<E> LanguageServerBase<E>
where
    E: FileSystem + LanguageClient + Process,
{
    pub async fn src_block_execute(&self, url: Url, block_offset: u32) -> anyhow::Result<()> {
        let Some(doc) = self.documents.get(&url) else {
            return Ok(());
        };

        let Some(block) = doc
            .org
            .document()
            .syntax()
            .descendants()
            .filter_map(SourceBlock::cast)
            .find(|n| n.begin() == block_offset)
        else {
            return Ok(());
        };

        let Some(options) = ExecuteOptions::new(block) else {
            self.env
                .show_message(MessageType::WARNING, "Code block can't be executed.".into())
                .await;
            return Ok(());
        };

        let new_text = options.run(&self.env).await?;

        let mut changes = HashMap::new();

        let range = doc.range_of(options.range);

        changes.insert(url, vec![TextEdit { new_text, range }]);

        let _ = self
            .env
            .apply_edit(WorkspaceEdit {
                changes: Some(changes),
                ..Default::default()
            })
            .await;

        Ok(())
    }
}

pub struct ExecuteOptions {
    pub format: Format,
    pub executable: String,
    pub content: String,
    pub range: TextRange,
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
            let end = block.syntax().text_range().end();
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

    pub async fn run<E: Process>(&self, env: &E) -> anyhow::Result<String> {
        let output = env.execute(&self.executable, &self.content).await?;

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
        TextRange::new(0.into(), 0.into()),
    );
}
