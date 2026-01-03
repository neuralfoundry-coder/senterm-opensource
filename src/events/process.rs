//! Process viewer event handling

use crossterm::event::KeyCode;
use crate::app::App;

/// Handle process viewer key events
/// Returns true if the key was handled
pub fn handle_process_viewer_keys(app: &mut App, key_code: KeyCode) -> bool {
    let viewer = &mut app.process_viewer;
    
    // Handle search mode separately
    if viewer.search_mode {
        match key_code {
            KeyCode::Esc => {
                viewer.search_mode = false;
                true
            },
            KeyCode::Enter => {
                viewer.search_mode = false;
                true
            },
            KeyCode::Backspace => {
                viewer.search_query.pop();
                let query = viewer.search_query.clone();
                viewer.set_search(query);
                true
            },
            KeyCode::Char(c) => {
                viewer.search_query.push(c);
                let query = viewer.search_query.clone();
                viewer.set_search(query);
                true
            },
            _ => false
        }
    } else {
        match key_code {
            // Close viewer
            KeyCode::Esc | KeyCode::F(9) => {
                app.show_process_viewer = false;
                true
            },
            
            // Navigation
            KeyCode::Up | KeyCode::Char('k') => {
                viewer.move_up();
                true
            },
            KeyCode::Down | KeyCode::Char('j') => {
                viewer.move_down();
                true
            },
            KeyCode::Home | KeyCode::Char('g') => {
                viewer.move_to_top();
                true
            },
            KeyCode::End | KeyCode::Char('G') => {
                viewer.move_to_bottom();
                true
            },
            KeyCode::PageUp => {
                viewer.page_up(10);
                true
            },
            KeyCode::PageDown => {
                viewer.page_down(10);
                true
            },
            
            // Tree operations
            KeyCode::Char('t') | KeyCode::Enter => {
                viewer.toggle_expand();
                true
            },
            KeyCode::Char('p') => {
                viewer.move_to_parent();
                true
            },
            
            // Kill process (Shift+K for force kill)
            KeyCode::Char('K') => {
                match viewer.kill_selected(true) {
                    Ok(_) => {
                        app.set_temp_message("Process killed (SIGKILL)".to_string());
                    },
                    Err(e) => {
                        app.set_temp_message(format!("Kill failed: {}", e));
                    }
                }
                true
            },
            KeyCode::Delete => {
                match viewer.kill_selected(false) {
                    Ok(_) => {
                        app.set_temp_message("Process terminated (SIGTERM)".to_string());
                    },
                    Err(e) => {
                        app.set_temp_message(format!("Kill failed: {}", e));
                    }
                }
                true
            },
            
            // Filter and sort
            KeyCode::Char('f') => {
                viewer.cycle_filter();
                true
            },
            KeyCode::Char('s') => {
                viewer.cycle_sort();
                true
            },
            KeyCode::Char('S') => {
                viewer.toggle_sort_order();
                true
            },
            
            // Search
            KeyCode::Char('/') => {
                viewer.search_mode = true;
                viewer.search_query.clear();
                true
            },
            
            // Refresh
            KeyCode::Char('r') => {
                viewer.refresh();
                app.set_temp_message("Process list refreshed".to_string());
                true
            },
            
            // Info (show details toggle - already shown by default)
            KeyCode::Char('i') => {
                viewer.show_details = !viewer.show_details;
                true
            },
            
            _ => false
        }
    }
}

