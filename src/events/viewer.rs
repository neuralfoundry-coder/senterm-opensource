//! Viewer mode event handling

use crossterm::event::{KeyCode, KeyModifiers};
use crate::app::{App, AppMode};
use super::utils::{copy_to_system_clipboard, paste_from_system_clipboard, get_viewer_content_text};
use super::editor::{handle_editor_keys, handle_nano_keys};

/// Handle viewer mode key events
pub fn handle_viewer_keys(app: &mut App, key_code: KeyCode, modifiers: KeyModifiers) {
    use crate::viewer::editor::EditorStyle;
    
    // Check if in editing mode
    if app.viewer_editing {
        // Ctrl+T: Toggle between Vim and Nano modes
        if modifiers.contains(KeyModifiers::CONTROL) || modifiers.contains(KeyModifiers::SUPER) {
            match key_code {
                KeyCode::Char('t') | KeyCode::Char('T') => {
                    if let Some(ref mut editor) = app.text_editor {
                        editor.toggle_editor_style();
                        return;
                    }
                },
                // Ctrl+C: Copy selection to system clipboard (in editor mode)
                KeyCode::Char('c') | KeyCode::Char('C') => {
                    if let Some(ref mut editor) = app.text_editor {
                        let copied_text = editor.get_clipboard_text();
                        if !copied_text.is_empty() {
                            match copy_to_system_clipboard(&copied_text) {
                                Ok(_) => {
                                    editor.status_message = format!("Copied {} chars to clipboard", copied_text.len());
                                },
                                Err(e) => {
                                    editor.status_message = format!("Copy failed: {}", e);
                                }
                            }
                        } else {
                            // Copy all if nothing selected
                            let all_text = editor.lines.join("\n");
                            match copy_to_system_clipboard(&all_text) {
                                Ok(_) => {
                                    editor.status_message = format!("Copied all ({} chars)", all_text.len());
                                },
                                Err(e) => {
                                    editor.status_message = format!("Copy failed: {}", e);
                                }
                            }
                        }
                        return;
                    }
                },
                // Ctrl+V: Paste from system clipboard
                KeyCode::Char('v') | KeyCode::Char('V') => {
                    if let Some(ref mut editor) = app.text_editor {
                        match paste_from_system_clipboard() {
                            Ok(text) => {
                                if !text.is_empty() {
                                    editor.paste_text(&text);
                                    editor.status_message = format!("Pasted {} chars", text.len());
                                }
                            },
                            Err(e) => {
                                editor.status_message = format!("Paste failed: {}", e);
                            }
                        }
                        return;
                    }
                },
                _ => {}
            }
        }
        
        // Check editor style
        let editor_style = app.text_editor.as_ref().map(|e| e.editor_style).unwrap_or(EditorStyle::Vim);
        
        match editor_style {
            EditorStyle::Nano => {
                handle_nano_keys(app, key_code, modifiers);
            },
            EditorStyle::Vim => {
                // Handle Ctrl combinations in Vim editor
                if modifiers.contains(KeyModifiers::CONTROL) {
                    if let Some(ref mut editor) = app.text_editor {
                        match key_code {
                            KeyCode::Char('r') | KeyCode::Char('R') => {
                                editor.redo();
                                return;
                            },
                            KeyCode::Char('d') | KeyCode::Char('D') => {
                                editor.move_half_page_down();
                                return;
                            },
                            KeyCode::Char('u') | KeyCode::Char('U') => {
                                editor.move_half_page_up();
                                return;
                            },
                            KeyCode::Char('f') | KeyCode::Char('F') => {
                                editor.move_page_down();
                                return;
                            },
                            KeyCode::Char('b') | KeyCode::Char('B') => {
                                editor.move_page_up();
                                return;
                            },
                            KeyCode::Char('o') | KeyCode::Char('O') => {
                                editor.move_to_first_line();
                                return;
                            },
                            _ => {}
                        }
                    }
                }
                handle_editor_keys(app, key_code);
            },
        }
    } else {
        // Normal viewing mode
        handle_readonly_viewer(app, key_code, modifiers);
    }
}

/// Handle readonly viewer mode
fn handle_readonly_viewer(app: &mut App, key_code: KeyCode, modifiers: KeyModifiers) {
    // Handle Ctrl+C: Copy all content to system clipboard
    if (modifiers.contains(KeyModifiers::CONTROL) || modifiers.contains(KeyModifiers::SUPER)) 
        && matches!(key_code, KeyCode::Char('c') | KeyCode::Char('C')) {
        if let Some(content) = get_viewer_content_text(app) {
            match copy_to_system_clipboard(&content) {
                Ok(_) => {
                    app.status_message = Some(format!("Copied {} chars to clipboard", content.len()));
                },
                Err(e) => {
                    app.status_message = Some(format!("Copy failed: {}", e));
                }
            }
        } else {
            app.status_message = Some("Cannot copy this content type".to_string());
        }
        return;
    }
    
    // Calculate total lines for scroll bounds
    let total_lines = get_viewer_total_lines(app);
    let half_page = 15usize;
    let full_page = 30usize;
    
    match key_code {
        KeyCode::Char('i') => {
            // Enter edit mode
            enter_edit_mode(app);
        },
        // Single line navigation
        KeyCode::Down | KeyCode::Char('j') => {
            app.viewer_scroll = app.viewer_scroll.saturating_add(1);
        },
        KeyCode::Up | KeyCode::Char('k') => {
            app.viewer_scroll = app.viewer_scroll.saturating_sub(1);
        },
        // Page navigation (10 lines)
        KeyCode::PageDown => {
            app.viewer_scroll = app.viewer_scroll.saturating_add(10);
        },
        KeyCode::PageUp => {
            app.viewer_scroll = app.viewer_scroll.saturating_sub(10);
        },
        // Top/Bottom navigation (gg/G style)
        KeyCode::Char('g') | KeyCode::Home => {
            app.viewer_scroll = 0;
        },
        KeyCode::Char('G') | KeyCode::End => {
            // Move to bottom (leave some visible lines)
            app.viewer_scroll = total_lines.saturating_sub(1);
        },
        // Half page navigation (d/u style)
        KeyCode::Char('d') => {
            app.viewer_scroll = app.viewer_scroll.saturating_add(half_page);
        },
        KeyCode::Char('u') => {
            app.viewer_scroll = app.viewer_scroll.saturating_sub(half_page);
        },
        // Full page navigation (Space/b style)
        KeyCode::Char(' ') => {
            app.viewer_scroll = app.viewer_scroll.saturating_add(full_page);
        },
        KeyCode::Char('b') => {
            app.viewer_scroll = app.viewer_scroll.saturating_sub(full_page);
        },
        // Toggle wrap mode
        KeyCode::Char('w') => {
            app.viewer_wrap_mode = !app.viewer_wrap_mode;
            app.viewer_scroll = 0; // Reset scroll when toggling wrap
            let mode = if app.viewer_wrap_mode { "ON" } else { "OFF" };
            app.status_message = Some(format!("Line wrap: {}", mode));
        },
        _ => {}
    }
    
    // Clamp scroll to valid range
    if total_lines > 0 {
        app.viewer_scroll = app.viewer_scroll.min(total_lines.saturating_sub(1));
    }
}

/// Get total line count from viewer content
pub fn get_viewer_total_lines(app: &App) -> usize {
    match &app.viewer_content {
        Some(crate::viewer::ViewerContent::PlainText(s)) => s.lines().count(),
        Some(crate::viewer::ViewerContent::HighlightedCode { highlighted, .. }) => highlighted.len(),
        Some(crate::viewer::ViewerContent::Markdown(s)) => s.lines().count(),
        Some(crate::viewer::ViewerContent::HexView(data, truncated)) => {
            // Hex view has header lines + data lines (16 bytes per line)
            let header_lines = if *truncated { 7 } else { 6 };
            header_lines + (data.len() + 15) / 16 + 1 // +1 for footer
        },
        Some(crate::viewer::ViewerContent::Image(_)) => 10, // Image info display
        Some(crate::viewer::ViewerContent::ImagePreviewContent(preview)) => {
            // Count lines in rendered preview + metadata
            preview.content.lines().count() + 5
        },
        Some(crate::viewer::ViewerContent::Error(_)) => 1,
        None => 0,
    }
}

/// Enter vim edit mode
pub fn enter_edit_mode(app: &mut App) {
    if let Some(content) = &app.viewer_content {
        let text = match content {
            crate::viewer::ViewerContent::PlainText(s) => s.clone(),
            crate::viewer::ViewerContent::HighlightedCode { raw, .. } => raw.clone(),
            crate::viewer::ViewerContent::Markdown(s) => s.clone(),
            crate::viewer::ViewerContent::Image(_) | 
            crate::viewer::ViewerContent::ImagePreviewContent(_) => {
                app.status_message = Some("Cannot edit image files".to_string());
                return;
            },
            crate::viewer::ViewerContent::HexView(_, _) => {
                app.status_message = Some("Cannot edit binary files".to_string());
                return;
            },
            crate::viewer::ViewerContent::Error(_) => {
                app.status_message = Some("Cannot edit error message".to_string());
                return;
            }
        };

        // Get file path from current selection
        let file_path = if let Some(active_dir) = crate::navigation::get_active_directory(app.active_fs()) {
            let entries = crate::fs::FileSystem::get_entries_for_dir(&active_dir);
            let selected_index = app.active_fs_mut().get_selection(&active_dir);
            entries.get(selected_index).cloned()
        } else {
            None
        };

        app.text_editor = Some(crate::viewer::TextEditor::new(text, file_path));
        app.viewer_editing = true;
        app.status_message = Some("Entered edit mode - ESC for normal, i for insert".to_string());
    }
}

/// Exit editor and return to file manager mode
pub fn exit_editor(app: &mut App, saved: bool) {
    app.viewer_editing = false;
    app.text_editor = None;
    app.viewer_content = None;
    app.viewer_scroll = 0;
    app.mode = AppMode::FileManager;
    app.status_message = Some(if saved { "Saved and exited".to_string() } else { "Exited".to_string() });
}

/// Save file to disk
pub fn save_file(app: &mut App) -> bool {
    use std::fs;
    
    let editor = app.text_editor.as_mut().unwrap();
    
    if let Some(file_path) = &editor.file_path {
        let content = editor.get_content();
        match fs::write(file_path, content) {
            Ok(_) => {
                editor.modified = false;
                true
            },
            Err(e) => {
                tracing::error!("Failed to save file: {}", e);
                false
            }
        }
    } else {
        editor.status_message = "Error: No file path".to_string();
        false
    }
}

