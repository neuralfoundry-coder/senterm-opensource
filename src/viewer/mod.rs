use std::fs;
use std::path::{Path, PathBuf};

pub mod text;
pub mod editor;
pub mod highlight;
pub mod image;

pub use editor::{TextEditor, VimMode, EditorStyle};
pub use highlight::{HighlightedLine, highlight_code, is_highlight_supported};
pub use image::{ImagePreview, load_image_auto};

/// Format binary data as hex view
pub fn format_hex_view(data: &[u8], truncated: bool) -> String {
    // Estimate capacity to avoid reallocations
    // Each 16 bytes line is ~85 chars. Ratio ~5.3. Using 6x to be safe.
    let estimated_capacity = data.len() * 6 + 1024;
    let mut output = String::with_capacity(estimated_capacity);
    
    // Header
    output.push_str("╔══════════════════════════════════════════════════════════════════════════════════╗\n");
    output.push_str("║                              HEX VIEWER                                          ║\n");
    output.push_str("╠══════════════════════════════════════════════════════════════════════════════════╣\n");
    
    if truncated {
        output.push_str("║ NOTE: File is larger than 100MB. Showing first 1MB only.                        ║\n");
        output.push_str("╠══════════════════════════════════════════════════════════════════════════════════╣\n");
    }
    
    output.push_str("║ Offset    00 01 02 03 04 05 06 07  08 09 0A 0B 0C 0D 0E 0F  ASCII                ║\n");
    output.push_str("╠══════════════════════════════════════════════════════════════════════════════════╣\n");
    
    // Process data in 16-byte chunks
    for (offset, chunk) in data.chunks(16).enumerate() {
        let address = offset * 16;
        output.push_str(&format!("║ {:08X}  ", address));
        
        // Hex values (first 8 bytes)
        for i in 0..8 {
            if i < chunk.len() {
                output.push_str(&format!("{:02X} ", chunk[i]));
            } else {
                output.push_str("   ");
            }
        }
        output.push(' ');
        
        // Hex values (next 8 bytes)
        for i in 8..16 {
            if i < chunk.len() {
                output.push_str(&format!("{:02X} ", chunk[i]));
            } else {
                output.push_str("   ");
            }
        }
        output.push(' ');
        
        // ASCII representation
        for &byte in chunk {
            let ch = if byte >= 0x20 && byte <= 0x7E {
                byte as char
            } else {
                '.'
            };
            output.push(ch);
        }
        
        // Padding for incomplete lines
        for _ in chunk.len()..16 {
            output.push(' ');
        }
        
        output.push_str(" ║\n");
    }
    
    output.push_str("╚══════════════════════════════════════════════════════════════════════════════════╝\n");
    output
}

/// Format JSON with pretty-printing
/// If the JSON is invalid, returns the original content with an error comment
pub fn format_json(content: &str) -> String {
    // Try to parse and pretty-print
    match serde_json::from_str::<serde_json::Value>(content) {
        Ok(value) => {
            // Successfully parsed - pretty print with 2-space indent
            serde_json::to_string_pretty(&value).unwrap_or_else(|_| content.to_string())
        },
        Err(e) => {
            // Invalid JSON - return original with error indicator
            format!("// JSON Parse Error: {}\n// Showing original content:\n\n{}", e, content)
        }
    }
}

#[derive(Clone)]
pub enum ViewerContent {
    PlainText(String),
    HighlightedCode { raw: String, highlighted: Vec<HighlightedLine> },
    Markdown(String),
    Image(PathBuf), // Store path to image file (legacy, for metadata display)
    ImagePreviewContent(ImagePreview), // Rendered image preview
    HexView(Vec<u8>, bool), // Binary data and whether it was truncated
    Error(String),
}

/// Check if a file type is supported for preview
pub fn is_supported_file_type(path: &Path) -> bool {
    let extension = path.extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_lowercase());
    
    // Also check filename for special files like Makefile, Dockerfile
    let filename = path.file_name()
        .and_then(|n| n.to_str())
        .map(|s| s.to_lowercase());
    
    // Special filenames without extensions
    if let Some(name) = &filename {
        if matches!(name.as_str(), 
            "makefile" | "dockerfile" | "gemfile" | "rakefile" | 
            "vagrantfile" | "jenkinsfile" | "cmakelists.txt" |
            ".bashrc" | ".zshrc" | ".bash_profile" | ".profile" |
            ".gitignore" | ".gitattributes" | ".editorconfig" |
            ".env" | ".env.local" | ".env.development" | ".env.production"
        ) {
            return true;
        }
    }
    
    match extension.as_deref() {
        // Image files
        Some("jpg") | Some("jpeg") | Some("png") | Some("gif") | Some("svg") |
        // Text/code files
        Some("md") | Some("markdown") |
        Some("txt") | Some("log") | Some("ini") | Some("conf") | Some("cfg") |
        // Programming languages
        Some("rs") | Some("go") | Some("c") | Some("cpp") | Some("h") | Some("hpp") |
        Some("java") | Some("kt") | Some("scala") | Some("groovy") |
        Some("py") | Some("pyw") | Some("rb") | Some("php") | Some("lua") | Some("pl") |
        Some("js") | Some("jsx") | Some("ts") | Some("tsx") | Some("mjs") | Some("cjs") |
        Some("cs") | Some("swift") | Some("m") | Some("mm") |
        Some("hs") | Some("ml") | Some("ex") | Some("exs") | Some("erl") | Some("clj") |
        Some("r") | Some("dart") | Some("nim") | Some("zig") | Some("v") | Some("d") |
        // Shell scripts
        Some("sh") | Some("bash") | Some("zsh") | Some("ksh") | Some("csh") | Some("fish") |
        Some("bat") | Some("cmd") | Some("ps1") | Some("psm1") |
        // Build/config files
        Some("toml") | Some("json") | Some("yaml") | Some("yml") | 
        Some("xml") | Some("html") | Some("htm") | Some("vue") | Some("css") | Some("scss") | Some("sass") |
        Some("sql") | Some("graphql") | Some("gql") |
        Some("make") | Some("mk") | Some("cmake") |
        Some("dockerfile") | Some("vagrantfile") |
        // Other
        Some("diff") | Some("patch") | Some("tex") | Some("proto") |
        Some("asm") | Some("s") | Some("vim") | Some("awk") | Some("sed") |
        // Document files
        Some("docx") | Some("xlsx") | Some("xls") => true,
        _ => {
            // Try to read as text to see if it's a text file
            fs::read_to_string(path).is_ok()
        }
    }
}

/// Get syntax extension from special filenames (Makefile, Dockerfile, etc.)
fn get_extension_for_special_file(path: &Path) -> Option<&'static str> {
    let filename = path.file_name()
        .and_then(|n| n.to_str())
        .map(|s| s.to_lowercase())?;
    
    match filename.as_str() {
        "makefile" | "gnumakefile" => Some("make"),
        "dockerfile" => Some("dockerfile"),
        "cmakelists.txt" => Some("cmake"),
        "gemfile" | "rakefile" => Some("rb"),
        "vagrantfile" => Some("rb"),
        "jenkinsfile" => Some("groovy"),
        ".bashrc" | ".bash_profile" | ".bash_aliases" | ".profile" => Some("sh"),
        ".zshrc" | ".zprofile" | ".zshenv" => Some("sh"),
        ".vimrc" | ".gvimrc" => Some("vim"),
        ".gitignore" | ".gitattributes" | ".gitmodules" => Some("ini"),
        ".editorconfig" => Some("ini"),
        ".env" | ".env.local" | ".env.development" | ".env.production" | ".env.test" => Some("sh"),
        ".htaccess" => Some("sh"),
        "config" if path.parent().map(|p| p.ends_with(".ssh")).unwrap_or(false) => Some("sh"),
        _ => None,
    }
}

pub fn load_file(path: &Path) -> ViewerContent {
    // Check for special filenames first (Makefile, Dockerfile, etc.)
    if let Some(ext) = get_extension_for_special_file(path) {
        match fs::read_to_string(path) {
            Ok(content) => {
                let highlighted = highlight_code(&content, ext);
                return ViewerContent::HighlightedCode { raw: content, highlighted };
            },
            Err(e) => return ViewerContent::Error(format!("Failed to read file: {}", e)),
        }
    }
    
    // Determine file type by extension
    let extension = path.extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_lowercase());
    
    match extension.as_deref() {
        // Image files - render preview
        Some("jpg") | Some("jpeg") | Some("png") | Some("gif") => {
            if path.exists() {
                // Render image preview (default size, will be adjusted by UI)
                let preview = load_image_auto(path, 80, 40);
                ViewerContent::ImagePreviewContent(preview)
            } else {
                ViewerContent::Error("Image file not found".to_string())
            }
        },
        // SVG - just show metadata for now (complex to render)
        Some("svg") => {
            if path.exists() {
                ViewerContent::Image(path.to_path_buf())
            } else {
                ViewerContent::Error("Image file not found".to_string())
            }
        },
        Some("md") | Some("markdown") => {
            match fs::read_to_string(path) {
                Ok(content) => ViewerContent::Markdown(content),
                Err(e) => ViewerContent::Error(format!("Failed to read file: {}", e)),
            }
        },
        // JSON files - auto format and syntax highlight
        Some("json") => {
            match fs::read_to_string(path) {
                Ok(content) => {
                    // Try to parse and pretty-print JSON
                    let formatted = format_json(&content);
                    let highlighted = highlight_code(&formatted, "json");
                    ViewerContent::HighlightedCode { raw: formatted, highlighted }
                },
                Err(e) => ViewerContent::Error(format!("Failed to read file: {}", e)),
            }
        },
        // Code files with syntax highlighting support
        Some(ext) if is_highlight_supported(ext) => {
            match fs::read_to_string(path) {
                Ok(content) => {
                    let highlighted = highlight_code(&content, ext);
                    ViewerContent::HighlightedCode { raw: content, highlighted }
                },
                Err(e) => ViewerContent::Error(format!("Failed to read file: {}", e)),
            }
        },
        // Plain text files without highlighting
        Some("txt") | Some("log") => {
            match fs::read_to_string(path) {
                Ok(content) => ViewerContent::PlainText(content),
                Err(e) => ViewerContent::Error(format!("Failed to read file: {}", e)),
            }
        },
        // Config files - try to apply INI highlighting
        Some("conf") | Some("cfg") | Some("ini") => {
            match fs::read_to_string(path) {
                Ok(content) => {
                    if is_highlight_supported("ini") {
                        let highlighted = highlight_code(&content, "ini");
                        ViewerContent::HighlightedCode { raw: content, highlighted }
                    } else {
                        ViewerContent::PlainText(content)
                    }
                },
                Err(e) => ViewerContent::Error(format!("Failed to read file: {}", e)),
            }
        },
        Some("docx") => {
            text::load_docx(path)
        },
        Some("xlsx") | Some("xls") => {
            text::load_xlsx(path)
        },
        Some("hwp") | Some("hwpx") => {
            ViewerContent::Error("HWP/HWPX support is experimental and not yet fully implemented".to_string())
        },
        _ => {
            // Unknown file type - check file size first
            let metadata = match fs::metadata(path) {
                Ok(m) => m,
                Err(e) => return ViewerContent::Error(format!("Failed to get file info: {}", e)),
            };
            
            let file_size = metadata.len();
            const MAX_PLAIN_TEXT_SIZE: u64 = 1024 * 1024; // 1MB
            const MAX_BINARY_SIZE: u64 = 5 * 1024 * 1024; // 5MB for binary/hex view
            
            // For files larger than 1MB, try binary view directly
            if file_size > MAX_PLAIN_TEXT_SIZE {
                if file_size > MAX_BINARY_SIZE {
                    return ViewerContent::Error(format!(
                        "File is too large to preview ({:.2} MB).\n\nMaximum size for hex view: 5MB\nUse an external viewer for larger files.",
                        file_size as f64 / (1024.0 * 1024.0)
                    ));
                }
                // Load as binary hex view
                return load_binary_file(path);
            }
            
            // Try to read as text (1MB or less)
            match fs::read_to_string(path) {
                Ok(content) => {
                    // Check if content looks like binary (contains null bytes or too many non-printable chars)
                    if is_likely_binary(&content) {
                        // Load as hex view instead of error
                        load_binary_file(path)
                    } else if let Some(extension) = path.extension().and_then(|e| e.to_str()) {
                        if is_highlight_supported(extension) {
                            let highlighted = highlight_code(&content, extension);
                            ViewerContent::HighlightedCode { raw: content, highlighted }
                        } else {
                            ViewerContent::PlainText(content)
                        }
                    } else {
                        ViewerContent::PlainText(content)
                    }
                },
                Err(_) => {
                    // Failed to read as text - try as binary hex view
                    load_binary_file(path)
                }
            }
        }
    }
}

/// Check if content is likely binary (contains null bytes or too many control characters)
fn is_likely_binary(content: &str) -> bool {
    if content.is_empty() {
        return false;
    }
    
    // Check first 8KB for binary indicators
    let sample = &content[..content.len().min(8192)];
    let mut control_chars = 0;
    
    for ch in sample.chars() {
        // Null byte is a strong indicator of binary
        if ch == '\0' {
            return true;
        }
        // Count control characters (except common whitespace)
        if ch.is_control() && !matches!(ch, '\n' | '\r' | '\t') {
            control_chars += 1;
        }
    }
    
    // If more than 10% are control characters, likely binary
    let ratio = control_chars as f64 / sample.len() as f64;
    ratio > 0.1
}

fn load_binary_file(path: &Path) -> ViewerContent {
    use std::fs::File;
    use std::io::Read;

    const MAX_BINARY_VIEW_SIZE: u64 = 5 * 1024 * 1024; // 5MB max for hex view

    let metadata = match fs::metadata(path) {
        Ok(m) => m,
        Err(e) => return ViewerContent::Error(format!("Failed to get metadata: {}", e)),
    };

    let file_size = metadata.len();
    
    // Check 5MB limit for binary view
    if file_size > MAX_BINARY_VIEW_SIZE {
        return ViewerContent::Error(format!(
            "Binary file is too large to preview ({:.2} MB).\n\nMaximum size for hex view: 5MB\nUse an external hex editor for larger files.",
            file_size as f64 / (1024.0 * 1024.0)
        ));
    }

    let file = match File::open(path) {
        Ok(f) => f,
        Err(e) => return ViewerContent::Error(format!("Failed to open file: {}", e)),
    };

    let mut handle = file.take(file_size);
    let mut buffer = Vec::new();
    match handle.read_to_end(&mut buffer) {
        Ok(_) => ViewerContent::HexView(buffer, false), // Not truncated since we enforce 5MB limit
        Err(e) => ViewerContent::Error(format!("Failed to read binary file: {}", e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::io::Write;

    #[test]
    fn test_format_hex_view_empty() {
        let data: &[u8] = &[];
        let output = format_hex_view(data, false);
        
        assert!(output.contains("HEX VIEWER"));
        assert!(output.contains("Offset"));
    }

    #[test]
    fn test_format_hex_view_small_data() {
        let data: &[u8] = &[0x48, 0x65, 0x6c, 0x6c, 0x6f]; // "Hello"
        let output = format_hex_view(data, false);
        
        assert!(output.contains("48 65 6C 6C 6F") || output.contains("48 65 6c 6c 6f"));
        assert!(output.contains("Hello"));
    }

    #[test]
    fn test_format_hex_view_truncated_flag() {
        let data: &[u8] = &[0x00, 0x01, 0x02];
        let output = format_hex_view(data, true);
        
        assert!(output.contains("NOTE: File is larger than 100MB"));
    }

    #[test]
    fn test_format_hex_view_non_printable() {
        let data: &[u8] = &[0x00, 0x01, 0x02, 0x1F]; // Non-printable bytes
        let output = format_hex_view(data, false);
        
        // Non-printable should show as '.'
        assert!(output.contains("...."));
    }

    #[test]
    fn test_format_json_valid() {
        let json = r#"{"name":"test","value":42}"#;
        let formatted = format_json(json);
        
        // Pretty printed should have newlines
        assert!(formatted.contains('\n'));
        assert!(formatted.contains("name"));
        assert!(formatted.contains("test"));
    }

    #[test]
    fn test_format_json_invalid() {
        let invalid_json = "{ invalid json }";
        let formatted = format_json(invalid_json);
        
        // Should contain error message
        assert!(formatted.contains("JSON Parse Error"));
        assert!(formatted.contains("invalid json"));
    }

    #[test]
    fn test_format_json_array() {
        let json = r#"[1,2,3]"#;
        let formatted = format_json(json);
        
        assert!(formatted.contains("1"));
        assert!(formatted.contains("2"));
        assert!(formatted.contains("3"));
    }

    #[test]
    fn test_is_supported_file_type_code_files() {
        let temp = tempdir().unwrap();
        
        // Create test files
        let rs_file = temp.path().join("test.rs");
        fs::File::create(&rs_file).unwrap();
        assert!(is_supported_file_type(&rs_file));

        let py_file = temp.path().join("test.py");
        fs::File::create(&py_file).unwrap();
        assert!(is_supported_file_type(&py_file));

        let js_file = temp.path().join("test.js");
        fs::File::create(&js_file).unwrap();
        assert!(is_supported_file_type(&js_file));
    }

    #[test]
    fn test_is_supported_file_type_special_files() {
        let temp = tempdir().unwrap();
        
        let makefile = temp.path().join("Makefile");
        fs::File::create(&makefile).unwrap();
        assert!(is_supported_file_type(&makefile));

        let dockerfile = temp.path().join("Dockerfile");
        fs::File::create(&dockerfile).unwrap();
        assert!(is_supported_file_type(&dockerfile));

        let gitignore = temp.path().join(".gitignore");
        fs::File::create(&gitignore).unwrap();
        assert!(is_supported_file_type(&gitignore));
    }

    #[test]
    fn test_is_supported_file_type_image_files() {
        let temp = tempdir().unwrap();
        
        let png_file = temp.path().join("test.png");
        fs::File::create(&png_file).unwrap();
        assert!(is_supported_file_type(&png_file));

        let jpg_file = temp.path().join("test.jpg");
        fs::File::create(&jpg_file).unwrap();
        assert!(is_supported_file_type(&jpg_file));
    }

    #[test]
    fn test_is_supported_file_type_document_files() {
        let temp = tempdir().unwrap();
        
        let docx_file = temp.path().join("test.docx");
        fs::File::create(&docx_file).unwrap();
        assert!(is_supported_file_type(&docx_file));

        let xlsx_file = temp.path().join("test.xlsx");
        fs::File::create(&xlsx_file).unwrap();
        assert!(is_supported_file_type(&xlsx_file));
    }

    #[test]
    fn test_is_likely_binary_empty() {
        assert!(!is_likely_binary(""));
    }

    #[test]
    fn test_is_likely_binary_text() {
        let text = "Hello, World!\nThis is normal text.\n";
        assert!(!is_likely_binary(text));
    }

    #[test]
    fn test_is_likely_binary_with_null() {
        let binary = "Hello\0World";
        assert!(is_likely_binary(binary));
    }

    #[test]
    fn test_load_file_plain_text() {
        let temp = tempdir().unwrap();
        let txt_file = temp.path().join("test.txt");
        
        let mut file = fs::File::create(&txt_file).unwrap();
        writeln!(file, "Hello, World!").unwrap();
        
        let content = load_file(&txt_file);
        match content {
            ViewerContent::PlainText(text) => {
                assert!(text.contains("Hello, World!"));
            },
            ViewerContent::HighlightedCode { raw, .. } => {
                // .txt files may be loaded as highlighted code in some cases
                assert!(raw.contains("Hello, World!"));
            },
            _ => panic!("Expected PlainText or HighlightedCode content"),
        }
    }

    #[test]
    fn test_load_file_markdown() {
        let temp = tempdir().unwrap();
        let md_file = temp.path().join("test.md");
        
        let mut file = fs::File::create(&md_file).unwrap();
        writeln!(file, "# Heading\n\nSome text").unwrap();
        
        let content = load_file(&md_file);
        match content {
            ViewerContent::Markdown(text) => {
                assert!(text.contains("# Heading"));
            },
            _ => panic!("Expected Markdown content"),
        }
    }

    #[test]
    fn test_load_file_json() {
        let temp = tempdir().unwrap();
        let json_file = temp.path().join("test.json");
        
        let mut file = fs::File::create(&json_file).unwrap();
        writeln!(file, r#"{{"key": "value"}}"#).unwrap();
        
        let content = load_file(&json_file);
        match content {
            ViewerContent::HighlightedCode { raw, .. } => {
                assert!(raw.contains("key"));
                assert!(raw.contains("value"));
            },
            _ => panic!("Expected HighlightedCode content for JSON"),
        }
    }

    #[test]
    fn test_load_file_rust_code() {
        let temp = tempdir().unwrap();
        let rs_file = temp.path().join("test.rs");
        
        let mut file = fs::File::create(&rs_file).unwrap();
        writeln!(file, "fn main() {{ println!(\"Hello\"); }}").unwrap();
        
        let content = load_file(&rs_file);
        match content {
            ViewerContent::HighlightedCode { raw, highlighted } => {
                assert!(raw.contains("fn main"));
                assert!(!highlighted.is_empty());
            },
            _ => panic!("Expected HighlightedCode content for Rust"),
        }
    }

    #[test]
    fn test_load_file_nonexistent() {
        let path = Path::new("/nonexistent/file/path.txt");
        let content = load_file(path);
        
        match content {
            ViewerContent::Error(_) => {
                // Expected
            },
            _ => panic!("Expected Error content for nonexistent file"),
        }
    }

    #[test]
    fn test_viewer_content_clone() {
        let content = ViewerContent::PlainText("test".to_string());
        let cloned = content.clone();
        
        match cloned {
            ViewerContent::PlainText(text) => assert_eq!(text, "test"),
            _ => panic!("Clone should preserve type"),
        }
    }

    #[test]
    fn test_get_extension_for_special_file() {
        let makefile = Path::new("/path/to/Makefile");
        assert_eq!(get_extension_for_special_file(makefile), Some("make"));

        let dockerfile = Path::new("/path/to/Dockerfile");
        assert_eq!(get_extension_for_special_file(dockerfile), Some("dockerfile"));

        let bashrc = Path::new("/home/user/.bashrc");
        assert_eq!(get_extension_for_special_file(bashrc), Some("sh"));

        let regular = Path::new("/path/to/file.rs");
        assert_eq!(get_extension_for_special_file(regular), None);
    }
}
