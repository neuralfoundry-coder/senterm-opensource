//! File manager event handling

use crossterm::event::KeyCode;
use crate::app::{App, AppMode};

/// Handle file manager specific key events
pub fn handle_file_manager_keys(app: &mut App, key_code: KeyCode) {
    match key_code {
        // File operations
        KeyCode::F(2) => {
            // Rename file/folder (from active directory / PATH)
            if let Some(active_dir) = crate::navigation::get_active_directory(app.active_fs()) {
                let entries = crate::fs::FileSystem::get_entries_for_dir(&active_dir);
                let selected_index = app.active_fs_mut().get_selection(&active_dir);
                if let Some(path) = entries.get(selected_index) {
                    let current_name = path.file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string();
                    app.dialog = crate::app::DialogMode::Rename {
                        current_name: current_name.clone(),
                        new_name: current_name,
                    };
                }
            }
        },
        KeyCode::Delete => {
            // Delete file/folder (with confirmation, from active directory / PATH)
            if let Some(active_dir) = crate::navigation::get_active_directory(app.active_fs()) {
                let entries = crate::fs::FileSystem::get_entries_for_dir(&active_dir);
                let selected_index = app.active_fs_mut().get_selection(&active_dir);
                if let Some(path) = entries.get(selected_index) {
                    let path_name = path.file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string();
                    app.dialog = crate::app::DialogMode::Delete { path_name };
                }
            }
        },
        KeyCode::F(7) => {
            // New folder
            app.dialog = crate::app::DialogMode::NewFolder { name: String::new() };
        },
        KeyCode::F(8) => {
            // New file
            app.dialog = crate::app::DialogMode::NewFile { name: String::new() };
        },
        // Up/Down navigate in the active column
        KeyCode::Down => {
            if let Some(active_dir) = crate::navigation::get_active_directory(app.active_fs()) {
                tracing::info!("Down key: navigating in {:?}", active_dir);
                // Ensure selection exists for this directory (simplified check)
                if !app.active_fs_mut().column_selections.contains_key(&active_dir) {
                    app.active_fs_mut().set_selection(active_dir.clone(), 0);
                }
                app.active_fs_mut().navigate_down(&active_dir);
            }
        },
        KeyCode::Up => {
            if let Some(active_dir) = crate::navigation::get_active_directory(app.active_fs()) {
                tracing::info!("Up key: navigating in {:?}", active_dir);
                // Ensure selection exists for this directory (simplified check)
                if !app.active_fs_mut().column_selections.contains_key(&active_dir) {
                    app.active_fs_mut().set_selection(active_dir.clone(), 0);
                }
                app.active_fs_mut().navigate_up(&active_dir);
            }
        },
        // Left/Right arrow keys for column navigation only - no auto-expand
        KeyCode::Left => {
            // Only move between existing columns - do NOT auto-navigate to parent
            // Use Enter to navigate
            crate::navigation::navigate_column_backward(app.active_fs_mut());
        },
        KeyCode::Right => {
            // Only move between existing columns - do NOT auto-expand directories
            // Use Enter to expand directories
            crate::navigation::navigate_column_forward(app.active_fs_mut());
        },
        KeyCode::Enter => {
            handle_enter_key(app);
        },
        KeyCode::Backspace => app.active_fs_mut().go_back(),
        KeyCode::Char('v') | KeyCode::Char('V') => {
            // Open file in viewer (force)
            let current_dir = app.active_fs_mut().current_dir.clone();
            let entries = crate::fs::FileSystem::get_entries_for_dir(&current_dir);
            let selected_index = app.active_fs_mut().get_selection(&current_dir);
            if let Some(path) = entries.get(selected_index) {
                if path.is_file() {
                    // Check if file type is supported before opening viewer
                    if crate::viewer::is_supported_file_type(path) {
                        app.viewer_content = Some(crate::viewer::load_file(path));
                        app.viewer_scroll = 0;
                        app.mode = AppMode::Viewer;
                    } else {
                        // Show temporary message for unsupported file types
                        app.set_temp_message("미리보기가 지원되지 않는 파일 형식입니다".to_string());
                    }
                }
            }
        },
        // Standalone clipboard keys (c/x/p)
        KeyCode::Char('c') | KeyCode::Char('C') => {
            app.active_fs_mut().copy_selected();
            app.status_message = Some("Copied to clipboard".to_string());
        },
        KeyCode::Char('x') | KeyCode::Char('X') => {
            app.active_fs_mut().cut_selected();
            app.status_message = Some("Cut to clipboard".to_string());
        },
        KeyCode::Char('p') | KeyCode::Char('P') => {
            app.active_fs_mut().paste();
            app.status_message = Some("Pasted from clipboard".to_string());
            app.refresh_both_panes();
        },
        // Search mode
        KeyCode::Char('/') => {
            app.dialog = crate::app::DialogMode::Search {
                query: String::new(),
                results: Vec::new(),
            };
        },
        // Command mode (Vim-style)
        KeyCode::Char(':') => {
            app.dialog = crate::app::DialogMode::Command {
                input: String::new(),
            };
        },
        // Bookmark operations
        KeyCode::Char('b') => {
            // Add current directory to bookmarks
            let current_dir = app.active_fs_mut().current_dir.clone();
            if !app.config.bookmarks.contains(&current_dir) {
                app.config.bookmarks.push(current_dir.clone());
                let _ = app.config.save();
                app.status_message = Some("Bookmarked current directory".to_string());
            } else {
                app.status_message = Some("Already bookmarked".to_string());
            }
        },
        KeyCode::Char('B') => {
            // Toggle bookmark list
            app.show_bookmarks = !app.show_bookmarks;
        },
        // Sort option cycling
        KeyCode::Char('s') | KeyCode::Char('S') => {
            use crate::config::SortOption;
            app.active_fs_mut().sort_option = match app.active_fs_mut().sort_option {
                SortOption::Name => SortOption::Size,
                SortOption::Size => SortOption::Modified,
                SortOption::Modified => SortOption::Name,
            };
            app.config.sort_option = app.active_fs_mut().sort_option;
            let _ = app.config.save();
            let sort_name = match app.active_fs_mut().sort_option {
                SortOption::Name => "Name",
                SortOption::Size => "Size",
                SortOption::Modified => "Modified Date",
            };
            app.status_message = Some(format!("Sorting by: {}", sort_name));
        },
        _ => {}
    }
}

/// Handle Enter key in file manager
fn handle_enter_key(app: &mut App) {
    // Get the active directory (currently focused column)
    if let Some(active_dir) = crate::navigation::get_active_directory(app.active_fs()) {
        let entries = crate::fs::FileSystem::get_entries_for_dir(&active_dir);
        let selected_index = app.active_fs_mut().get_selection(&active_dir);
        
        if let Some(path) = entries.get(selected_index) {
            if path.is_file() {
                // Check if file type is supported before opening viewer
                if crate::viewer::is_supported_file_type(path) {
                    // Clear editor state and open file in viewer popup
                    app.text_editor = None;
                    app.viewer_editing = false;
                    app.viewer_content = Some(crate::viewer::load_file(path));
                    app.viewer_scroll = 0;
                    app.mode = AppMode::Viewer;
                } else {
                    // Show temporary message for unsupported file types
                    app.set_temp_message("미리보기가 지원되지 않는 파일 형식입니다".to_string());
                }
            } else if path.is_dir() {
                // Check if this is the parent entry (..)
                let is_parent = active_dir.parent()
                    .map(|p| p == path.as_path())
                    .unwrap_or(false);
                
                if is_parent {
                    handle_parent_navigation(app, &active_dir);
                } else if active_dir == app.active_fs_mut().current_dir {
                    // Entering a subdirectory from current_dir
                    app.active_fs_mut().enter_directory();
                } else {
                    // Entering a subdirectory from a non-current column
                    handle_subdirectory_entry(app, &active_dir, path);
                }
            }
        }
    }
}

/// Handle navigation to parent directory
fn handle_parent_navigation(app: &mut App, active_dir: &std::path::PathBuf) {
    // Going back to parent from active directory
    if *active_dir == app.active_fs_mut().current_dir {
        // Active dir is current_dir: use standard go_back
        app.active_fs_mut().go_back();
    } else {
        // Active dir is not current_dir: navigate to parent of active_dir
        if let Some(parent) = active_dir.parent() {
            app.active_fs_mut().current_dir = parent.to_path_buf();
            
            // Find active_dir in navigation_path and remove it and everything after
            if let Some(pos) = app.active_fs_mut().navigation_path.iter().position(|p| p == active_dir) {
                app.active_fs_mut().navigation_path.truncate(pos);
            }
            
            // Ensure parent has selection initialized
            {
                let fs = app.active_fs_mut();
                let current_dir = fs.current_dir.clone();
                if !fs.column_selections.contains_key(&current_dir) {
                    fs.column_selections.insert(current_dir, 0);
                }
            }
            
            // Focus on the parent directory column
            let col_index = app.active_fs_mut().calculate_current_dir_column_index();
            app.active_fs_mut().active_column_index = col_index;
        }
    }
}

/// Handle entering a subdirectory from a non-current column
fn handle_subdirectory_entry(app: &mut App, active_dir: &std::path::PathBuf, path: &std::path::PathBuf) {
    // Update current_dir to the selected directory and rebuild navigation
    app.active_fs_mut().current_dir = path.clone();
    
    // Find position in navigation_path and truncate
    if let Some(pos) = app.active_fs_mut().navigation_path.iter().position(|p| p == active_dir) {
        app.active_fs_mut().navigation_path.truncate(pos + 1);
    }
    app.active_fs_mut().navigation_path.push(path.clone());
    
    // Initialize selection if needed
    if !app.active_fs_mut().column_selections.contains_key(path) {
        app.active_fs_mut().column_selections.insert(path.clone(), 0);
    }
    
    // Focus on the new current directory
    app.active_fs_mut().active_column_index = app.active_fs_mut().calculate_current_dir_column_index();
}

