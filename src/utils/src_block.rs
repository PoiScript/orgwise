use jetscii::Substring;
use nom::{
    bytes::complete::take_while1,
    character::complete::{space0, space1},
    InputTake,
};
use orgize::{
    ast::{Headline, Keyword, SourceBlock, Token},
    export::{from_fn_with_ctx, Container, Event},
    rowan::ast::AstNode,
    Org, SyntaxKind, SyntaxNode,
};

pub fn collect_src_blocks(org: &Org) -> Vec<SourceBlock> {
    let mut blocks = Vec::<SourceBlock>::new();

    org.traverse(&mut from_fn_with_ctx(|event, ctx| {
        match event {
            Event::Enter(Container::SourceBlock(block)) => {
                blocks.push(block);
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
            | Event::Enter(Container::ExportBlock(_)) => ctx.skip(),
            _ => {}
        }
    }));

    blocks
}

pub fn language_comments(language: &str) -> Option<(&str, &str)> {
    match language {
        "c" | "cpp" | "c++" | "go" | "js" | "javascript" | "ts" | "typescript" | "rust"
        | "vera" | "jsonc" => Some(("//", "")),
        "toml" | "tml" | "yaml" | "yml" | "conf" | "gitconfig" | "conf-toml" | "sh" | "shell"
        | "bash" | "zsh" | "fish" => Some(("#", "")),
        "lua" | "sql" => Some(("--", "")),
        "lisp" | "emacs-lisp" | "elisp" => Some((";;", "")),
        "xml" | "html" | "svg" => Some(("<!--", "-->")),
        _ => None,
    }
}

pub fn language_execute_command(language: &str) -> Option<&str> {
    match language {
        "js" | "javascript" => Some("node"),
        "sh" | "bash" => Some("bash"),
        "py" | "python" => Some("python"),
        "fish" => Some("fish"),
        _ => None,
    }
}

pub fn header_argument<'a>(
    arg1: &'a str,
    arg2: &'a str,
    arg3: &'a str,
    key: &str,
    default: &'static str,
) -> &'a str {
    extract_header_args(arg1, key)
        .or_else(|_| extract_header_args(arg2, key))
        .or_else(|_| extract_header_args(arg3, key))
        .unwrap_or(default)
}

pub fn property_keyword(node: &SyntaxNode) -> Option<Token> {
    node.ancestors()
        .find(|n| n.kind() == SyntaxKind::DOCUMENT)
        .and_then(|n| n.first_child())
        .filter(|n| n.kind() == SyntaxKind::SECTION)
        .and_then(|n| {
            n.children()
                .filter_map(Keyword::cast)
                .filter(|kw| kw.key().eq_ignore_ascii_case("PROPERTY"))
                .map(|kw| kw.value())
                .find(|value| value.trim_start().starts_with("header-args "))
        })
}

pub fn property_drawer(node: &SyntaxNode) -> Option<Token> {
    node.ancestors()
        .find_map(Headline::cast)
        .and_then(|hdl| hdl.properties())
        .and_then(|drawer| drawer.get("header-args"))
}

pub fn extract_header_args<'a>(input: &'a str, key: &str) -> Result<&'a str, nom::Err<()>> {
    let mut i = input;

    while !i.is_empty() {
        let (input, _) = space0(i)?;
        let (input, name) = take_while1(|c| c != ' ' && c != '\t')(input)?;

        if !name.eq_ignore_ascii_case(key) {
            debug_assert!(input.len() < i.len(), "{} < {}", input.len(), i.len());
            i = input;
            continue;
        }

        let (input, _) = space1(input)?;

        if let Some(idx) = Substring::new(" :")
            .find(input)
            .or_else(|| Substring::new("\t:").find(input))
        {
            let idx = input[0..idx]
                .rfind(|c| c != ' ' && c != '\t')
                .map(|i| i + 1)
                .unwrap_or(idx);

            let (_, value) = input.take_split(idx);

            return Ok(value.trim());
        } else {
            return Ok(input.trim());
        }
    }

    Err(nom::Err::Error(()))
}

#[test]
fn parse_header_args() {
    assert!(extract_header_args("", ":tangle").is_err());
    assert!(extract_header_args(" :noweb yes", ":tangle1").is_err());
    assert!(extract_header_args(":tangle", ":tangle").is_err());

    assert_eq!(extract_header_args(":tangle  ", ":tangle").unwrap(), "");

    assert_eq!(
        extract_header_args(":tangle emacs.d/init.el", ":tangle").unwrap(),
        "emacs.d/init.el"
    );
    assert_eq!(
        extract_header_args(" :tangle emacs.d/init.el", ":tangle").unwrap(),
        "emacs.d/init.el"
    );
    assert_eq!(
        extract_header_args(" :tangle emacs.d/init.el  :noweb yes", ":tangle").unwrap(),
        "emacs.d/init.el"
    );
    assert_eq!(
        extract_header_args(" :noweb yes :tangle emacs.d/init.el", ":tangle").unwrap(),
        "emacs.d/init.el"
    );

    assert_eq!(
        extract_header_args(":results output code", ":results").unwrap(),
        "output code"
    );
}
