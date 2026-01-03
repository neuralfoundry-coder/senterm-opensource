//! Dialog mode event handling

use crossterm::event::KeyCode;
use std::path::PathBuf;
use crate::app::{App, DialogMode};

/// Handle dialog mode key events
/// Returns true if a key was handled
pub fn handle_dialog_keys(app: &mut App, key_code: KeyCode) -> bool {
    // Pre-fetch values that might be needed during dialog handling
    let search_dir = app.active_fs().current_dir.clone();
    
    // Clone dialog data to avoid borrow issues
    let dialog_clone = std::mem::replace(&mut app.dialog, DialogMode::None);
    
    let result = match dialog_clone {
        DialogMode::None => {
            app.dialog = DialogMode::None;
            false
        },
        DialogMode::Rename { current_name, new_name } => {
            app.dialog = DialogMode::Rename { current_name, new_name };
            handle_rename_dialog(app, key_code)
        },
        DialogMode::Delete { path_name } => {
            app.dialog = DialogMode::Delete { path_name };
            handle_delete_dialog(app, key_code)
        },
        DialogMode::NewFile { name } => {
            app.dialog = DialogMode::NewFile { name };
            handle_new_file_dialog(app, key_code)
        },
        DialogMode::NewFolder { name } => {
            app.dialog = DialogMode::NewFolder { name };
            handle_new_folder_dialog(app, key_code)
        },
        DialogMode::Search { query, results } => {
            app.dialog = DialogMode::Search { query, results };
            handle_search_dialog(app, key_code, &search_dir)
        },
        DialogMode::Command { input } => {
            app.dialog = DialogMode::Command { input };
            handle_command_dialog(app, key_code)
        },
        DialogMode::QuitConfirm => {
            app.dialog = DialogMode::QuitConfirm;
            handle_quit_confirm_dialog(app, key_code)
        }
    };
    
    result
}

fn handle_rename_dialog(app: &mut App, key_code: KeyCode) -> bool {
    match key_code {
        KeyCode::Char(c) => {
            if let DialogMode::Rename { ref mut new_name, .. } = app.dialog {
                new_name.push(c);
            }
        },
        KeyCode::Backspace => {
            if let DialogMode::Rename { ref mut new_name, .. } = app.dialog {
                new_name.pop();
            }
        },
        KeyCode::Enter => {
            let name_to_use = if let DialogMode::Rename { ref new_name, .. } = app.dialog {
                new_name.clone()
            } else {
                return true;
            };
            app.dialog = DialogMode::None;

            match app.active_fs_mut().rename_selected(&name_to_use) {
                Ok(_) => {
                    app.status_message = Some(format!("Renamed to '{}'", name_to_use));
                    app.refresh_both_panes();
                },
                Err(e) => {
                    app.status_message = Some(format!("Rename failed: {}", e));
                }
            }
        },
        KeyCode::Esc => {
            app.dialog = DialogMode::None;
        },
        _ => {} // Ignore all other keys (including arrow keys) to prevent confusion
    }
    true // Always consume key events when dialog is active
}

fn handle_delete_dialog(app: &mut App, key_code: KeyCode) -> bool {
    match key_code {
        KeyCode::Char('y') | KeyCode::Char('Y') => {
            let path_name = if let DialogMode::Delete { ref path_name } = app.dialog {
                path_name.clone()
            } else {
                return true;
            };
            app.dialog = DialogMode::None;

            match app.active_fs_mut().delete_selected() {
                Ok(_) => {
                    app.status_message = Some(format!("Deleted '{}'", path_name));
                    app.refresh_both_panes();
                },
                Err(e) => {
                    app.status_message = Some(format!("Delete failed: {}", e));
                }
            }
        },
        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
            app.dialog = DialogMode::None;
        },
        _ => {} // Ignore all other keys
    }
    true // Always consume key events when dialog is active
}

fn handle_new_file_dialog(app: &mut App, key_code: KeyCode) -> bool {
    match key_code {
        KeyCode::Char(c) => {
            if let DialogMode::NewFile { ref mut name } = app.dialog {
                name.push(c);
            }
        },
        KeyCode::Backspace => {
            if let DialogMode::NewFile { ref mut name } = app.dialog {
                name.pop();
            }
        },
        KeyCode::Enter => {
            let file_name = if let DialogMode::NewFile { ref name } = app.dialog {
                name.clone()
            } else {
                return true;
            };
            app.dialog = DialogMode::None;

            if !file_name.is_empty() {
                match app.active_fs_mut().create_file(&file_name) {
                    Ok(_) => {
                        app.status_message = Some(format!("Created file '{}'", file_name));
                        // Clear viewer content to avoid showing old file content
                        app.viewer_content = None;
                        app.text_editor = None;
                        app.viewer_editing = false;
                        app.refresh_both_panes();
                    },
                    Err(e) => {
                        app.status_message = Some(format!("Create file failed: {}", e));
                    }
                }
            }
        },
        KeyCode::Esc => {
            app.dialog = DialogMode::None;
        },
        _ => {} // Ignore all other keys
    }
    true // Always consume key events when dialog is active
}

fn handle_new_folder_dialog(app: &mut App, key_code: KeyCode) -> bool {
    match key_code {
        KeyCode::Char(c) => {
            if let DialogMode::NewFolder { ref mut name } = app.dialog {
                name.push(c);
            }
        },
        KeyCode::Backspace => {
            if let DialogMode::NewFolder { ref mut name } = app.dialog {
                name.pop();
            }
        },
        KeyCode::Enter => {
            let folder_name = if let DialogMode::NewFolder { ref name } = app.dialog {
                name.clone()
            } else {
                return true;
            };
            app.dialog = DialogMode::None;

            if !folder_name.is_empty() {
                match app.active_fs_mut().create_folder(&folder_name) {
                    Ok(_) => {
                        app.status_message = Some(format!("Created folder '{}'", folder_name));
                        app.refresh_both_panes();
                    },
                    Err(e) => {
                        app.status_message = Some(format!("Create folder failed: {}", e));
                    }
                }
            }
        },
        KeyCode::Esc => {
            app.dialog = DialogMode::None;
        },
        _ => {} // Ignore all other keys
    }
    true // Always consume key events when dialog is active
}

fn handle_search_dialog(app: &mut App, key_code: KeyCode, search_dir: &PathBuf) -> bool {
    match key_code {
        KeyCode::Char(c) => {
            if let DialogMode::Search { ref mut query, ref mut results } = app.dialog {
                query.push(c);
                // Update search results
                *results = perform_search(search_dir, query);
            }
        },
        KeyCode::Backspace => {
            if let DialogMode::Search { ref mut query, ref mut results } = app.dialog {
                query.pop();
                // Update search results
                *results = perform_search(search_dir, query);
            }
        },
        KeyCode::Enter => {
            let (first_result, result_count) = if let DialogMode::Search { ref results, .. } = app.dialog {
                (results.first().cloned(), results.len())
            } else {
                return true;
            };
            
            if let Some((path, _)) = first_result {
                if let Some(parent) = path.parent() {
                    let fs = app.active_fs_mut();
                    fs.current_dir = parent.to_path_buf();
                    let current_dir = fs.current_dir.clone();
                    // Find and select the file in the current directory
                    let entries = crate::fs::FileSystem::get_entries_for_dir(&current_dir);
                    if let Some(idx) = entries.iter().position(|p| p == &path) {
                        fs.set_selection(current_dir, idx);
                    }
                }
                app.status_message = Some(format!("Found {} result(s)", result_count));
            } else {
                app.status_message = Some("No results found".to_string());
            }
            app.dialog = DialogMode::None;
        },
        KeyCode::Esc => {
            app.dialog = DialogMode::None;
        },
        _ => {} // Ignore all other keys
    }
    true // Always consume key events when dialog is active
}

fn handle_command_dialog(app: &mut App, key_code: KeyCode) -> bool {
    match key_code {
        KeyCode::Char(c) => {
            if let DialogMode::Command { ref mut input } = app.dialog {
                input.push(c);
            }
        },
        KeyCode::Backspace => {
            if let DialogMode::Command { ref mut input } = app.dialog {
                input.pop();
            }
        },
        KeyCode::Enter => {
            let command = if let DialogMode::Command { ref input } = app.dialog {
                input.trim().to_lowercase()
            } else {
                return true;
            };
            app.dialog = DialogMode::None;

            // Parse and execute command
            match command.as_str() {
                "game" => {
                    tracing::info!("Launching senterm-games via :game command");
                    app.launch_external_game = true;
                    app.status_message = Some("Launching senterm-games...".to_string());
                },
                "help" => {
                    app.show_help = true;
                    app.status_message = Some("Showing help".to_string());
                },
                "quit" | "q" => {
                    app.should_quit = true;
                },
                _ => {
                    app.status_message = Some(format!("Unknown command: {}", command));
                }
            }
        },
        KeyCode::Esc => {
            app.dialog = DialogMode::None;
        },
        _ => {} // Ignore all other keys
    }
    true // Always consume key events when dialog is active
}

fn handle_quit_confirm_dialog(app: &mut App, key_code: KeyCode) -> bool {
    match key_code {
        KeyCode::Char('y') | KeyCode::Char('Y') => {
            app.dialog = DialogMode::None;
            app.should_quit = true;
        },
        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
            app.dialog = DialogMode::None;
        },
        _ => {} // Ignore all other keys
    }
    true // Always consume key events when dialog is active
}

/// Perform recursive file search in directory
fn perform_search(dir: &PathBuf, query: &str) -> Vec<(PathBuf, usize)> {
    use std::fs;

    if query.is_empty() {
        return Vec::new();
    }

    let mut results = Vec::new();
    let query_lower = query.to_lowercase();

    // Search recursively
    fn search_recursive(
        dir: &PathBuf,
        query: &str,
        results: &mut Vec<(PathBuf, usize)>,
        depth: usize,
    ) {
        // Limit recursion depth to avoid performance issues
        if depth > 5 {
            return;
        }

        if let Ok(entries) = fs::read_dir(dir) {
            for (idx, entry) in entries.flatten().enumerate() {
                let path = entry.path();
                if let Some(name) = path.file_name() {
                    if name.to_string_lossy().to_lowercase().contains(query) {
                        results.push((path.clone(), idx));
                    }
                }

                // Recurse into subdirectories
                if path.is_dir() && results.len() < 50 {
                    search_recursive(&path, query, results, depth + 1);
                }
            }
        }
    }

    search_recursive(dir, &query_lower, &mut results, 0);
    results.truncate(50); // Limit to 50 results
    results
}
