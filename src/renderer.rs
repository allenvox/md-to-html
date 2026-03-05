// Renderer: AST to HTML string.

use crate::ast::{Block, ListItem};

/// HTML escaping: < > & to prevent text from breaking the markup and creating XSS.
/// Analog in C: traverse the string and replace characters in a new buffer.
pub fn escape_html(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            _ => out.push(c),
        }
    }
    out
}

/// Parsing inline: *italic*, **bold**, `code`, [text](url), ![alt](url).
fn render_inline(s: &str) -> String {
    let mut out = String::new();
    let mut rest = s;

    while !rest.is_empty() {
        // Find the nearest of **[**, *[*, `[`, [ (link)
        let link_start = rest.find('[');
        let image_start = rest.find("!["); // image before link when both at same [
        let (start, kind) = [
            (rest.find("**"), "**"),
            (rest.find('*'), "*"),
            (rest.find('`'), "`"),
            (image_start, "!["),
            (link_start, "["),
        ]
        .into_iter()
        .filter_map(|(p, d)| p.map(|pos| (pos, d)))
        .min_by_key(|(p, _)| *p)
        .unwrap_or((rest.len(), ""));

        if start >= rest.len() {
            out.push_str(&escape_html(rest));
            break;
        }

        out.push_str(&escape_html(&rest[..start]));
        rest = &rest[start..];

        if kind == "![" {
            // Image: ![alt](url); rest starts with "!["
            let Some(bracket_end) = rest.find(']') else {
                out.push_str(&escape_html("!"));
                rest = &rest[1..];
                continue;
            };
            let alt = escape_html(&rest[2..bracket_end]); // after "!["
            rest = &rest[bracket_end + 1..];
            let rest_trim = rest.trim_start();
            if rest_trim.starts_with('(') {
                let Some(paren_end) = rest_trim.find(')') else {
                    out.push_str(&escape_html("!["));
                    out.push_str(&alt);
                    out.push_str("]");
                    continue;
                };
                let url = escape_html(&rest_trim[1..paren_end]);
                rest = &rest_trim[paren_end + 1..];
                out.push_str("<img src=\"");
                out.push_str(&url);
                out.push_str("\" alt=\"");
                out.push_str(&alt);
                out.push_str("\">");
            } else {
                out.push_str(&escape_html("!["));
                out.push_str(&alt);
                out.push_str("]");
                rest = rest_trim;
            }
            continue;
        }

        if kind == "[" {
            // Link: [text](url) — find ] then (
            let Some(bracket_end) = rest.find(']') else {
                out.push_str(&escape_html(&rest[..1]));
                rest = &rest[1..];
                continue;
            };
            let text = escape_html(&rest[1..bracket_end]);
            rest = &rest[bracket_end + 1..];
            let rest_trim = rest.trim_start();
            if rest_trim.starts_with('(') {
                let Some(paren_end) = rest_trim.find(')') else {
                    out.push_str(&escape_html("["));
                    out.push_str(&text);
                    out.push_str("]");
                    continue;
                };
                let url = escape_html(&rest_trim[1..paren_end]);
                rest = &rest_trim[paren_end + 1..];
                out.push_str("<a href=\"");
                out.push_str(&url);
                out.push_str("\">");
                out.push_str(&text);
                out.push_str("</a>");
            } else {
                out.push_str(&escape_html("["));
                out.push_str(&text);
                out.push_str("]");
                rest = rest_trim;
            }
            continue;
        }

        let delim = kind;
        rest = &rest[delim.len()..];
        let end = rest.find(delim).unwrap_or(rest.len());
        let inner = escape_html(&rest[..end]);

        match delim {
            "**" => {
                out.push_str("<strong>");
                out.push_str(&inner);
                out.push_str("</strong>");
            }
            "*" => {
                out.push_str("<em>");
                out.push_str(&inner);
                out.push_str("</em>");
            }
            "`" => {
                out.push_str("<code>");
                out.push_str(&inner);
                out.push_str("</code>");
            }
            _ => {}
        }
        rest = if end < rest.len() {
            &rest[end + delim.len()..]
        } else {
            ""
        };
    }

    out
}

/// Converts blocks to HTML fragment.
pub fn render(blocks: Vec<Block>) -> String {
    blocks
        .into_iter()
        .map(|block| match block {
            Block::Heading(lvl, txt) => format!("<h{lvl}>{}</h{lvl}>", render_inline(&txt)),
            Block::Paragraph(txt) => format!("<p>{}</p>", render_inline(&txt)),
            Block::Code(txt) => format!("<pre><code>{}</code></pre>", escape_html(&txt)),
            Block::HorizontalRule => "<hr>".to_string(),
            Block::BulletList(items) => render_list_items(&items, "ul"),
            Block::NumberedList(items) => render_list_items(&items, "ol"),
            Block::Blockquote(lines) => {
                let inner = lines
                    .iter()
                    .map(|s| format!("<p>{}</p>", render_inline(s)))
                    .collect::<Vec<_>>()
                    .join("\n");
                format!("<blockquote>\n{inner}\n</blockquote>")
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn render_list_items(items: &[ListItem], tag: &str) -> String {
    let lis: Vec<String> = items
        .iter()
        .map(|item| {
            let mut s = format!("<li>{}</li>", render_inline(&item.content));
            if !item.nested.is_empty() {
                let sub = item
                    .nested
                    .iter()
                    .map(|n| format!("<li>{}</li>", render_inline(n)))
                    .collect::<Vec<_>>()
                    .join("\n");
                s.push_str("\n<ul>\n");
                s.push_str(&sub);
                s.push_str("\n</ul>");
            }
            s
        })
        .collect();
    format!("<{tag}>\n{}\n</{tag}>", lis.join("\n"))
}

/// Wraps HTML fragment in a full document.
pub fn wrap_standalone(fragment: &str, title: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <title>{}</title>
</head>
<body>
{}
</body>
</html>"#,
        escape_html(title),
        fragment
    )
}

#[cfg(test)]
mod tests {
    use crate::ast::{Block, ListItem};
    use super::{escape_html, render, wrap_standalone};

    #[test]
    fn escape_angles() {
        assert_eq!(escape_html("<script>"), "&lt;script&gt;");
    }

    #[test]
    fn inline_bold_italic() {
        let blocks = vec![
            Block::Paragraph("Say **bold** and *italic*.".to_string()),
        ];
        let html = render(blocks);
        assert!(html.contains("<strong>bold</strong>"));
        assert!(html.contains("<em>italic</em>"));
    }

    #[test]
    fn link() {
        let blocks = vec![
            Block::Paragraph("See [Rust](https://rust-lang.org).".to_string()),
        ];
        let html = render(blocks);
        assert!(html.contains("<a href=\"https://rust-lang.org\">Rust</a>"));
    }

    #[test]
    fn image() {
        let blocks = vec![
            Block::Paragraph("Pic: ![logo](img/logo.png).".to_string()),
        ];
        let html = render(blocks);
        assert!(html.contains("<img src=\"img/logo.png\" alt=\"logo\">"));
    }

    #[test]
    fn standalone_wrap() {
        let html = wrap_standalone("<p>Hi</p>", "Doc");
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("<title>Doc</title>"));
        assert!(html.contains("<body>"));
        assert!(html.contains("<p>Hi</p>"));
    }

    #[test]
    fn nested_list() {
        let blocks = vec![
            Block::BulletList(vec![
                ListItem { content: "a".to_string(), nested: vec!["b".to_string()] },
                ListItem { content: "c".to_string(), nested: vec![] },
            ]),
        ];
        let html = render(blocks);
        assert!(html.contains("<ul>"));
        assert!(html.contains("<li>a</li>"));
        assert!(html.contains("<ul>"));
        assert!(html.contains("<li>b</li>"));
    }
}
