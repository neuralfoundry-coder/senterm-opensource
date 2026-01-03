//! Vim/Nano editor event handling

use crossterm::event::{KeyCode, KeyModifiers};
use crate::app::App;
use crate::viewer::VimMode;
use crate::viewer::editor::PendingOperator;
use super::viewer::{exit_editor, save_file};

/// Handle vim editor key events
pub fn handle_editor_keys(app: &mut App, key_code: KeyCode) {
    let editor = match &mut app.text_editor {
        Some(ed) => ed,
        None => return,
    };

    match editor.mode {
        VimMode::Normal => handle_normal_mode_keys(app, key_code),
        VimMode::Insert => handle_insert_mode_keys(app, key_code),
        VimMode::Command => handle_command_mode_keys(app, key_code),
        VimMode::Visual | VimMode::VisualLine => handle_visual_mode_keys(app, key_code),
    }
}

/// Handle Visual mode keys
fn handle_visual_mode_keys(app: &mut App, key_code: KeyCode) {
    let editor = app.text_editor.as_mut().unwrap();
    
    match key_code {
        // Exit visual mode
        KeyCode::Esc | KeyCode::Char('v') | KeyCode::Char('V') => {
            editor.enter_normal_mode();
        },
        
        // Movement (extends selection)
        KeyCode::Char('h') | KeyCode::Left => editor.move_cursor_left(),
        KeyCode::Char('j') | KeyCode::Down => editor.move_cursor_down(),
        KeyCode::Char('k') | KeyCode::Up => editor.move_cursor_up(),
        KeyCode::Char('l') | KeyCode::Right => editor.move_cursor_right(),
        KeyCode::Char('w') => editor.move_word_forward(),
        KeyCode::Char('b') => editor.move_word_backward(),
        KeyCode::Char('e') => editor.move_word_end(),
        KeyCode::Char('0') | KeyCode::Home => editor.move_to_line_start(),
        KeyCode::Char('$') | KeyCode::End => editor.move_to_line_end(),
        KeyCode::Char('^') => editor.move_to_first_nonblank(),
        KeyCode::Char('G') => editor.move_to_last_line(),
        KeyCode::Char('g') => editor.move_to_first_line(),
        
        // Operations on selection
        KeyCode::Char('d') | KeyCode::Char('x') => {
            editor.delete_visual_selection();
        },
        KeyCode::Char('y') => {
            // Get selection text before yanking (for system clipboard)
            let selection_text = editor.get_visual_selection_text();
            editor.yank_visual_selection();
            // Also copy to system clipboard
            if !selection_text.is_empty() {
                if let Ok(mut clipboard) = arboard::Clipboard::new() {
                    let _ = clipboard.set_text(&selection_text);
                }
            }
        },
        KeyCode::Char('c') | KeyCode::Char('s') => {
            editor.delete_visual_selection();
            let ed = app.text_editor.as_mut().unwrap();
            ed.enter_insert_mode();
        },
        
        // Indentation
        KeyCode::Char('>') => {
            let (sr, _, er, _) = editor.get_visual_selection();
            editor.save_undo();
            for row in sr..=er {
                editor.lines[row].insert_str(0, "    ");
            }
            editor.modified = true;
            editor.enter_normal_mode();
        },
        KeyCode::Char('<') => {
            let (sr, _, er, _) = editor.get_visual_selection();
            editor.save_undo();
            for row in sr..=er {
                let line = &mut editor.lines[row];
                let mut removed = 0;
                while removed < 4 && line.starts_with(' ') {
                    line.remove(0);
                    removed += 1;
                }
            }
            editor.modified = true;
            editor.enter_normal_mode();
        },
        
        // Case toggle
        KeyCode::Char('~') | KeyCode::Char('u') | KeyCode::Char('U') => {
            let (sr, sc, er, ec) = editor.get_visual_selection();
            editor.save_undo();
            
            for row in sr..=er {
                let line = &editor.lines[row];
                let chars: Vec<char> = line.chars().collect();
                let start = if row == sr { sc } else { 0 };
                let end = if row == er { ec + 1 } else { chars.len() };
                
                let new_line: String = chars.iter().enumerate().map(|(i, &c)| {
                    if i >= start && i < end {
                        match key_code {
                            KeyCode::Char('~') => {
                                if c.is_lowercase() { c.to_uppercase().next().unwrap_or(c) }
                                else { c.to_lowercase().next().unwrap_or(c) }
                            },
                            KeyCode::Char('u') => c.to_lowercase().next().unwrap_or(c),
                            KeyCode::Char('U') => c.to_uppercase().next().unwrap_or(c),
                            _ => c
                        }
                    } else {
                        c
                    }
                }).collect();
                editor.lines[row] = new_line;
            }
            editor.modified = true;
            editor.enter_normal_mode();
        },
        
        // Join selected lines
        KeyCode::Char('J') => {
            let (sr, _, er, _) = editor.get_visual_selection();
            editor.cursor_row = sr;
            for _ in sr..er {
                editor.join_lines();
            }
            editor.enter_normal_mode();
        },
        
        _ => {}
    }
}

/// Handle Normal mode keys
fn handle_normal_mode_keys(app: &mut App, key_code: KeyCode) {
    let editor = app.text_editor.as_mut().unwrap();
    
    // Handle count prefix (1-9 for first digit, 0-9 for subsequent)
    if let KeyCode::Char(c) = key_code {
        if c.is_ascii_digit() {
            if c != '0' || !editor.count_buffer.is_empty() {
                editor.count_buffer.push(c);
                editor.status_message = format!("{}", editor.count_buffer);
                return;
            }
        }
    }
    
    // Handle pending operator
    if editor.pending_op != PendingOperator::None {
        let count = editor.get_count();
        match (editor.pending_op, key_code) {
            // dd - delete line(s)
            (PendingOperator::Delete, KeyCode::Char('d')) => {
                editor.delete_lines(count);
                editor.pending_op = PendingOperator::None;
                return;
            },
            // yy - yank line(s)
            (PendingOperator::Yank, KeyCode::Char('y')) => {
                editor.yank_lines(count);
                editor.pending_op = PendingOperator::None;
                return;
            },
            // cc - change line(s)
            (PendingOperator::Change, KeyCode::Char('c')) => {
                editor.substitute_line();
                editor.pending_op = PendingOperator::None;
                return;
            },
            // >> - indent
            (PendingOperator::Indent, KeyCode::Char('>')) => {
                for _ in 0..count {
                    editor.indent_line();
                }
                editor.pending_op = PendingOperator::None;
                return;
            },
            // << - outdent
            (PendingOperator::Outdent, KeyCode::Char('<')) => {
                for _ in 0..count {
                    editor.outdent_line();
                }
                editor.pending_op = PendingOperator::None;
                return;
            },
            // d$ or D - delete to end
            (PendingOperator::Delete, KeyCode::Char('$')) => {
                editor.delete_to_end();
                editor.pending_op = PendingOperator::None;
                return;
            },
            // y$ - yank to end
            (PendingOperator::Yank, KeyCode::Char('$')) => {
                editor.yank_to_end();
                editor.pending_op = PendingOperator::None;
                return;
            },
            // c$ - change to end
            (PendingOperator::Change, KeyCode::Char('$')) => {
                editor.change_to_end();
                editor.pending_op = PendingOperator::None;
                return;
            },
            // dw - delete word
            (PendingOperator::Delete, KeyCode::Char('w')) => {
                for _ in 0..count {
                    editor.delete_word();
                }
                editor.pending_op = PendingOperator::None;
                return;
            },
            // dG - delete to end of file
            (PendingOperator::Delete, KeyCode::Char('G')) => {
                let start = editor.cursor_row;
                let end = editor.lines.len();
                editor.clipboard = editor.lines.drain(start..end).collect();
                editor.clipboard_is_line = true;
                if editor.lines.is_empty() {
                    editor.lines.push(String::new());
                }
                editor.cursor_row = editor.cursor_row.min(editor.lines.len().saturating_sub(1));
                editor.modified = true;
                editor.pending_op = PendingOperator::None;
                return;
            },
            // dgg - delete to beginning of file
            (PendingOperator::Delete, KeyCode::Char('g')) => {
                let end = editor.cursor_row + 1;
                editor.clipboard = editor.lines.drain(0..end).collect();
                editor.clipboard_is_line = true;
                if editor.lines.is_empty() {
                    editor.lines.push(String::new());
                }
                editor.cursor_row = 0;
                editor.cursor_col = 0;
                editor.modified = true;
                editor.pending_op = PendingOperator::None;
                return;
            },
            // gg - go to first line (when pending is not set, but we handle 'g' followed by 'g')
            _ => {
                editor.pending_op = PendingOperator::None;
            }
        }
    }
    
    let count = editor.get_count();
    
    match key_code {
        // Movement
        KeyCode::Char('h') | KeyCode::Left => {
            for _ in 0..count {
                editor.move_cursor_left();
            }
        },
        KeyCode::Char('j') | KeyCode::Down => {
            for _ in 0..count {
                editor.move_cursor_down();
            }
        },
        KeyCode::Char('k') | KeyCode::Up => {
            for _ in 0..count {
                editor.move_cursor_up();
            }
        },
        KeyCode::Char('l') | KeyCode::Right => {
            for _ in 0..count {
                editor.move_cursor_right();
            }
        },
        KeyCode::Char('0') => editor.move_to_line_start(),
        KeyCode::Char('^') => editor.move_to_first_nonblank(),
        KeyCode::Char('$') | KeyCode::End => editor.move_to_line_end(),
        KeyCode::Home => editor.move_to_line_start(),
        
        // Word motions
        KeyCode::Char('w') => {
            for _ in 0..count {
                editor.move_word_forward();
            }
        },
        KeyCode::Char('b') => {
            for _ in 0..count {
                editor.move_word_backward();
            }
        },
        KeyCode::Char('e') => {
            for _ in 0..count {
                editor.move_word_end();
            }
        },
        
        // Line/file navigation
        KeyCode::Char('g') => {
            // Set up for 'gg' or go to line with count
            if count > 1 {
                editor.move_to_line(count);
            } else {
                // Wait for second 'g'
                editor.pending_op = PendingOperator::Delete; // Reuse for 'gg' detection
                editor.status_message = "g".to_string();
                // Actually, let's handle gg specially - just go to first line for single g
                editor.move_to_first_line();
            }
        },
        KeyCode::Char('G') => {
            if count > 1 {
                editor.move_to_line(count);
            } else {
                editor.move_to_last_line();
            }
        },
        
        // Bracket matching
        KeyCode::Char('%') => editor.move_to_matching_bracket(),
        
        // Edit commands
        KeyCode::Char('x') => {
            for _ in 0..count {
                editor.delete_char();
            }
        },
        KeyCode::Char('X') => {
            for _ in 0..count {
                editor.delete_char_before();
            }
        },
        KeyCode::Char('d') => {
            editor.pending_op = PendingOperator::Delete;
            editor.status_message = "d".to_string();
        },
        KeyCode::Char('D') => {
            editor.delete_to_end();
        },
        KeyCode::Char('y') => {
            editor.pending_op = PendingOperator::Yank;
            editor.status_message = "y".to_string();
        },
        KeyCode::Char('Y') => {
            editor.yank_line();
        },
        KeyCode::Char('c') => {
            editor.pending_op = PendingOperator::Change;
            editor.status_message = "c".to_string();
        },
        KeyCode::Char('C') => {
            editor.change_to_end();
        },
        KeyCode::Char('p') => editor.paste_after(),
        KeyCode::Char('P') => editor.paste_before(),
        
        // Single char operations
        KeyCode::Char('r') => {
            // Replace mode - wait for next char
            editor.status_message = "r".to_string();
            editor.mode = VimMode::Command;
            editor.command_buffer = "r".to_string();
        },
        KeyCode::Char('s') => editor.substitute_char(),
        KeyCode::Char('S') => editor.substitute_line(),
        
        // Undo/Redo
        KeyCode::Char('u') => editor.undo(),
        
        // Join lines
        KeyCode::Char('J') => {
            for _ in 0..count {
                editor.join_lines();
            }
        },
        
        // Indentation
        KeyCode::Char('>') => {
            editor.pending_op = PendingOperator::Indent;
            editor.status_message = ">".to_string();
        },
        KeyCode::Char('<') => {
            editor.pending_op = PendingOperator::Outdent;
            editor.status_message = "<".to_string();
        },
        
        // Case toggle
        KeyCode::Char('~') => {
            for _ in 0..count {
                editor.toggle_case();
            }
        },
        
        // Search
        KeyCode::Char('/') => editor.start_search_forward(),
        KeyCode::Char('?') => editor.start_search_backward(),
        KeyCode::Char('n') => {
            for _ in 0..count {
                editor.search_next();
            }
        },
        KeyCode::Char('N') => {
            for _ in 0..count {
                editor.search_prev();
            }
        },
        KeyCode::Char('*') => editor.search_word_under_cursor(),
        
        // Visual modes
        KeyCode::Char('v') => editor.enter_visual_mode(),
        KeyCode::Char('V') => editor.enter_visual_line_mode(),
        
        // Mode changes
        KeyCode::Char('i') => editor.enter_insert_mode(),
        KeyCode::Char('I') => {
            editor.move_to_first_nonblank();
            editor.enter_insert_mode();
        },
        KeyCode::Char('a') => {
            editor.move_cursor_right();
            editor.enter_insert_mode();
        },
        KeyCode::Char('A') => {
            editor.move_to_line_end();
            let line_len = crate::viewer::editor::char_count_pub(editor.get_current_line());
            editor.cursor_col = line_len; // Allow cursor past last char in insert mode
            editor.enter_insert_mode();
        },
        KeyCode::Char('o') => {
            editor.save_undo();
            editor.move_to_line_end();
            editor.insert_newline();
            editor.enter_insert_mode();
        },
        KeyCode::Char('O') => {
            editor.save_undo();
            editor.lines.insert(editor.cursor_row, String::new());
            editor.cursor_col = 0;
            editor.modified = true;
            editor.enter_insert_mode();
        },
        KeyCode::Char(':') => editor.enter_command_mode(),
        
        KeyCode::Esc => {
            editor.pending_op = PendingOperator::None;
            editor.count_buffer.clear();
            editor.status_message = "-- NORMAL --".to_string();
        },
        
        _ => {}
    }
}

/// Handle Insert mode keys
fn handle_insert_mode_keys(app: &mut App, key_code: KeyCode) {
    let editor = app.text_editor.as_mut().unwrap();
    
    match key_code {
        KeyCode::Esc => {
            editor.enter_normal_mode();
        },
        KeyCode::Char(c) => {
            editor.insert_char(c);
        },
        KeyCode::Enter => {
            editor.insert_newline();
        },
        KeyCode::Backspace => {
            editor.backspace();
        },
        KeyCode::Delete => {
            editor.delete_char();
        },
        KeyCode::Tab => {
            // Insert 4 spaces for tab
            for _ in 0..4 {
                editor.insert_char(' ');
            }
        },
        KeyCode::Left => editor.move_cursor_left(),
        KeyCode::Right => editor.move_cursor_right(),
        KeyCode::Up => editor.move_cursor_up(),
        KeyCode::Down => editor.move_cursor_down(),
        KeyCode::Home => editor.move_to_line_start(),
        KeyCode::End => {
            let line_len = crate::viewer::editor::char_count_pub(editor.get_current_line());
            editor.cursor_col = line_len;
        },
        _ => {}
    }
}

/// Handle Nano editor mode keys
pub fn handle_nano_keys(app: &mut App, key_code: KeyCode, modifiers: KeyModifiers) {
    let editor = match app.text_editor.as_mut() {
        Some(ed) => ed,
        None => return,
    };
    
    // Check if in search mode
    if editor.nano_search_mode {
        match key_code {
            KeyCode::Esc => {
                editor.nano_search_mode = false;
                editor.command_buffer.clear();
                editor.status_message = "Search cancelled".to_string();
            },
            KeyCode::Enter => {
                editor.search_pattern = editor.command_buffer.clone();
                editor.nano_search_mode = false;
                editor.command_buffer.clear();
                editor.search_next();
            },
            KeyCode::Char(c) => {
                editor.command_buffer.push(c);
                editor.status_message = format!("Search: {}", editor.command_buffer);
            },
            KeyCode::Backspace => {
                editor.command_buffer.pop();
                editor.status_message = format!("Search: {}", editor.command_buffer);
            },
            _ => {}
        }
        return;
    }
    
    // Handle Ctrl key combinations (nano shortcuts)
    if modifiers.contains(KeyModifiers::CONTROL) {
        match key_code {
            // Ctrl+X: Exit
            KeyCode::Char('x') | KeyCode::Char('X') => {
                if editor.modified {
                    editor.status_message = "Modified! Save first (^O) or use ESC to exit without saving".to_string();
                } else {
                    exit_editor(app, false);
                }
            },
            // Ctrl+O: Save (WriteOut)
            KeyCode::Char('o') | KeyCode::Char('O') => {
                let save_success = save_file(app);
                let editor = app.text_editor.as_mut().unwrap();
                if save_success {
                    editor.status_message = "File saved".to_string();
                } else {
                    editor.status_message = "Error saving file".to_string();
                }
            },
            // Ctrl+K: Cut line
            KeyCode::Char('k') | KeyCode::Char('K') => {
                editor.save_undo();
                editor.yank_line();
                editor.delete_line();
                editor.status_message = "Line cut".to_string();
            },
            // Ctrl+U: Paste (Uncut)
            KeyCode::Char('u') | KeyCode::Char('U') => {
                editor.paste_after();
                editor.status_message = "Pasted".to_string();
            },
            // Ctrl+W: Search (Where is)
            KeyCode::Char('w') | KeyCode::Char('W') => {
                editor.nano_search_mode = true;
                editor.command_buffer.clear();
                editor.status_message = "Search: ".to_string();
            },
            // Ctrl+G: Help (display shortcuts)
            KeyCode::Char('g') | KeyCode::Char('G') => {
                editor.status_message = "^X:Exit ^O:Save ^K:Cut ^U:Paste ^W:Search ^\\:Replace ^T:Vim".to_string();
            },
            // Ctrl+\: Replace
            KeyCode::Char('\\') => {
                editor.status_message = "Replace: Use :s/old/new/g in Vim mode (^T to switch)".to_string();
            },
            // Ctrl+A: Go to beginning of line
            KeyCode::Char('a') | KeyCode::Char('A') => {
                editor.move_to_line_start();
            },
            // Ctrl+E: Go to end of line
            KeyCode::Char('e') | KeyCode::Char('E') => {
                let line_len = crate::viewer::editor::char_count_pub(editor.get_current_line());
                editor.cursor_col = line_len;
            },
            // Ctrl+Y: Page up
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                editor.move_page_up();
            },
            // Ctrl+V: Page down
            KeyCode::Char('v') | KeyCode::Char('V') => {
                editor.move_page_down();
            },
            // Ctrl+_: Go to line
            KeyCode::Char('_') => {
                editor.status_message = "Go to line: (not implemented in nano mode, use :n in Vim mode)".to_string();
            },
            // Ctrl+C: Show cursor position
            KeyCode::Char('c') | KeyCode::Char('C') => {
                editor.status_message = format!("Line {}, Col {}, {} lines total",
                    editor.cursor_row + 1,
                    editor.cursor_col + 1,
                    editor.lines.len()
                );
            },
            // Ctrl+Z: Undo (non-standard for nano but useful)
            KeyCode::Char('z') | KeyCode::Char('Z') => {
                editor.undo();
            },
            _ => {}
        }
        return;
    }
    
    // Regular key handling (similar to Insert mode)
    match key_code {
        KeyCode::Esc => {
            // In nano, ESC can be used to exit without saving
            if editor.modified {
                editor.status_message = "Modified! Press ESC again to discard, ^O to save".to_string();
            } else {
                exit_editor(app, false);
            }
        },
        KeyCode::Char(c) => {
            editor.save_undo();
            editor.insert_char(c);
        },
        KeyCode::Enter => {
            editor.save_undo();
            editor.insert_newline();
        },
        KeyCode::Backspace => {
            editor.save_undo();
            editor.backspace();
        },
        KeyCode::Delete => {
            editor.save_undo();
            editor.delete_char();
        },
        KeyCode::Tab => {
            editor.save_undo();
            for _ in 0..4 {
                editor.insert_char(' ');
            }
        },
        KeyCode::Left => editor.move_cursor_left(),
        KeyCode::Right => editor.move_cursor_right(),
        KeyCode::Up => editor.move_cursor_up(),
        KeyCode::Down => editor.move_cursor_down(),
        KeyCode::Home => editor.move_to_line_start(),
        KeyCode::End => {
            let line_len = crate::viewer::editor::char_count_pub(editor.get_current_line());
            editor.cursor_col = line_len;
        },
        KeyCode::PageUp => editor.move_page_up(),
        KeyCode::PageDown => editor.move_page_down(),
        _ => {}
    }
}

/// Handle Command mode keys
fn handle_command_mode_keys(app: &mut App, key_code: KeyCode) {
    let editor = app.text_editor.as_mut().unwrap();
    
    // Handle replace char mode (after 'r' in normal mode)
    if editor.command_buffer == "r" {
        if let KeyCode::Char(c) = key_code {
            editor.command_buffer.clear();
            editor.replace_char(c);
            editor.enter_normal_mode();
            return;
        } else if key_code == KeyCode::Esc {
            editor.command_buffer.clear();
            editor.enter_normal_mode();
            return;
        }
    }
    
    // Handle search mode
    let is_search = editor.status_message.starts_with('/') || editor.status_message.starts_with('?');
    
    match key_code {
        KeyCode::Esc => {
            editor.enter_normal_mode();
        },
        KeyCode::Char(c) => {
            editor.append_command_char(c);
        },
        KeyCode::Backspace => {
            editor.backspace_command();
            if editor.command_buffer.is_empty() && !is_search {
                editor.enter_normal_mode();
            }
        },
        KeyCode::Enter => {
            if is_search {
                editor.execute_search();
            } else {
                execute_command(app);
            }
        },
        _ => {}
    }
}

/// Execute vim command
fn execute_command(app: &mut App) {
    let command = {
        let editor = app.text_editor.as_ref().unwrap();
        editor.command_buffer.clone()
    };
    
    // Handle line number commands (e.g., :123)
    if let Ok(line_num) = command.parse::<usize>() {
        let editor = app.text_editor.as_mut().unwrap();
        editor.move_to_line(line_num);
        editor.enter_normal_mode();
        return;
    }
    
    // Handle search and replace (%s/old/new/g)
    if command.starts_with("%s/") || command.starts_with("s/") {
        execute_substitute(app, &command);
        return;
    }
    
    match command.as_str() {
        "q" => {
            // Quit without saving
            let modified = app.text_editor.as_ref().unwrap().modified;
            if modified {
                let editor = app.text_editor.as_mut().unwrap();
                editor.status_message = "No write since last change (use :q! to force)".to_string();
                editor.enter_normal_mode();
            } else {
                exit_editor(app, false);
            }
        },
        "q!" => {
            // Force quit without saving
            exit_editor(app, false);
        },
        "w" => {
            // Save file
            let save_success = save_file(app);
            let editor = app.text_editor.as_mut().unwrap();
            if save_success {
                editor.status_message = format!("File saved{}", 
                    if let Some(path) = &editor.file_path {
                        format!(": {:?}", path.file_name().unwrap_or_default())
                    } else {
                        String::new()
                    }
                );
                editor.enter_normal_mode();
            } else {
                editor.status_message = "Error: Could not save file".to_string();
                editor.enter_normal_mode();
            }
        },
        "wq" | "x" | "wq!" | "x!" => {
            // Save and quit
            if save_file(app) {
                exit_editor(app, true);
            } else {
                let editor = app.text_editor.as_mut().unwrap();
                editor.status_message = "Error: Could not save file".to_string();
                editor.enter_normal_mode();
            }
        },
        "e!" => {
            // Reload file (discard changes)
            let editor = app.text_editor.as_mut().unwrap();
            if let Some(path) = editor.file_path.clone() {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    editor.lines = content.lines().map(|s| s.to_string()).collect();
                    if editor.lines.is_empty() {
                        editor.lines.push(String::new());
                    }
                    editor.cursor_row = 0;
                    editor.cursor_col = 0;
                    editor.modified = false;
                    editor.undo_stack.clear();
                    editor.redo_stack.clear();
                    editor.status_message = "File reloaded".to_string();
                } else {
                    editor.status_message = "Error reloading file".to_string();
                }
            }
            editor.enter_normal_mode();
        },
        "set nu" | "set number" => {
            let editor = app.text_editor.as_mut().unwrap();
            editor.status_message = "Line numbers are always shown".to_string();
            editor.enter_normal_mode();
        },
        "noh" | "nohlsearch" => {
            let editor = app.text_editor.as_mut().unwrap();
            editor.search_pattern.clear();
            editor.status_message = "Search highlighting cleared".to_string();
            editor.enter_normal_mode();
        },
        "$" => {
            // Go to last line
            let editor = app.text_editor.as_mut().unwrap();
            editor.move_to_last_line();
            editor.enter_normal_mode();
        },
        "0" => {
            // Go to first line
            let editor = app.text_editor.as_mut().unwrap();
            editor.move_to_first_line();
            editor.enter_normal_mode();
        },
        _ => {
            let editor = app.text_editor.as_mut().unwrap();
            editor.status_message = format!("Unknown command: {}", command);
            editor.enter_normal_mode();
        }
    }
}

/// Execute substitute command (:s/old/new/g or :%s/old/new/g)
fn execute_substitute(app: &mut App, command: &str) {
    let editor = app.text_editor.as_mut().unwrap();
    
    let global = command.starts_with("%s/");
    let parts: Vec<&str> = if global {
        command[3..].splitn(3, '/').collect()
    } else {
        command[2..].splitn(3, '/').collect()
    };
    
    if parts.len() < 2 {
        editor.status_message = "Invalid substitute command".to_string();
        editor.enter_normal_mode();
        return;
    }
    
    let pattern = parts[0];
    let replacement = parts[1];
    let flags = if parts.len() > 2 { parts[2] } else { "" };
    let replace_all = flags.contains('g');
    
    editor.save_undo();
    
    let mut total_replacements = 0;
    
    if global {
        // Replace in all lines
        for line in &mut editor.lines {
            if replace_all {
                let count = line.matches(pattern).count();
                total_replacements += count;
                *line = line.replace(pattern, replacement);
            } else if line.contains(pattern) {
                *line = line.replacen(pattern, replacement, 1);
                total_replacements += 1;
            }
        }
    } else {
        // Replace in current line only
        let line = &mut editor.lines[editor.cursor_row];
        if replace_all {
            let count = line.matches(pattern).count();
            total_replacements += count;
            *line = line.replace(pattern, replacement);
        } else if line.contains(pattern) {
            *line = line.replacen(pattern, replacement, 1);
            total_replacements += 1;
        }
    }
    
    if total_replacements > 0 {
        editor.modified = true;
        editor.status_message = format!("{} substitution(s) made", total_replacements);
    } else {
        editor.status_message = format!("Pattern not found: {}", pattern);
    }
    editor.enter_normal_mode();
}

