use std::{collections::HashMap, path::PathBuf};

use crate::lsp::commands::{
    src_block_detangle::DetangleOptions, src_block_execute::ExecuteOptions,
    src_block_tangle::TangleOptions,
};
use clap::{
    builder::styling::{AnsiColor, Color, Style},
    Args,
};
use orgize::{
    ast::SourceBlock,
    export::{Container, Event, TraversalContext, Traverser},
    Org,
};

use crate::cli::{diff, environment::TokioEnvironment};

fn collect_src_blocks(org: &Org) -> Vec<SourceBlock> {
    struct CollectSrcBlock {
        blocks: Vec<SourceBlock>,
    }

    impl Traverser for CollectSrcBlock {
        fn event(&mut self, event: Event, ctx: &mut TraversalContext) {
            match event {
                Event::Enter(Container::SourceBlock(block)) => {
                    self.blocks.push(block);
                    ctx.skip();
                }

                // skip some containers for performance
                Event::Enter(Container::List(_))
                | Event::Enter(Container::OrgTable(_))
                | Event::Enter(Container::SpecialBlock(_))
                | Event::Enter(Container::QuoteBlock(_))
                | Event::Enter(Container::CenterBlock(_))
                | Event::Enter(Container::VerseBlock(_))
                | Event::Enter(Container::CommentBlock(_))
                | Event::Enter(Container::ExampleBlock(_))
                | Event::Enter(Container::ExportBlock(_)) => {
                    ctx.skip();
                }

                _ => {}
            }
        }
    }

    let mut t = CollectSrcBlock { blocks: vec![] };
    org.traverse(&mut t);
    t.blocks
}

#[derive(Debug, Args)]
pub struct DetangleCommand {
    path: Vec<PathBuf>,

    #[arg(short, long)]
    dry_run: bool,
}

impl DetangleCommand {
    pub async fn run(self) -> anyhow::Result<()> {
        for path in self.path {
            if !path.exists() {
                tracing::error!("{:?} is not existed", path);
                continue;
            }

            let input = std::fs::read_to_string(&path)?;
            let org = Org::parse(&input);

            let mut results: Vec<(usize, usize, String)> = vec![];

            for block in collect_src_blocks(&org) {
                if let Some(option) = DetangleOptions::new(block, &path, &TokioEnvironment) {
                    let (range, content) = option.run(&TokioEnvironment).await?;
                    results.push((range.start().into(), range.end().into(), content));
                }
            }

            if self.dry_run {
                diff::print(&input, results);
            } else {
                diff::write_to_file(&input, results, path)?;
            }
        }

        Ok(())
    }
}

#[derive(Debug, Args)]
pub struct ExecuteCommand {
    path: Vec<PathBuf>,

    #[arg(short, long)]
    dry_run: bool,
}

impl ExecuteCommand {
    pub async fn run(self) -> anyhow::Result<()> {
        let dir = tempfile::tempdir()?;

        tracing::debug!("Create tempdir {:?}", dir.path().to_string_lossy());

        for path in self.path {
            if !path.exists() {
                tracing::error!("{:?} is not existed", path);
                continue;
            }

            let input = std::fs::read_to_string(&path)?;
            let org = Org::parse(&input);

            let mut results: Vec<(usize, usize, String)> = vec![];

            for block in collect_src_blocks(&org) {
                if let Some(option) = ExecuteOptions::new(block) {
                    let content = option.run(&TokioEnvironment).await?;
                    results.push((
                        option.range.start().into(),
                        option.range.end().into(),
                        content,
                    ));
                }
            }

            if self.dry_run {
                diff::print(&input, results);
            } else {
                diff::write_to_file(&input, results, path)?;
            }
        }

        Ok(())
    }
}

#[derive(Debug, Args)]
pub struct TangleCommand {
    path: Vec<PathBuf>,

    #[arg(short, long)]
    dry_run: bool,
}

impl TangleCommand {
    pub async fn run(self) -> anyhow::Result<()> {
        let results = HashMap::<PathBuf, (Option<u32>, String, bool)>::new();

        for path in self.path {
            if !path.exists() {
                tracing::error!("{:?} is not existed", path);
            }

            let string = std::fs::read_to_string(&path)?;
            let org = Org::parse(string);

            let mut i = 0;

            for block in collect_src_blocks(&org) {
                let Some(options) = TangleOptions::new(block, &path, &TokioEnvironment) else {
                    continue;
                };

                let (range, new_text) = options.run(&TokioEnvironment).await?;

                i += 1;

                // results
                //     .entry(&options.)
                //     .and_modify(|e| {
                //         e.1 += &options.content;
                //     })
                //     .or_insert((options.permission, options.content, options.mkdir));
            }

            if i > 0 {
                tracing::info!("Found {} code block from {}", i, path.display());
            }
        }

        if self.dry_run {
            for (path, (permission, content, mkdir)) in results {
                let style = Style::new()
                    .fg_color(Color::Ansi(AnsiColor::BrightYellow).into())
                    .underline()
                    .bold();
                print!(
                    "{}{}{}",
                    style.render(),
                    path.display(),
                    style.render_reset(),
                );
                if let Some(permission) = permission {
                    print!(" (permission: {:o})", permission);
                }
                if mkdir {
                    print!(" (mkdir: yes)");
                }
                println!("\n{}", content);
            }
        } else {
            for (path, (_, contents, _)) in results {
                tokio::fs::write(&path, contents).await?;
                tracing::info!("Wrote to {}", path.display());
            }
        }

        Ok(())
    }
}
