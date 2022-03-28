pub type MarkdownText = Vec<MarkdownInline>;

#[derive(Clone, Debug, PartialEq)]
pub enum Markdown {
    Heading(usize, MarkdownText),
    OrderedList(Vec<MarkdownText>),
    UnorderedList(Vec<MarkdownText>),
    Line(MarkdownText),
    Codeblock(String, String),
    HorizontalRule,
}

#[derive(Clone, Debug, PartialEq)]
pub enum MarkdownInline {
    Link(String, String),
    Image(String, String),
    InlineCode(String),
    Bold(String),
    Italic(String),
    Strike(String),
    Plaintext(String),
}
