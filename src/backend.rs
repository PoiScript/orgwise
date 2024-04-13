use dashmap::DashMap;
use lsp_types::*;
use orgize::{export::Traverser, Org};
use orgize::{rowan::TextRange, ParseConfig};
use std::iter::once;

pub struct OrgDocument {
    pub text: String,
    pub line_starts: Vec<u32>,
    pub org: Org,
}

impl OrgDocument {
    pub fn new(text: impl AsRef<str>, config: ParseConfig) -> Self {
        let text = text.as_ref().to_string();

        OrgDocument {
            org: config.parse(&text),
            line_starts: line_starts(&text),
            text,
        }
    }

    pub fn update(&mut self, start: u32, end: u32, text: &str) {
        self.text
            .replace_range((start as usize)..(end as usize), text);

        self.line_starts = line_starts(&self.text);

        self.org
            .replace_range(TextRange::new(start.into(), end.into()), text);
    }

    pub fn position_of(&self, offset: u32) -> Position {
        let line = self
            .line_starts
            .binary_search(&offset)
            .unwrap_or_else(|i| i - 1);

        let line_start = self.line_starts[line];

        let character = self.text.as_str()[(line_start as usize)..(offset as usize)]
            .chars()
            .count();

        Position::new(line as u32, character as u32)
    }

    pub fn line_of(&self, offset: u32) -> u32 {
        self.line_starts
            .binary_search(&offset)
            .unwrap_or_else(|i| i - 1) as u32
    }

    pub fn range_of(&self, range: TextRange) -> Range {
        self.range_of2(range.start(), range.end())
    }

    pub fn range_of2<T: Into<u32>>(&self, start_offset: T, end_offset: T) -> Range {
        Range::new(
            self.position_of(start_offset.into()),
            self.position_of(end_offset.into()),
        )
    }

    pub fn offset_of(&self, position: Position) -> u32 {
        let line_start = self.line_starts[position.line as usize] as usize;

        let line_end = match self.line_starts.get((position.line + 1) as usize) {
            Some(x) => *x as usize,
            None => self.text.len(),
        };

        if position.character == 0 {
            return line_start as u32;
        }

        let line_str = &self.text.as_str()[line_start..line_end];

        let index = line_str
            .char_indices()
            .nth(position.character as usize)
            .map(|(i, _)| i)
            .unwrap_or_else(|| line_str.len());

        (line_start + index) as u32
    }

    pub fn traverse<H: Traverser>(&self, h: &mut H) {
        self.org.traverse(h);
    }
}

fn line_starts(text: &str) -> Vec<u32> {
    let bytes = text.as_bytes();

    once(0)
        .chain(
            memchr::memchr2_iter(b'\r', b'\n', bytes)
                .filter(|&i| bytes[i] == b'\n' || !matches!(bytes.get(i + 1), Some(b'\n')))
                .map(|i| (i + 1) as u32),
        )
        .collect()
}

#[test]
fn test() {
    let doc = OrgDocument::new(
        r#"* toc :toc:

fsfs
fasdfs



fasdfs
 
*a* _a_ /1/ ~default~ =default= a_a

# abc

* abc12121
12121


#+begin_src javascript
console.log(a);
#+end_src

"#,
        ParseConfig::default(),
    );

    let start = 12;
    let start_position = Position {
        line: 1,
        character: 0,
    };
    let end = 81;
    let end_position = Position {
        line: 13,
        character: 0,
    };

    assert_eq!(doc.position_of(start), start_position);
    assert_eq!(doc.position_of(end), end_position);

    assert_eq!(doc.offset_of(start_position), start);
    assert_eq!(doc.offset_of(end_position), end);

    let doc = OrgDocument::new("ab", ParseConfig::default());
    assert_eq!(
        doc.offset_of(Position {
            line: 0,
            character: 2,
        }),
        2
    );
    let doc = OrgDocument::new("\nab", ParseConfig::default());
    assert_eq!(
        doc.offset_of(Position {
            line: 1,
            character: 2,
        }),
        3
    );
    let doc = OrgDocument::new("ab\n", ParseConfig::default());
    assert_eq!(
        doc.offset_of(Position {
            line: 0,
            character: 2,
        }),
        2
    );
}

pub trait Backend {
    fn documents(&self) -> &DashMap<Url, OrgDocument>;

    fn default_parse_config(&self) -> ParseConfig;

    fn set_default_parse_config(&self, config: ParseConfig);

    fn home_dir(&self) -> Option<Url> {
        None
    }

    async fn write(&self, url: &Url, content: &str) -> anyhow::Result<()> {
        let _ = (url, content);
        anyhow::bail!("unimplemented")
    }

    async fn read_to_string(&self, url: &Url) -> anyhow::Result<String> {
        let _ = url;
        anyhow::bail!("unimplemented")
    }

    fn resolve_in(&self, url: &str, base: &Url) -> anyhow::Result<Url> {
        if let Some(url) = url.strip_prefix("~/") {
            if let Some(home_dir) = self.home_dir() {
                return Ok(Url::options().base_url(Some(&home_dir)).parse(url)?);
            }
        }

        Ok(Url::options().base_url(Some(base)).parse(url)?)
    }

    async fn log_message(&self, ty: MessageType, message: String) {
        let _ = (ty, message);
    }

    async fn show_message(&self, ty: MessageType, message: String) {
        let _ = (ty, message);
    }

    async fn apply_edit(&self, url: Url, new_text: String, range: TextRange) -> anyhow::Result<()> {
        self.apply_edits(std::iter::once((url, new_text, range)))
            .await
    }

    async fn apply_edits(
        &self,
        items: impl Iterator<Item = (Url, String, TextRange)>,
    ) -> anyhow::Result<()> {
        let _ = items;
        anyhow::bail!("unimplemented")
    }

    async fn execute(&self, executable: &str, content: &str) -> anyhow::Result<String> {
        let _ = (executable, content);
        anyhow::bail!("unimplemented")
    }

    fn add_doc(&self, url: Url, content: String) {
        let config = self.default_parse_config().clone();
        self.documents()
            .insert(url, OrgDocument::new(content, config));
    }

    fn update_doc(&self, url: Url, range: Option<Range>, new_text: String) {
        if let (Some(mut doc), Some(range)) = (self.documents().get_mut(&url), range) {
            let start = doc.offset_of(range.start);
            let end = doc.offset_of(range.end);
            doc.update(start, end, &new_text);
        } else {
            let config = self.default_parse_config().clone();

            self.documents()
                .insert(url, OrgDocument::new(new_text, config));
        }
    }
}
