//! Clipboard operations event handling

use crossterm::event::{KeyCode, KeyModifiers};
use crate::app::{App, AppMode};

/// Handle copy/cut/paste operations (with modifier keys)
/// Supports both Ctrl (Linux/Windows) and Cmd (macOS) modifiers
/// Only operates in FileManager mode - other modes handle clipboard differently
pub fn handle_clipboard_operations(app: &mut App, key_code: KeyCode, modifiers: KeyModifiers) -> bool {
    let is_ctrl_or_super = modifiers.contains(KeyModifiers::CONTROL) || modifiers.contains(KeyModifiers::SUPER);

    if !is_ctrl_or_super {
        return false;
    }

    // Only handle file clipboard operations in FileManager mode
    // In Viewer mode (editing), let the vim-style y/p commands handle clipboard
    // In other modes, ignore clipboard shortcuts to prevent errors
    match app.mode {
        AppMode::FileManager => {
            // Check for both lowercase and uppercase (macOS may report uppercase with Cmd)
            match key_code {
                KeyCode::Char('c') | KeyCode::Char('C') => {
                    app.active_fs_mut().copy_selected();
                    app.status_message = Some("Copied to clipboard".to_string());
                    true
                },
                KeyCode::Char('x') | KeyCode::Char('X') => {
                    app.active_fs_mut().cut_selected();
                    app.status_message = Some("Cut to clipboard".to_string());
                    true
                },
                KeyCode::Char('v') | KeyCode::Char('V') => {
                    app.active_fs_mut().paste();
                    app.status_message = Some("Pasted from clipboard".to_string());
                    app.refresh_both_panes();
                    true
                },
                _ => false
            }
        },
        AppMode::Viewer => {
            // In viewer mode with editing, handle text clipboard operations
            if app.viewer_editing {
                if let Some(ref mut editor) = app.text_editor {
                    match key_code {
                        KeyCode::Char('c') | KeyCode::Char('C') => {
                            // Yank current line (Cmd+C in editor)
                            editor.yank_line();
                            true
                        },
                        KeyCode::Char('v') | KeyCode::Char('V') => {
                            // Paste after current line (Cmd+V in editor)
                            editor.paste_after();
                            true
                        },
                        KeyCode::Char('x') | KeyCode::Char('X') => {
                            // Cut current line (yank + delete)
                            editor.yank_line();
                            editor.delete_line();
                            true
                        },
                        _ => false
                    }
                } else {
                    false
                }
            } else {
                // In readonly viewer, ignore clipboard shortcuts
                true // Return true to consume the event and prevent errors
            }
        },
        _ => {
            // In other modes (Settings, etc.), consume but don't act
            // This prevents errors from attempting file operations
            true
        }
    }
}

