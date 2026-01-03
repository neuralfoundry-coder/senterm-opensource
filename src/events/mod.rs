//! Event handling module
//! 
//! This module handles all keyboard and user input events for the application.

mod file_manager;
mod viewer;
mod editor;
mod settings;
mod dialog;
mod shell;
mod clipboard;
mod process;

// Re-export all public handlers
pub use file_manager::handle_file_manager_keys;
pub use viewer::handle_viewer_keys;
pub use settings::handle_settings_keys;
pub use dialog::handle_dialog_keys;
pub use shell::handle_shell_keys;
pub use shell::handle_console_keys;
pub use clipboard::handle_clipboard_operations;
pub use process::handle_process_viewer_keys;

// Re-export setup handler (minimal, kept inline)
pub use self::setup::handle_setup_keys;

mod setup {
    use crossterm::event::KeyCode;
    use crate::app::App;
    
    /// Handle setup mode key events
    pub fn handle_setup_keys(app: &mut App, key_code: KeyCode) {
        match key_code {
            KeyCode::Enter => {
                app.config.first_run = false;
                app.toggle_mode(true); // Switch to FileManager
            },
            _ => {}
        }
    }
}

// Shared utility functions
pub(crate) mod utils {
    use arboard::Clipboard;
    use crate::app::App;
    
    /// Copy text to system clipboard
    pub fn copy_to_system_clipboard(text: &str) -> Result<(), String> {
        match Clipboard::new() {
            Ok(mut clipboard) => {
                clipboard.set_text(text).map_err(|e| e.to_string())
            },
            Err(e) => Err(e.to_string())
        }
    }
    
    /// Paste text from system clipboard
    pub fn paste_from_system_clipboard() -> Result<String, String> {
        match Clipboard::new() {
            Ok(mut clipboard) => {
                clipboard.get_text().map_err(|e| e.to_string())
            },
            Err(e) => Err(e.to_string())
        }
    }
    
    /// Get viewer content as plain text
    pub fn get_viewer_content_text(app: &App) -> Option<String> {
        match &app.viewer_content {
            Some(crate::viewer::ViewerContent::PlainText(s)) => Some(s.clone()),
            Some(crate::viewer::ViewerContent::HighlightedCode { raw, .. }) => Some(raw.clone()),
            Some(crate::viewer::ViewerContent::Markdown(s)) => Some(s.clone()),
            Some(crate::viewer::ViewerContent::HexView(_, _)) => None,
            Some(crate::viewer::ViewerContent::Image(_)) => None,
            Some(crate::viewer::ViewerContent::ImagePreviewContent(_)) => None,
            Some(crate::viewer::ViewerContent::Error(e)) => Some(e.clone()),
            None => None,
        }
    }
}
