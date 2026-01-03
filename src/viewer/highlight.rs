#![allow(dead_code)]

use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;
use syntect::easy::HighlightLines;
use ratatui::style::Color;

/// A single styled segment of text
#[derive(Clone, Debug)]
pub struct StyledSegment {
    pub text: String,
    pub fg: Color,
    pub bg: Color,
}

/// A highlighted line containing multiple styled segments
#[derive(Clone, Debug)]
pub struct HighlightedLine {
    pub segments: Vec<StyledSegment>,
}

/// Syntax highlighter using syntect
pub struct SyntaxHighlighter {
    syntax_set: SyntaxSet,
    theme_set: ThemeSet,
}

impl SyntaxHighlighter {
    pub fn new() -> Self {
        SyntaxHighlighter {
            syntax_set: SyntaxSet::load_defaults_newlines(),
            theme_set: ThemeSet::load_defaults(),
        }
    }

    /// Get the syntax name for a file extension
    fn get_syntax_name(&self, extension: &str) -> Option<&str> {
        match extension.to_lowercase().as_str() {
            // Python
            "py" | "pyw" | "pyi" => Some("Python"),
            // JavaScript/TypeScript
            "js" | "mjs" | "cjs" => Some("JavaScript"),
            "jsx" => Some("JavaScript (Babel)"),
            "ts" | "mts" | "cts" => Some("TypeScript"),
            "tsx" => Some("TypeScriptReact"),
            // C/C++
            "c" => Some("C"),
            "h" => Some("C"),
            "cpp" | "cc" | "cxx" | "c++" => Some("C++"),
            "hpp" | "hxx" | "h++" => Some("C++"),
            // Go
            "go" => Some("Go"),
            // Rust
            "rs" => Some("Rust"),
            // C#
            "cs" => Some("C#"),
            // Shell scripts (Unix)
            "sh" | "bash" | "zsh" | "ksh" | "csh" | "tcsh" => Some("Bourne Again Shell (bash)"),
            "fish" => Some("Bourne Again Shell (bash)"), // Fish uses similar syntax
            // Windows scripts
            "bat" | "cmd" => Some("Batch File"),
            "ps1" | "psm1" | "psd1" => Some("PowerShell"),
            // Build/Make scripts
            "make" | "makefile" | "mk" | "mak" => Some("Makefile"),
            "cmake" => Some("CMake"),
            "dockerfile" => Some("Dockerfile"),
            // Scripting languages
            "pl" | "pm" | "pod" | "t" => Some("Perl"),
            "rb" | "erb" | "rake" | "gemspec" => Some("Ruby"),
            "php" | "phtml" | "php3" | "php4" | "php5" => Some("PHP"),
            "lua" => Some("Lua"),
            "awk" => Some("AWK"),
            "sed" => Some("Bourne Again Shell (bash)"), // sed scripts use shell-like syntax
            "vim" | "vimrc" => Some("VimL"),
            // JSON
            "json" | "jsonl" | "json5" => Some("JSON"),
            // YAML
            "yaml" | "yml" => Some("YAML"),
            // Additional common formats
            "toml" => Some("TOML"),
            "xml" | "xsl" | "xslt" | "svg" | "plist" => Some("XML"),
            "html" | "htm" | "xhtml" => Some("HTML"),
            "vue" => Some("HTML"), // Vue.js (uses HTML-like syntax)
            "css" | "scss" | "sass" | "less" => Some("CSS"),
            "sql" => Some("SQL"),
            "md" | "markdown" => Some("Markdown"),
            // Config files
            "ini" | "cfg" | "conf" => Some("INI"),
            "properties" => Some("Java Properties"),
            "env" => Some("Bourne Again Shell (bash)"), // .env files use shell-like syntax
            // Java/JVM
            "java" => Some("Java"),
            "kt" | "kts" => Some("Kotlin"),
            "scala" | "sc" => Some("Scala"),
            "groovy" | "gradle" => Some("Groovy"),
            // Swift/Objective-C
            "swift" => Some("Swift"),
            "m" | "mm" => Some("Objective-C"),
            // Haskell/Functional
            "hs" | "lhs" => Some("Haskell"),
            "ml" | "mli" => Some("OCaml"),
            "ex" | "exs" => Some("Elixir"),
            "erl" | "hrl" => Some("Erlang"),
            "clj" | "cljs" | "cljc" | "edn" => Some("Clojure"),
            // Other
            "r" | "rmd" => Some("R"),
            "dart" => Some("Dart"),
            "nim" => Some("Nim"),
            "zig" => Some("Zig"),
            "v" => Some("V"),
            "d" => Some("D"),
            // Assembly
            "asm" | "s" => Some("Assembly (x86_64)"),
            // Diff/Patch
            "diff" | "patch" => Some("Diff"),
            // LaTeX
            "tex" | "latex" | "sty" | "cls" => Some("LaTeX"),
            // Protobuf/GraphQL
            "proto" => Some("Protocol Buffer"),
            "graphql" | "gql" => Some("GraphQL"),
            _ => None,
        }
    }

    /// Check if an extension is supported for syntax highlighting
    pub fn is_supported(&self, extension: &str) -> bool {
        // Check by extension first (most reliable)
        self.syntax_set.find_syntax_by_extension(extension).is_some()
            || self.get_syntax_name(extension).is_some()
    }

    /// Highlight code content
    pub fn highlight(&self, content: &str, extension: &str) -> Vec<HighlightedLine> {
        // Try by extension first (most reliable)
        let syntax = if let Some(s) = self.syntax_set.find_syntax_by_extension(extension) {
            s
        } else if let Some(syntax_name) = self.get_syntax_name(extension) {
            // Try by syntax name as fallback
            match self.syntax_set.find_syntax_by_name(syntax_name) {
                Some(s) => s,
                None => return self.plain_text_lines(content),
            }
        } else {
            return self.plain_text_lines(content);
        };

        // Use base16-ocean.dark theme (terminal-friendly)
        let theme = &self.theme_set.themes["base16-ocean.dark"];
        let mut highlighter = HighlightLines::new(syntax, theme);

        let mut result = Vec::new();

        for line in content.lines() {
            let ranges = match highlighter.highlight_line(line, &self.syntax_set) {
                Ok(r) => r,
                Err(_) => {
                    // On error, return plain text for this line
                    result.push(HighlightedLine {
                        segments: vec![StyledSegment {
                            text: line.to_string(),
                            fg: Color::White,
                            bg: Color::Reset,
                        }],
                    });
                    continue;
                }
            };

            let segments: Vec<StyledSegment> = ranges
                .iter()
                .map(|(style, text)| StyledSegment {
                    text: text.to_string(),
                    fg: syntect_to_ratatui_color(style.foreground),
                    bg: Color::Reset, // Use terminal background
                })
                .collect();

            result.push(HighlightedLine { segments });
        }

        result
    }

    /// Convert plain text to unhighlighted lines
    fn plain_text_lines(&self, content: &str) -> Vec<HighlightedLine> {
        content
            .lines()
            .map(|line| HighlightedLine {
                segments: vec![StyledSegment {
                    text: line.to_string(),
                    fg: Color::White,
                    bg: Color::Reset,
                }],
            })
            .collect()
    }
}

impl Default for SyntaxHighlighter {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert syntect color to ratatui color
fn syntect_to_ratatui_color(color: syntect::highlighting::Color) -> Color {
    Color::Rgb(color.r, color.g, color.b)
}

/// Global highlighter instance (lazy initialization)
use std::sync::OnceLock;

static HIGHLIGHTER: OnceLock<SyntaxHighlighter> = OnceLock::new();

/// Get or create the global highlighter instance
pub fn get_highlighter() -> &'static SyntaxHighlighter {
    HIGHLIGHTER.get_or_init(SyntaxHighlighter::new)
}

/// Highlight code with the global highlighter
pub fn highlight_code(content: &str, extension: &str) -> Vec<HighlightedLine> {
    get_highlighter().highlight(content, extension)
}

/// Check if extension is supported for highlighting
pub fn is_highlight_supported(extension: &str) -> bool {
    get_highlighter().is_supported(extension)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_python_highlighting() {
        let code = r#"def hello():
    print("Hello, World!")
"#;
        let result = highlight_code(code, "py");
        assert!(!result.is_empty());
    }

    #[test]
    fn test_rust_highlighting() {
        let code = r#"fn main() {
    println!("Hello, World!");
}
"#;
        let result = highlight_code(code, "rs");
        assert!(!result.is_empty());
    }

    #[test]
    fn test_json_highlighting() {
        let code = r#"{"key": "value", "number": 42}"#;
        let result = highlight_code(code, "json");
        assert!(!result.is_empty());
    }

    #[test]
    fn test_unsupported_extension() {
        let code = "some text content";
        let result = highlight_code(code, "xyz");
        assert!(!result.is_empty());
        // Should return plain white text
        assert_eq!(result[0].segments[0].fg, Color::White);
    }

    #[test]
    fn test_shell_highlighting() {
        let code = r#"#!/bin/bash
echo "Hello, World!"
if [ -f "file.txt" ]; then
    cat file.txt
fi
"#;
        let result = highlight_code(code, "sh");
        assert!(!result.is_empty());
        // Should have syntax highlighting (not plain white)
        assert!(is_highlight_supported("sh"));
        assert!(is_highlight_supported("bash"));
        assert!(is_highlight_supported("zsh"));
    }

    #[test]
    fn test_batch_highlighting() {
        let code = r#"@echo off
echo Hello, World!
if exist "file.txt" (
    type file.txt
)
"#;
        let result = highlight_code(code, "bat");
        assert!(!result.is_empty());
        // Should have syntax highlighting
        assert!(is_highlight_supported("bat"));
        assert!(is_highlight_supported("cmd"));
    }
}

