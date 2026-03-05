// AST (Abstract Syntax Tree) — nodes after parsing Markdown.

/// One list item; `nested` is one level of sub-items (indented in source).
#[derive(Debug, Clone)]
pub struct ListItem {
    pub content: String,
    pub nested: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum Block {
    Heading(usize, String),
    Paragraph(String),
    Code(String),
    HorizontalRule,
    BulletList(Vec<ListItem>),
    NumberedList(Vec<ListItem>),
    Blockquote(Vec<String>),
}
