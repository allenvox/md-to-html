# md-to-html

CLI utility that converts Markdown to HTML. No dependencies. Output is a fragment (no `<html>`/`<body>`), suitable for embedding.

## Build & run

```bash
cargo build --release
./target/release/md-to-html input.md -o output.html
```

Without `-o`, output goes to stdout. `-s` / `--standalone` wraps output in a full HTML document (title from filename). `--help`, `--version` supported.

## Architecture

- **`src/ast.rs`** - `Block` enum and `ListItem` (content + nested). Blocks: Heading, Paragraph, Code, HorizontalRule, BulletList, NumberedList, Blockquote.
- **`src/parser.rs`** - `parse_blocks(input)` -> `Vec<Block>`. Single pass: code blocks, horizontal rules, lists (indent = nested), blockquotes, paragraph merging, headings.
- **`src/renderer.rs`** - `render(blocks)` -> fragment; `wrap_standalone(fragment, title)` -> full doc. Inline: `**bold**`, `*italic*`, `` `code` ``, `[link](url)`, `![alt](url)`.
- **`src/main.rs`** - Manual CLI, I/O, parser -> renderer, optional standalone wrap.

Pipeline: **file -> parse_blocks -> render -> [wrap_standalone] -> write**.

## Supported Markdown

| Block          | Syntax                          |
|----------------|----------------------------------|
| Headings       | `#` … `######`                   |
| Paragraphs     | plain text (merged by newlines)  |
| Code block     | ` ``` ` … ` ``` `                |
| Horizontal rule| `---` / `***` / `___` (3+)        |
| Bullet list    | `- ` / `* ` / `+ `; indent = nested |
| Numbered       | `1. ` …; indent = nested         |
| Blockquote     | `> `                             |

Inline: **bold**, *italic*, `code`, [link](url), ![alt](url).

## Tests

```bash
cargo test
```
