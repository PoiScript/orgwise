use lsp_types::*;
use orgize::{
    export::{Container, Event, TraversalContext, Traverser},
    rowan::ast::AstNode,
};
use crate::common::{header_argument, property_drawer, property_keyword};

use super::{
    commands::OrgwiseCommand, org_document::OrgDocument, FileSystem, LanguageClient,
    LanguageServerBase, Process,
};

impl<E> LanguageServerBase<E>
where
    E: FileSystem + LanguageClient + Process,
{
    pub fn code_lens(&self, params: CodeLensParams) -> Option<Vec<CodeLens>> {
        let doc = self.documents.get(&params.text_document.uri)?;

        let mut traverser = CodeLensTraverser {
            url: params.text_document.uri,
            lens: vec![],
            doc: &doc,
        };

        doc.traverse(&mut traverser);

        Some(traverser.lens)
    }

    pub fn code_lens_resolve(&self, params: CodeLens) -> CodeLens {
        params
    }
}

struct CodeLensTraverser<'a> {
    url: Url,
    doc: &'a OrgDocument,
    lens: Vec<CodeLens>,
}

impl<'a> Traverser for CodeLensTraverser<'a> {
    fn event(&mut self, event: Event, ctx: &mut TraversalContext) {
        match event {
            Event::Enter(Container::SourceBlock(block)) => {
                let start = block.begin();

                let arg1 = block.parameters().unwrap_or_default();
                let arg2 = property_drawer(block.syntax()).unwrap_or_default();
                let arg3 = property_keyword(block.syntax()).unwrap_or_default();

                let range = self.doc.range_of2(start, start);

                let tangle = header_argument(&arg1, &arg2, &arg3, ":tangle", "no");

                if header_argument(&arg1, &arg2, &arg3, ":results", "no") != "no" {
                    self.lens.push(CodeLens {
                        range,
                        command: Some(
                            OrgwiseCommand::SrcBlockExecute {
                                block_offset: start,
                                url: self.url.clone(),
                            }
                            .into(),
                        ),
                        data: None,
                    });
                }

                if tangle != "no" {
                    self.lens.push(CodeLens {
                        range,
                        command: Some(
                            OrgwiseCommand::SrcBlockTangle {
                                block_offset: start,
                                url: self.url.clone(),
                            }
                            .into(),
                        ),
                        data: None,
                    });

                    self.lens.push(CodeLens {
                        range,
                        command: Some(
                            OrgwiseCommand::SrcBlockDetangle {
                                block_offset: start,
                                url: self.url.clone(),
                            }
                            .into(),
                        ),
                        data: None,
                    });
                }

                ctx.skip();
            }
            Event::Enter(Container::Headline(headline)) => {
                if headline.tags().any(|t| t.eq_ignore_ascii_case("TOC")) {
                    let start = headline.begin();

                    self.lens.push(CodeLens {
                        range: self.doc.range_of2(start, start),
                        command: Some(
                            OrgwiseCommand::HeadlineToc {
                                heading_offset: start,
                                url: self.url.clone(),
                            }
                            .into(),
                        ),
                        data: None,
                    });
                }
            }
            _ => {}
        }
    }
}
