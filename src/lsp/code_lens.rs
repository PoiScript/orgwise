use lsp_types::*;
use orgize::{
    export::{Container, Event, TraversalContext, Traverser},
    rowan::ast::AstNode,
};

use crate::command::{
    ClockingStop, HeadlineGenerateToc, SrcBlockDetangle, SrcBlockExecute, SrcBlockTangle,
};
use crate::utils::src_block::{header_argument, property_drawer, property_keyword};
use crate::{backend::Backend, command::ClockingStart};
use crate::{backend::OrgDocument, utils::clocking::find_logbook};

pub fn code_lens<B: Backend>(backend: &B, params: CodeLensParams) -> Option<Vec<CodeLens>> {
    backend
        .documents()
        .get_map(&params.text_document.uri.clone(), |doc| {
            let mut traverser = CodeLensTraverser {
                url: params.text_document.uri,
                lens: vec![],
                doc,
            };

            doc.traverse(&mut traverser);

            traverser.lens
        })
}

pub fn code_lens_resolve<B: Backend>(_: &B, params: CodeLens) -> CodeLens {
    params
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
                let start = block.start();

                let arg1 = block.parameters().unwrap_or_default();
                let arg2 = property_drawer(block.syntax()).unwrap_or_default();
                let arg3 = property_keyword(block.syntax()).unwrap_or_default();

                let range = self.doc.range_of2(start, start);

                let tangle = header_argument(&arg1, &arg2, &arg3, ":tangle", "no");

                if header_argument(&arg1, &arg2, &arg3, ":results", "no") != "no" {
                    self.lens.push(CodeLens {
                        range,
                        command: Some(
                            SrcBlockExecute {
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
                            SrcBlockTangle {
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
                            SrcBlockDetangle {
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
                let start = headline.start();

                if headline.tags().any(|t| t.eq_ignore_ascii_case("TOC")) {
                    self.lens.push(CodeLens {
                        range: self.doc.range_of2(start, start),
                        command: Some(
                            HeadlineGenerateToc {
                                headline_offset: start,
                                url: self.url.clone(),
                            }
                            .into(),
                        ),
                        data: None,
                    });
                }

                if find_logbook(&headline).is_some() {
                    self.lens.push(CodeLens {
                        range: self.doc.range_of2(start, start),
                        command: Some(if headline.clocks().any(|c| c.is_running()) {
                            ClockingStop {
                                url: self.url.clone(),
                                line: self.doc.line_of(start.into()) + 1,
                            }
                            .into()
                        } else {
                            ClockingStart {
                                url: self.url.clone(),
                                line: self.doc.line_of(start.into()) + 1,
                            }
                            .into()
                        }),
                        data: None,
                    });
                }
            }
            _ => {}
        }
    }
}
