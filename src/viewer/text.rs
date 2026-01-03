use std::path::Path;
use super::ViewerContent;

/// Load DOCX file and extract text content
pub fn load_docx(path: &Path) -> ViewerContent {
    use std::fs;
    
    let bytes = match fs::read(path) {
        Ok(b) => b,
        Err(e) => return ViewerContent::Error(format!("Failed to read file: {}", e)),
    };
    
    match docx_rs::read_docx(&bytes) {
        Ok(docx) => {
            let mut text = String::new();
            // Extract text from paragraphs
            for child in &docx.document.children {
                if let docx_rs::DocumentChild::Paragraph(para) = child {
                    for child in &para.children {
                        if let docx_rs::ParagraphChild::Run(run) = child {
                            for child in &run.children {
                                if let docx_rs::RunChild::Text(t) = child {
                                    text.push_str(&t.text);
                                }
                            }
                        }
                    }
                    text.push('\n');
                }
            }
            ViewerContent::PlainText(text)
        },
        Err(e) => ViewerContent::Error(format!("Failed to parse DOCX: {}", e)),
    }
}

/// Load XLSX file and extract data as formatted text
pub fn load_xlsx(path: &Path) -> ViewerContent {
    use calamine::{open_workbook, Reader, Xlsx};
    
    match open_workbook::<Xlsx<_>, _>(path) {
        Ok(mut workbook) => {
            let mut output = String::new();
            
            for sheet_name in workbook.sheet_names() {
                output.push_str(&format!("=== Sheet: {} ===\n\n", sheet_name));
                
                if let Ok(range) = workbook.worksheet_range(&sheet_name) {
                    for row in range.rows() {
                        let row_text: Vec<String> = row.iter()
                            .map(|cell| format!("{:<15}", cell.to_string()))
                            .collect();
                        output.push_str(&row_text.join(" | "));
                        output.push('\n');
                    }
                }
                output.push('\n');
            }
            
            ViewerContent::PlainText(output)
        },
        Err(e) => ViewerContent::Error(format!("Failed to parse XLSX: {}", e)),
    }
}
