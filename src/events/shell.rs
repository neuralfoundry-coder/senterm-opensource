//! Shell popup event handling

use crossterm::event::{KeyCode, KeyModifiers};
use crate::app::App;

/// Handle shell popup key events (PTY-based)
/// Returns true if the key was handled
pub fn handle_shell_keys(app: &mut App, key_code: KeyCode, modifiers: KeyModifiers) -> bool {
    // Check if shell is running
    if !app.shell.is_running {
        // Shell not running, only allow closing
        if matches!(key_code, KeyCode::Esc | KeyCode::F(12)) || key_code == KeyCode::Char('`') {
            app.toggle_shell();
            return true;
        }
        return false;
    }
    
    // Convert key to bytes and send to PTY
    let bytes: Option<Vec<u8>> = match key_code {
        // Close shell - special handling
        KeyCode::F(12) => {
            app.toggle_shell();
            return true;
        },
        // Backtick with no modifiers closes shell
        KeyCode::Char('`') if modifiers.is_empty() => {
            app.toggle_shell();
            return true;
        },
        
        // Regular characters
        KeyCode::Char(c) => {
            if modifiers.contains(KeyModifiers::CONTROL) {
                // Ctrl+key combinations
                let ctrl_char = (c.to_ascii_lowercase() as u8).wrapping_sub(b'a').wrapping_add(1);
                Some(vec![ctrl_char])
            } else if modifiers.contains(KeyModifiers::ALT) {
                // Alt+key - send escape followed by character
                Some(vec![0x1b, c as u8])
            } else {
                Some(c.to_string().into_bytes())
            }
        },
        
        // Special keys
        KeyCode::Enter => Some(vec![b'\r']),
        KeyCode::Tab => Some(vec![b'\t']),
        KeyCode::Backspace => Some(vec![0x7f]),  // DEL character
        KeyCode::Esc => Some(vec![0x1b]),
        KeyCode::Delete => Some(b"\x1b[3~".to_vec()),
        
        // Arrow keys
        KeyCode::Up => Some(b"\x1b[A".to_vec()),
        KeyCode::Down => Some(b"\x1b[B".to_vec()),
        KeyCode::Right => Some(b"\x1b[C".to_vec()),
        KeyCode::Left => Some(b"\x1b[D".to_vec()),
        
        // Home/End
        KeyCode::Home => Some(b"\x1b[H".to_vec()),
        KeyCode::End => Some(b"\x1b[F".to_vec()),
        
        // Page Up/Down
        KeyCode::PageUp => Some(b"\x1b[5~".to_vec()),
        KeyCode::PageDown => Some(b"\x1b[6~".to_vec()),
        
        // Insert
        KeyCode::Insert => Some(b"\x1b[2~".to_vec()),
        
        // Function keys
        KeyCode::F(1) => Some(b"\x1bOP".to_vec()),
        KeyCode::F(2) => Some(b"\x1bOQ".to_vec()),
        KeyCode::F(3) => Some(b"\x1bOR".to_vec()),
        KeyCode::F(4) => Some(b"\x1bOS".to_vec()),
        KeyCode::F(5) => Some(b"\x1b[15~".to_vec()),
        KeyCode::F(6) => Some(b"\x1b[17~".to_vec()),
        KeyCode::F(7) => Some(b"\x1b[18~".to_vec()),
        KeyCode::F(8) => Some(b"\x1b[19~".to_vec()),
        KeyCode::F(9) => Some(b"\x1b[20~".to_vec()),
        KeyCode::F(10) => Some(b"\x1b[21~".to_vec()),
        KeyCode::F(11) => Some(b"\x1b[23~".to_vec()),
        // F12 handled above for closing shell
        KeyCode::F(_) => None,
        
        _ => None,
    };
    
    if let Some(data) = bytes {
        if let Err(e) = app.shell.write(&data) {
            tracing::error!("Failed to write to PTY: {}", e);
        }
        true
    } else {
        false
    }
}

/// Handle console panel key events when panel is focused
/// Returns true if the key was handled
/// 
/// Note: F5 toggle key is handled globally in main.rs
pub fn handle_console_keys(app: &mut App, key_code: KeyCode, modifiers: KeyModifiers) -> bool {
    // Handle console panel keys if console is shown and focused
    if app.show_console && app.console_focus {
        return handle_console_shell_keys(app, key_code, modifiers);
    }
    
    false
}

/// Handle shell mode console keys
fn handle_console_shell_keys(app: &mut App, key_code: KeyCode, modifiers: KeyModifiers) -> bool {
    // Check if console is running
    if !app.console.is_running {
        // Console not running, allow focus cycling or Esc to unfocus
        match key_code {
            KeyCode::Tab => {
                app.cycle_focus_forward();
                return true;
            },
            KeyCode::Esc => {
                app.console_focus = false;
                return true;
            },
            _ => return false,
        }
    }
    
    // Convert key to bytes and send to PTY
    let bytes: Option<Vec<u8>> = match key_code {
        // Escape unfocuses console (returns focus to file manager)
        KeyCode::Esc => {
            app.console_focus = false;
            return true;
        },
        // Tab is sent to terminal for shell completion
        KeyCode::Tab => Some(vec![b'\t']),
        
        // Regular characters
        KeyCode::Char(c) => {
            if modifiers.contains(KeyModifiers::CONTROL) {
                // Ctrl+key combinations
                let ctrl_char = (c.to_ascii_lowercase() as u8).wrapping_sub(b'a').wrapping_add(1);
                Some(vec![ctrl_char])
            } else if modifiers.contains(KeyModifiers::ALT) {
                // Alt+key - send escape followed by character
                Some(vec![0x1b, c as u8])
            } else {
                Some(c.to_string().into_bytes())
            }
        },
        
        // Special keys
        KeyCode::Enter => Some(vec![b'\r']),
        KeyCode::Backspace => Some(vec![0x7f]),  // DEL character
        KeyCode::Delete => Some(b"\x1b[3~".to_vec()),
        
        // Arrow keys
        KeyCode::Up => Some(b"\x1b[A".to_vec()),
        KeyCode::Down => Some(b"\x1b[B".to_vec()),
        KeyCode::Right => Some(b"\x1b[C".to_vec()),
        KeyCode::Left => Some(b"\x1b[D".to_vec()),
        
        // Home/End
        KeyCode::Home => Some(b"\x1b[H".to_vec()),
        KeyCode::End => Some(b"\x1b[F".to_vec()),
        
        // Page Up/Down
        KeyCode::PageUp => Some(b"\x1b[5~".to_vec()),
        KeyCode::PageDown => Some(b"\x1b[6~".to_vec()),
        
        // Insert
        KeyCode::Insert => Some(b"\x1b[2~".to_vec()),
        
        // Function keys (pass through)
        KeyCode::F(1) => Some(b"\x1bOP".to_vec()),
        KeyCode::F(2) => Some(b"\x1bOQ".to_vec()),
        KeyCode::F(3) => Some(b"\x1bOR".to_vec()),
        KeyCode::F(4) => Some(b"\x1bOS".to_vec()),
        KeyCode::F(5) => Some(b"\x1b[15~".to_vec()),
        KeyCode::F(6) => Some(b"\x1b[17~".to_vec()),
        KeyCode::F(7) => Some(b"\x1b[18~".to_vec()),
        KeyCode::F(8) => Some(b"\x1b[19~".to_vec()),
        KeyCode::F(9) => Some(b"\x1b[20~".to_vec()),
        KeyCode::F(10) => Some(b"\x1b[21~".to_vec()),
        KeyCode::F(11) => Some(b"\x1b[23~".to_vec()),
        KeyCode::F(12) => Some(b"\x1b[24~".to_vec()),
        KeyCode::F(_) => None,
        
        _ => None,
    };
    
    if let Some(data) = bytes {
        if let Err(e) = app.console.write(&data) {
            tracing::error!("Failed to write to console PTY: {}", e);
        }
        true
    } else {
        false
    }
}
