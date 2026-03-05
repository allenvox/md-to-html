// Entry point. In C this would be main.c, including parser.h, renderer.h.
//
// Modules in Rust (analog of .c/.h in C):
//   mod ast;     -> compiles src/ast.rs; everything in the module is private by default
//   mod parser;  -> compiles src/parser.rs; pub fn/pub struct visible outside
//   mod renderer;
// No separate .h: public API of the module is the pub elements in the .rs file.
// Tests: in each module #[cfg(test)] mod tests { ... }, run: cargo test

mod ast;
mod parser;
mod renderer;

use std::path::PathBuf;

use parser::parse_blocks;
use renderer::{render, wrap_standalone};

/// Parse result: success with paths and flags, or help/version/error.
enum ParseResult {
    Ok((PathBuf, Option<PathBuf>, bool)), // input, output, standalone
    Help,
    Version,
    Error,
}

/// Parsing arguments: md-to-html <input.md> [-o out.html] [--standalone]
fn parse_args() -> ParseResult {
    let argv: Vec<String> = std::env::args().skip(1).collect();
    let mut input = None;
    let mut output = None;
    let mut standalone = false;
    let mut i = 0;

    while i < argv.len() {
        let arg = &argv[i];
        if arg == "-o" || arg == "--output" {
            i += 1;
            if i < argv.len() {
                output = Some(PathBuf::from(&argv[i]));
                i += 1;
            } else {
                eprintln!("missing value for {arg}");
                return ParseResult::Error;
            }
        } else if arg == "-s" || arg == "--standalone" {
            standalone = true;
            i += 1;
        } else if arg == "-h" || arg == "--help" {
            return ParseResult::Help;
        } else if arg == "-V" || arg == "--version" {
            return ParseResult::Version;
        } else if input.is_none() && !arg.starts_with('-') {
            input = Some(PathBuf::from(arg));
            i += 1;
        } else {
            eprintln!("unexpected argument: {arg}");
            return ParseResult::Error;
        }
    }

    match input {
        Some(inp) => ParseResult::Ok((inp, output, standalone)),
        None => ParseResult::Error,
    }
}

fn print_usage() {
    eprintln!(
        "Usage: md-to-html <input.md> [-o output.html] [--standalone]
Options:
  -o, --output <file>   write HTML to file (default: stdout)
  -s, --standalone      wrap in <!DOCTYPE html>, <html>, <body>
  -h, --help            show this help
  -V, --version         show version"
    );
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    match parse_args() {
        ParseResult::Ok((input_path, output_path, standalone)) => {
            let input = std::fs::read_to_string(&input_path)?;
            let blocks = parse_blocks(&input);
            let mut html = render(blocks);
            if standalone {
                let title = input_path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("Document");
                html = wrap_standalone(&html, title);
            }
            match output_path {
                Some(path) => std::fs::write(path, html)?,
                None => print!("{html}"),
            }
            Ok(())
        }
        ParseResult::Help => {
            print_usage();
            std::process::exit(0);
        }
        ParseResult::Version => {
            println!("md-to-html {}", env!("CARGO_PKG_VERSION"));
            std::process::exit(0);
        }
        ParseResult::Error => {
            print_usage();
            std::process::exit(1);
        }
    }
}
