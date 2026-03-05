// Parser: text to AST (list of blocks).

use crate::ast::{Block, ListItem};

/// Bullet item: "- ", "* ", "+ " at the beginning of the line (after trim).
fn is_bullet_item(trimmed: &str) -> bool {
    trimmed.starts_with("- ") || trimmed.starts_with("* ") || trimmed.starts_with("+ ")
}

/// Numbered item: "1. ", "2. " and so on (digits + ". ").
fn is_numbered_item(trimmed: &str) -> Option<usize> {
    let rest = trimmed.trim_start_matches(|c: char| c.is_ascii_digit());
    if rest.starts_with(". ") && trimmed.len() > rest.len() {
        Some(trimmed.len() - rest.len()) // длина префикса "N."
    } else {
        None
    }
}

/// Quote line: starts with ">" (after trim).
fn is_blockquote_line(trimmed: &str) -> bool {
    trimmed.starts_with('>')
}

/// Content of quote line: remove ">" and optional space.
fn strip_blockquote_prefix(line: &str) -> &str {
    let t = line.trim();
    if t.starts_with("> ") {
        t[2..].trim()
    } else if t.starts_with('>') {
        t[1..].trim()
    } else {
        t
    }
}

/// Horizontal rule: 3+ of - or * or _ (optional trailing spaces).
fn is_horizontal_rule(trimmed: &str) -> bool {
    let t = trimmed.trim();
    if t.len() < 3 {
        return false;
    }
    let c = t.chars().next().unwrap();
    (c == '-' || c == '*' || c == '_') && t.chars().all(|x| x == c)
}

/// Parses the entire input into a list of blocks.
/// Handles code blocks, lists, blockquotes.
pub fn parse_blocks(input: &str) -> Vec<Block> {
    let mut blocks = Vec::new();
    let mut in_code = false;
    let mut code_lines = Vec::new();
    let mut list_bullet: Option<Vec<ListItem>> = None;
    let mut list_numbered: Option<Vec<ListItem>> = None;
    let mut blockquote_lines: Option<Vec<String>> = None;
    let mut paragraph_lines: Option<Vec<String>> = None;

    fn flush_paragraph(blocks: &mut Vec<Block>, acc: &mut Option<Vec<String>>) {
        if let Some(lines) = acc.take() {
            if !lines.is_empty() {
                blocks.push(Block::Paragraph(lines.join("\n")));
            }
        }
    }
    fn flush_bullet(blocks: &mut Vec<Block>, acc: &mut Option<Vec<ListItem>>) {
        if let Some(items) = acc.take() {
            if !items.is_empty() {
                blocks.push(Block::BulletList(items));
            }
        }
    }
    fn flush_numbered(blocks: &mut Vec<Block>, acc: &mut Option<Vec<ListItem>>) {
        if let Some(items) = acc.take() {
            if !items.is_empty() {
                blocks.push(Block::NumberedList(items));
            }
        }
    }
    fn flush_blockquote(blocks: &mut Vec<Block>, acc: &mut Option<Vec<String>>) {
        if let Some(lines) = acc.take() {
            if !lines.is_empty() {
                blocks.push(Block::Blockquote(lines));
            }
        }
    }

    for line in input.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("```") {
            flush_paragraph(&mut blocks, &mut paragraph_lines);
            flush_bullet(&mut blocks, &mut list_bullet);
            flush_numbered(&mut blocks, &mut list_numbered);
            flush_blockquote(&mut blocks, &mut blockquote_lines);
            if in_code {
                let content = code_lines.join("\n");
                blocks.push(Block::Code(content));
                code_lines.clear();
                in_code = false;
            } else {
                in_code = true;
            }
            continue;
        }

        if in_code {
            code_lines.push(line.to_string());
            continue;
        }

        if trimmed.is_empty() {
            flush_paragraph(&mut blocks, &mut paragraph_lines);
            flush_bullet(&mut blocks, &mut list_bullet);
            flush_numbered(&mut blocks, &mut list_numbered);
            flush_blockquote(&mut blocks, &mut blockquote_lines);
            continue;
        }

        if is_horizontal_rule(trimmed) {
            flush_paragraph(&mut blocks, &mut paragraph_lines);
            flush_bullet(&mut blocks, &mut list_bullet);
            flush_numbered(&mut blocks, &mut list_numbered);
            flush_blockquote(&mut blocks, &mut blockquote_lines);
            blocks.push(Block::HorizontalRule);
            continue;
        }

        let has_indent = line.starts_with(' ') || line.starts_with('\t');

        if is_bullet_item(trimmed) {
            flush_paragraph(&mut blocks, &mut paragraph_lines);
            flush_numbered(&mut blocks, &mut list_numbered);
            flush_blockquote(&mut blocks, &mut blockquote_lines);
            let content = trimmed[2..].trim().to_string();
            let list = list_bullet.get_or_insert_with(Vec::new);
            if has_indent && !list.is_empty() {
                list.last_mut().unwrap().nested.push(content);
            } else {
                list.push(ListItem { content, nested: Vec::new() });
            }
            continue;
        }

        if let Some(prefix_len) = is_numbered_item(trimmed) {
            flush_paragraph(&mut blocks, &mut paragraph_lines);
            flush_bullet(&mut blocks, &mut list_bullet);
            flush_blockquote(&mut blocks, &mut blockquote_lines);
            let content = trimmed[prefix_len + 2..].trim().to_string();
            let list = list_numbered.get_or_insert_with(Vec::new);
            if has_indent && !list.is_empty() {
                list.last_mut().unwrap().nested.push(content);
            } else {
                list.push(ListItem { content, nested: Vec::new() });
            }
            continue;
        }

        if is_blockquote_line(trimmed) {
            flush_paragraph(&mut blocks, &mut paragraph_lines);
            flush_bullet(&mut blocks, &mut list_bullet);
            flush_numbered(&mut blocks, &mut list_numbered);
            let content = strip_blockquote_prefix(trimmed).to_string();
            blockquote_lines.get_or_insert_with(Vec::new).push(content);
            continue;
        }

        // Heading or paragraph
        flush_bullet(&mut blocks, &mut list_bullet);
        flush_numbered(&mut blocks, &mut list_numbered);
        flush_blockquote(&mut blocks, &mut blockquote_lines);

        if trimmed.starts_with('#') {
            flush_paragraph(&mut blocks, &mut paragraph_lines);
            blocks.push(parse_line(trimmed));
        } else {
            paragraph_lines.get_or_insert_with(Vec::new).push(trimmed.to_string());
        }
    }

    flush_paragraph(&mut blocks, &mut paragraph_lines);
    flush_bullet(&mut blocks, &mut list_bullet);
    flush_numbered(&mut blocks, &mut list_numbered);
    flush_blockquote(&mut blocks, &mut blockquote_lines);

    if in_code && !code_lines.is_empty() {
        blocks.push(Block::Code(code_lines.join("\n")));
    }

    blocks
}

/// Parsing one line (heading, paragraph). Called only outside the code block.
fn parse_line(line: &str) -> Block {
    if line.starts_with('#') {
        let level = line.chars().take_while(|&c| c == '#').count();
        let content = line.trim_start_matches('#').trim().to_string();
        Block::Heading(level.min(6), content) // h1..h6 headings
    } else {
        Block::Paragraph(line.to_string())
    }
}

#[cfg(test)] // tests are only compiled when cargo test is run
mod tests {
    use super::*;

    #[test]
    fn heading_level() {
        let b = parse_blocks("# H1");
        assert!(matches!(&b[0], Block::Heading(1, t) if t == "H1"));
        let b = parse_blocks("## H2");
        assert!(matches!(&b[0], Block::Heading(2, t) if t == "H2"));
    }

    #[test]
    fn code_block() {
        let b = parse_blocks("```\nfn main() {}\n```");
        assert!(matches!(&b[0], Block::Code(c) if c == "fn main() {}"));
    }

    #[test]
    fn bullet_list() {
        let b = parse_blocks("- one\n- two");
        assert!(matches!(&b[0], Block::BulletList(v) if v.len() == 2 && v[0].content == "one" && v[1].content == "two"));
    }

    #[test]
    fn horizontal_rule() {
        let b = parse_blocks("---");
        assert!(matches!(&b[0], Block::HorizontalRule));
    }

    #[test]
    fn paragraph_merge() {
        let b = parse_blocks("line one\nline two");
        assert!(matches!(&b[0], Block::Paragraph(p) if p == "line one\nline two"));
    }

    #[test]
    fn blockquote() {
        let b = parse_blocks("> quote line");
        assert!(matches!(&b[0], Block::Blockquote(v) if v.len() == 1 && v[0] == "quote line"));
    }
}
