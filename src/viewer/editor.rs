use std::path::PathBuf;

/// Editor style (Vim or Nano)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorStyle {
    Vim,
    Nano,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VimMode {
    Normal,
    Insert,
    Command,
    Visual,
    VisualLine,
}

/// Pending operator for operator-pending mode (e.g., d, y, c followed by motion)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PendingOperator {
    None,
    Delete,   // d
    Yank,     // y
    Change,   // c
    Indent,   // >
    Outdent,  // <
}

#[derive(Debug, Clone)]
pub struct TextEditor {
    pub file_path: Option<PathBuf>,
    pub lines: Vec<String>,
    pub cursor_row: usize,
    pub cursor_col: usize, // Character index, not byte index
    pub mode: VimMode,
    pub editor_style: EditorStyle, // Vim or Nano
    pub command_buffer: String,
    pub clipboard: Vec<String>,
    pub clipboard_is_line: bool, // True if clipboard contains whole lines
    pub status_message: String,
    pub modified: bool,
    // Operator-pending mode
    pub pending_op: PendingOperator,
    // Undo/Redo
    pub undo_stack: Vec<(Vec<String>, usize, usize)>, // (lines, row, col)
    pub redo_stack: Vec<(Vec<String>, usize, usize)>,
    // Search
    pub search_pattern: String,
    pub search_direction: bool, // true = forward, false = backward
    pub last_search_row: usize,
    pub last_search_col: usize,
    // Visual mode
    pub visual_start_row: usize,
    pub visual_start_col: usize,
    // Count prefix (e.g., 5j to move down 5 lines)
    pub count_buffer: String,
    // Nano specific
    pub nano_search_mode: bool, // True when Ctrl+W search is active
}

// Helper functions for UTF-8 safe string operations
fn char_count(s: &str) -> usize {
    s.chars().count()
}

fn char_to_byte_index(s: &str, char_idx: usize) -> usize {
    s.char_indices()
        .nth(char_idx)
        .map(|(byte_idx, _)| byte_idx)
        .unwrap_or(s.len())
}

fn insert_char_at(s: &mut String, char_idx: usize, c: char) {
    let byte_idx = char_to_byte_index(s, char_idx);
    s.insert(byte_idx, c);
}

fn remove_char_at(s: &mut String, char_idx: usize) -> Option<char> {
    let byte_idx = char_to_byte_index(s, char_idx);
    if byte_idx < s.len() {
        Some(s.remove(byte_idx))
    } else {
        None
    }
}

fn split_off_at_char(s: &mut String, char_idx: usize) -> String {
    let byte_idx = char_to_byte_index(s, char_idx);
    s.split_off(byte_idx)
}

/// Check if a character is a word character (alphanumeric or underscore)
fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// Public helper for char count
pub fn char_count_pub(s: &str) -> usize {
    s.chars().count()
}

impl TextEditor {
    pub fn new(content: String, file_path: Option<PathBuf>) -> Self {
        let lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
        let lines = if lines.is_empty() {
            vec![String::new()]
        } else {
            lines
        };

        TextEditor {
            file_path,
            lines,
            cursor_row: 0,
            cursor_col: 0,
            mode: VimMode::Normal,
            editor_style: EditorStyle::Vim,
            command_buffer: String::new(),
            clipboard: Vec::new(),
            clipboard_is_line: false,
            status_message: "-- NORMAL --".to_string(),
            modified: false,
            pending_op: PendingOperator::None,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            search_pattern: String::new(),
            search_direction: true,
            last_search_row: 0,
            last_search_col: 0,
            visual_start_row: 0,
            visual_start_col: 0,
            count_buffer: String::new(),
            nano_search_mode: false,
        }
    }
    
    /// Toggle between Vim and Nano editor styles
    pub fn toggle_editor_style(&mut self) {
        match self.editor_style {
            EditorStyle::Vim => {
                self.editor_style = EditorStyle::Nano;
                self.mode = VimMode::Insert; // Nano is always in "insert" mode
                self.status_message = "-- NANO MODE -- ^X:Exit ^O:Save ^K:Cut ^U:Paste ^W:Search".to_string();
            },
            EditorStyle::Nano => {
                self.editor_style = EditorStyle::Vim;
                self.mode = VimMode::Normal;
                self.status_message = "-- NORMAL --".to_string();
            },
        }
        self.pending_op = PendingOperator::None;
        self.count_buffer.clear();
        self.nano_search_mode = false;
    }
    
    /// Get the count from count_buffer, default to 1
    pub fn get_count(&mut self) -> usize {
        let count = self.count_buffer.parse::<usize>().unwrap_or(1).max(1);
        self.count_buffer.clear();
        count
    }
    
    /// Save current state for undo
    pub fn save_undo(&mut self) {
        self.undo_stack.push((self.lines.clone(), self.cursor_row, self.cursor_col));
        self.redo_stack.clear(); // Clear redo on new change
        // Limit undo stack size
        if self.undo_stack.len() > 100 {
            self.undo_stack.remove(0);
        }
    }
    
    /// Undo last change
    pub fn undo(&mut self) {
        if let Some((lines, row, col)) = self.undo_stack.pop() {
            self.redo_stack.push((self.lines.clone(), self.cursor_row, self.cursor_col));
            self.lines = lines;
            self.cursor_row = row.min(self.lines.len().saturating_sub(1));
            self.cursor_col = col;
            self.clamp_cursor_col();
            self.status_message = "Undo".to_string();
        } else {
            self.status_message = "Already at oldest change".to_string();
        }
    }
    
    /// Redo last undone change
    pub fn redo(&mut self) {
        if let Some((lines, row, col)) = self.redo_stack.pop() {
            self.undo_stack.push((self.lines.clone(), self.cursor_row, self.cursor_col));
            self.lines = lines;
            self.cursor_row = row.min(self.lines.len().saturating_sub(1));
            self.cursor_col = col;
            self.clamp_cursor_col();
            self.status_message = "Redo".to_string();
        } else {
            self.status_message = "Already at newest change".to_string();
        }
    }

    pub fn get_content(&self) -> String {
        self.lines.join("\n")
    }

    pub fn get_current_line(&self) -> &str {
        self.lines.get(self.cursor_row).map(|s| s.as_str()).unwrap_or("")
    }

    pub fn get_current_line_mut(&mut self) -> &mut String {
        &mut self.lines[self.cursor_row]
    }

    // Movement commands
    pub fn move_cursor_left(&mut self) {
        if self.cursor_col > 0 {
            self.cursor_col -= 1;
        }
    }

    pub fn move_cursor_right(&mut self) {
        let line_len = char_count(self.get_current_line());
        let max_col = if self.mode == VimMode::Insert {
            line_len
        } else {
            line_len.saturating_sub(1)
        };
        
        if self.cursor_col < max_col {
            self.cursor_col += 1;
        }
    }

    pub fn move_cursor_up(&mut self) {
        if self.cursor_row > 0 {
            self.cursor_row -= 1;
            self.clamp_cursor_col();
        }
    }

    pub fn move_cursor_down(&mut self) {
        if self.cursor_row < self.lines.len() - 1 {
            self.cursor_row += 1;
            self.clamp_cursor_col();
        }
    }

    pub fn move_to_line_start(&mut self) {
        self.cursor_col = 0;
    }

    pub fn move_to_line_end(&mut self) {
        let line_len = char_count(self.get_current_line());
        self.cursor_col = if self.mode == VimMode::Insert {
            line_len
        } else {
            line_len.saturating_sub(1).max(0)
        };
    }

    pub fn move_to_first_line(&mut self) {
        self.cursor_row = 0;
        self.cursor_col = 0;
    }

    pub fn move_to_last_line(&mut self) {
        self.cursor_row = self.lines.len().saturating_sub(1);
        self.cursor_col = 0;
    }
    
    /// Move to line number (1-indexed)
    pub fn move_to_line(&mut self, line_num: usize) {
        if line_num > 0 {
            self.cursor_row = (line_num - 1).min(self.lines.len().saturating_sub(1));
            self.cursor_col = 0;
        }
    }
    
    /// Move to first non-blank character (^)
    pub fn move_to_first_nonblank(&mut self) {
        let line = self.get_current_line();
        self.cursor_col = line.chars()
            .position(|c| !c.is_whitespace())
            .unwrap_or(0);
    }
    
    /// Move forward by word (w)
    pub fn move_word_forward(&mut self) {
        let line = self.get_current_line();
        let chars: Vec<char> = line.chars().collect();
        let len = chars.len();
        
        if self.cursor_col >= len {
            // Move to next line
            if self.cursor_row < self.lines.len() - 1 {
                self.cursor_row += 1;
                self.cursor_col = 0;
                self.move_to_first_nonblank();
            }
            return;
        }
        
        let mut col = self.cursor_col;
        
        // Skip current word
        if col < len && is_word_char(chars[col]) {
            while col < len && is_word_char(chars[col]) {
                col += 1;
            }
        } else if col < len && !chars[col].is_whitespace() {
            // Non-word, non-whitespace
            while col < len && !chars[col].is_whitespace() && !is_word_char(chars[col]) {
                col += 1;
            }
        }
        
        // Skip whitespace
        while col < len && chars[col].is_whitespace() {
            col += 1;
        }
        
        if col >= len {
            // Move to next line
            if self.cursor_row < self.lines.len() - 1 {
                self.cursor_row += 1;
                self.cursor_col = 0;
                self.move_to_first_nonblank();
            } else {
                self.cursor_col = len.saturating_sub(1);
            }
        } else {
            self.cursor_col = col;
        }
    }
    
    /// Move backward by word (b)
    pub fn move_word_backward(&mut self) {
        if self.cursor_col == 0 {
            // Move to previous line
            if self.cursor_row > 0 {
                self.cursor_row -= 1;
                let line = self.get_current_line();
                self.cursor_col = char_count(line).saturating_sub(1);
            }
            return;
        }
        
        let line = self.get_current_line();
        let chars: Vec<char> = line.chars().collect();
        let mut col = self.cursor_col.saturating_sub(1);
        
        // Skip whitespace
        while col > 0 && chars[col].is_whitespace() {
            col -= 1;
        }
        
        // Find start of word
        if is_word_char(chars[col]) {
            while col > 0 && is_word_char(chars[col - 1]) {
                col -= 1;
            }
        } else if !chars[col].is_whitespace() {
            while col > 0 && !chars[col - 1].is_whitespace() && !is_word_char(chars[col - 1]) {
                col -= 1;
            }
        }
        
        self.cursor_col = col;
    }
    
    /// Move to end of word (e)
    pub fn move_word_end(&mut self) {
        let line = self.get_current_line();
        let chars: Vec<char> = line.chars().collect();
        let len = chars.len();
        
        if self.cursor_col >= len.saturating_sub(1) {
            // Move to next line
            if self.cursor_row < self.lines.len() - 1 {
                self.cursor_row += 1;
                self.cursor_col = 0;
            }
            return;
        }
        
        let mut col = self.cursor_col + 1;
        
        // Skip whitespace
        while col < len && chars[col].is_whitespace() {
            col += 1;
        }
        
        if col >= len {
            if self.cursor_row < self.lines.len() - 1 {
                self.cursor_row += 1;
                self.cursor_col = 0;
                let new_line = self.get_current_line();
                let new_chars: Vec<char> = new_line.chars().collect();
                // Find end of first word
                let mut new_col = 0;
                while new_col < new_chars.len() && new_chars[new_col].is_whitespace() {
                    new_col += 1;
                }
                while new_col < new_chars.len().saturating_sub(1) {
                    if is_word_char(new_chars[new_col]) {
                        if !is_word_char(new_chars[new_col + 1]) {
                            break;
                        }
                    } else if !new_chars[new_col].is_whitespace() {
                        if new_chars[new_col + 1].is_whitespace() || is_word_char(new_chars[new_col + 1]) {
                            break;
                        }
                    }
                    new_col += 1;
                }
                self.cursor_col = new_col;
            }
            return;
        }
        
        // Find end of word
        while col < len.saturating_sub(1) {
            if is_word_char(chars[col]) {
                if !is_word_char(chars[col + 1]) {
                    break;
                }
            } else if !chars[col].is_whitespace() {
                if chars[col + 1].is_whitespace() || is_word_char(chars[col + 1]) {
                    break;
                }
            }
            col += 1;
        }
        
        self.cursor_col = col.min(len.saturating_sub(1));
    }
    
    /// Move half page down (Ctrl+d)
    pub fn move_half_page_down(&mut self) {
        let half_page = 15;
        self.cursor_row = (self.cursor_row + half_page).min(self.lines.len().saturating_sub(1));
        self.clamp_cursor_col();
    }
    
    /// Move half page up (Ctrl+u)
    pub fn move_half_page_up(&mut self) {
        let half_page = 15;
        self.cursor_row = self.cursor_row.saturating_sub(half_page);
        self.clamp_cursor_col();
    }
    
    /// Move full page down (Ctrl+f)
    pub fn move_page_down(&mut self) {
        let page = 30;
        self.cursor_row = (self.cursor_row + page).min(self.lines.len().saturating_sub(1));
        self.clamp_cursor_col();
    }
    
    /// Move full page up (Ctrl+b)
    pub fn move_page_up(&mut self) {
        let page = 30;
        self.cursor_row = self.cursor_row.saturating_sub(page);
        self.clamp_cursor_col();
    }
    
    /// Find matching bracket (%)
    pub fn move_to_matching_bracket(&mut self) {
        let line = self.get_current_line();
        let chars: Vec<char> = line.chars().collect();
        
        if self.cursor_col >= chars.len() {
            return;
        }
        
        let current = chars[self.cursor_col];
        let (target, forward) = match current {
            '(' => (')', true),
            ')' => ('(', false),
            '[' => (']', true),
            ']' => ('[', false),
            '{' => ('}', true),
            '}' => ('{', false),
            '<' => ('>', true),
            '>' => ('<', false),
            _ => return,
        };
        
        let mut depth = 1;
        let mut row = self.cursor_row;
        let mut col = self.cursor_col;
        
        if forward {
            col += 1;
            while row < self.lines.len() {
                let line_chars: Vec<char> = self.lines[row].chars().collect();
                while col < line_chars.len() {
                    if line_chars[col] == current {
                        depth += 1;
                    } else if line_chars[col] == target {
                        depth -= 1;
                        if depth == 0 {
                            self.cursor_row = row;
                            self.cursor_col = col;
                            return;
                        }
                    }
                    col += 1;
                }
                row += 1;
                col = 0;
            }
        } else {
            if col > 0 {
                col -= 1;
            } else if row > 0 {
                row -= 1;
                col = self.lines[row].chars().count().saturating_sub(1);
            }
            loop {
                let line_chars: Vec<char> = self.lines[row].chars().collect();
                loop {
                    if col < line_chars.len() {
                        if line_chars[col] == current {
                            depth += 1;
                        } else if line_chars[col] == target {
                            depth -= 1;
                            if depth == 0 {
                                self.cursor_row = row;
                                self.cursor_col = col;
                                return;
                            }
                        }
                    }
                    if col == 0 {
                        break;
                    }
                    col -= 1;
                }
                if row == 0 {
                    break;
                }
                row -= 1;
                col = self.lines[row].chars().count().saturating_sub(1);
            }
        }
    }

    fn clamp_cursor_col(&mut self) {
        let line_len = char_count(self.get_current_line());
        let max_col = if self.mode == VimMode::Insert {
            line_len
        } else {
            line_len.saturating_sub(1).max(0)
        };
        self.cursor_col = self.cursor_col.min(max_col);
    }

    // Edit commands (Normal mode)
    pub fn delete_char(&mut self) {
        self.save_undo();
        let cursor_col = self.cursor_col;
        let line = self.get_current_line_mut();
        if cursor_col < char_count(line) {
            let c = remove_char_at(line, cursor_col);
            if let Some(ch) = c {
                self.clipboard = vec![ch.to_string()];
                self.clipboard_is_line = false;
            }
            self.modified = true;
            self.clamp_cursor_col();
        }
    }
    
    /// Delete character before cursor (X)
    pub fn delete_char_before(&mut self) {
        if self.cursor_col > 0 {
            self.save_undo();
            self.cursor_col -= 1;
            let cursor_col = self.cursor_col;
            let line = self.get_current_line_mut();
            let c = remove_char_at(line, cursor_col);
            if let Some(ch) = c {
                self.clipboard = vec![ch.to_string()];
                self.clipboard_is_line = false;
            }
            self.modified = true;
        }
    }

    pub fn delete_line(&mut self) {
        self.save_undo();
        if self.lines.len() > 1 {
            let deleted_line = self.lines.remove(self.cursor_row);
            self.clipboard = vec![deleted_line];
            self.clipboard_is_line = true;
            if self.cursor_row >= self.lines.len() {
                self.cursor_row = self.lines.len() - 1;
            }
            self.modified = true;
        } else {
            // Last line - just clear it
            self.clipboard = vec![self.lines[0].clone()];
            self.clipboard_is_line = true;
            self.lines[0].clear();
            self.cursor_col = 0;
            self.modified = true;
        }
    }

    pub fn yank_line(&mut self) {
        self.clipboard = vec![self.get_current_line().to_string()];
        self.clipboard_is_line = true;
        self.status_message = "1 line yanked".to_string();
    }
    
    /// Yank multiple lines
    pub fn yank_lines(&mut self, count: usize) {
        let end = (self.cursor_row + count).min(self.lines.len());
        self.clipboard = self.lines[self.cursor_row..end].to_vec();
        self.clipboard_is_line = true;
        self.status_message = format!("{} lines yanked", self.clipboard.len());
    }
    
    /// Yank from cursor to end of line (y$)
    pub fn yank_to_end(&mut self) {
        let line = self.get_current_line();
        let chars: Vec<char> = line.chars().collect();
        let text: String = chars[self.cursor_col..].iter().collect();
        self.clipboard = vec![text];
        self.clipboard_is_line = false;
        self.status_message = "Yanked to end of line".to_string();
    }
    
    /// Delete to end of line (D or d$)
    pub fn delete_to_end(&mut self) {
        self.save_undo();
        let cursor_col = self.cursor_col;
        let chars: Vec<char> = self.lines[self.cursor_row].chars().collect();
        let deleted: String = chars[cursor_col..].iter().collect();
        self.clipboard = vec![deleted];
        self.clipboard_is_line = false;
        self.lines[self.cursor_row] = chars[..cursor_col].iter().collect();
        self.modified = true;
        self.clamp_cursor_col();
    }
    
    /// Delete multiple lines
    pub fn delete_lines(&mut self, count: usize) {
        self.save_undo();
        let end = (self.cursor_row + count).min(self.lines.len());
        self.clipboard = self.lines.drain(self.cursor_row..end).collect();
        self.clipboard_is_line = true;
        if self.lines.is_empty() {
            self.lines.push(String::new());
        }
        self.cursor_row = self.cursor_row.min(self.lines.len().saturating_sub(1));
        self.cursor_col = 0;
        self.modified = true;
        self.status_message = format!("{} lines deleted", self.clipboard.len());
    }

    pub fn paste_after(&mut self) {
        if self.clipboard.is_empty() {
            return;
        }
        self.save_undo();
        
        if self.clipboard_is_line {
            for line in self.clipboard.iter().rev() {
                self.lines.insert(self.cursor_row + 1, line.clone());
            }
            self.cursor_row += 1;
            self.cursor_col = 0;
        } else {
            // Paste inline after cursor
            let text = self.clipboard.join("\n");
            let cursor_col = self.cursor_col + 1;
            let line = self.get_current_line_mut();
            let byte_idx = char_to_byte_index(line, cursor_col.min(char_count(line)));
            line.insert_str(byte_idx, &text);
            self.cursor_col = cursor_col + char_count(&text) - 1;
        }
        self.modified = true;
    }
    
    /// Paste before cursor (P)
    pub fn paste_before(&mut self) {
        if self.clipboard.is_empty() {
            return;
        }
        self.save_undo();
        
        if self.clipboard_is_line {
            for line in self.clipboard.iter().rev() {
                self.lines.insert(self.cursor_row, line.clone());
            }
            self.cursor_col = 0;
        } else {
            // Paste inline at cursor
            let text = self.clipboard.join("\n");
            let cursor_col = self.cursor_col;
            let line = self.get_current_line_mut();
            let byte_idx = char_to_byte_index(line, cursor_col);
            line.insert_str(byte_idx, &text);
            self.cursor_col = cursor_col + char_count(&text);
        }
        self.modified = true;
    }
    
    /// Paste text from system clipboard at cursor position
    pub fn paste_text(&mut self, text: &str) {
        if text.is_empty() {
            return;
        }
        self.save_undo();
        
        let paste_lines: Vec<&str> = text.lines().collect();
        
        if paste_lines.len() == 1 {
            // Single line paste - insert at cursor position
            let cursor_col = self.cursor_col;
            let line = self.get_current_line_mut();
            let byte_idx = char_to_byte_index(line, cursor_col);
            line.insert_str(byte_idx, paste_lines[0]);
            self.cursor_col = cursor_col + char_count(paste_lines[0]);
        } else {
            // Multi-line paste
            let cursor_col = self.cursor_col;
            let current_line = self.lines[self.cursor_row].clone();
            let chars: Vec<char> = current_line.chars().collect();
            
            // Split current line at cursor
            let before: String = chars[..cursor_col].iter().collect();
            let after: String = chars[cursor_col..].iter().collect();
            
            // First line: before + first paste line
            self.lines[self.cursor_row] = format!("{}{}", before, paste_lines[0]);
            
            // Middle lines: insert as-is
            for (i, line) in paste_lines.iter().skip(1).take(paste_lines.len() - 2).enumerate() {
                self.lines.insert(self.cursor_row + 1 + i, line.to_string());
            }
            
            // Last line: last paste line + after
            if paste_lines.len() > 1 {
                let last_paste_line = paste_lines[paste_lines.len() - 1];
                let insert_idx = self.cursor_row + paste_lines.len() - 1;
                self.lines.insert(insert_idx, format!("{}{}", last_paste_line, after));
                
                // Move cursor to end of pasted text
                self.cursor_row = insert_idx;
                self.cursor_col = char_count(last_paste_line);
            }
        }
        self.modified = true;
        
        // Enter insert mode if in normal mode (for smoother editing experience)
        if self.mode == VimMode::Normal {
            self.enter_insert_mode();
        }
    }
    
    /// Replace single character (r)
    pub fn replace_char(&mut self, c: char) {
        self.save_undo();
        let cursor_col = self.cursor_col;
        let line = self.get_current_line_mut();
        if cursor_col < char_count(line) {
            remove_char_at(line, cursor_col);
            insert_char_at(line, cursor_col, c);
            self.modified = true;
        }
    }
    
    /// Substitute character (s) - delete char and enter insert mode
    pub fn substitute_char(&mut self) {
        self.save_undo();
        self.delete_char();
        self.enter_insert_mode();
    }
    
    /// Substitute line (S or cc) - clear line and enter insert mode
    pub fn substitute_line(&mut self) {
        self.save_undo();
        self.clipboard = vec![self.lines[self.cursor_row].clone()];
        self.clipboard_is_line = true;
        self.lines[self.cursor_row].clear();
        self.cursor_col = 0;
        self.modified = true;
        self.enter_insert_mode();
    }
    
    /// Change to end of line (C or c$)
    pub fn change_to_end(&mut self) {
        self.save_undo();
        let cursor_col = self.cursor_col;
        let chars: Vec<char> = self.lines[self.cursor_row].chars().collect();
        self.clipboard = vec![chars[cursor_col..].iter().collect()];
        self.clipboard_is_line = false;
        self.lines[self.cursor_row] = chars[..cursor_col].iter().collect();
        self.modified = true;
        self.enter_insert_mode();
    }
    
    /// Join lines (J)
    pub fn join_lines(&mut self) {
        if self.cursor_row < self.lines.len() - 1 {
            self.save_undo();
            let next_line = self.lines.remove(self.cursor_row + 1);
            let trimmed = next_line.trim_start();
            let current_len = char_count(&self.lines[self.cursor_row]);
            
            if !self.lines[self.cursor_row].is_empty() && !trimmed.is_empty() {
                self.lines[self.cursor_row].push(' ');
                self.cursor_col = current_len;
            } else {
                self.cursor_col = current_len;
            }
            self.lines[self.cursor_row].push_str(trimmed);
            self.modified = true;
        }
    }
    
    /// Indent current line (>>)
    pub fn indent_line(&mut self) {
        self.save_undo();
        self.lines[self.cursor_row].insert_str(0, "    ");
        self.cursor_col += 4;
        self.modified = true;
    }
    
    /// Outdent current line (<<)
    pub fn outdent_line(&mut self) {
        self.save_undo();
        let line = &mut self.lines[self.cursor_row];
        let mut removed = 0;
        while removed < 4 && line.starts_with(' ') {
            line.remove(0);
            removed += 1;
        }
        if removed > 0 {
            self.cursor_col = self.cursor_col.saturating_sub(removed);
            self.modified = true;
        }
    }
    
    /// Delete word forward (dw)
    pub fn delete_word(&mut self) {
        self.save_undo();
        let start_col = self.cursor_col;
        let start_row = self.cursor_row;
        self.move_word_forward();
        let end_col = self.cursor_col;
        
        if self.cursor_row == start_row { // Same line
            let chars: Vec<char> = self.lines[self.cursor_row].chars().collect();
            if end_col > start_col {
                let deleted: String = chars[start_col..end_col].iter().collect();
                self.clipboard = vec![deleted];
                self.clipboard_is_line = false;
                self.lines[self.cursor_row] = format!("{}{}", 
                    chars[..start_col].iter().collect::<String>(),
                    chars[end_col..].iter().collect::<String>()
                );
                self.cursor_col = start_col;
                self.modified = true;
            }
        }
    }
    
    // Search operations
    
    /// Start forward search (/)
    pub fn start_search_forward(&mut self) {
        self.mode = VimMode::Command;
        self.search_direction = true;
        self.command_buffer.clear();
        self.status_message = "/".to_string();
    }
    
    /// Start backward search (?)
    pub fn start_search_backward(&mut self) {
        self.mode = VimMode::Command;
        self.search_direction = false;
        self.command_buffer.clear();
        self.status_message = "?".to_string();
    }
    
    /// Execute search
    pub fn execute_search(&mut self) {
        if self.command_buffer.is_empty() {
            self.enter_normal_mode();
            return;
        }
        
        self.search_pattern = self.command_buffer.clone();
        self.last_search_row = self.cursor_row;
        self.last_search_col = self.cursor_col;
        
        if self.search_direction {
            self.search_next();
        } else {
            self.search_prev();
        }
        self.enter_normal_mode();
    }
    
    /// Search next occurrence (n)
    pub fn search_next(&mut self) {
        if self.search_pattern.is_empty() {
            self.status_message = "No search pattern".to_string();
            return;
        }
        
        let start_row = self.cursor_row;
        let start_col = self.cursor_col + 1;
        
        // Search from current position to end
        for row in start_row..self.lines.len() {
            let line = &self.lines[row];
            let search_start = if row == start_row { 
                char_to_byte_index(line, start_col.min(char_count(line)))
            } else { 
                0 
            };
            
            if let Some(byte_pos) = line[search_start..].find(&self.search_pattern) {
                let actual_byte_pos = search_start + byte_pos;
                self.cursor_row = row;
                self.cursor_col = line[..actual_byte_pos].chars().count();
                self.status_message = format!("/{}", self.search_pattern);
                return;
            }
        }
        
        // Wrap around
        for row in 0..=start_row {
            let line = &self.lines[row];
            let search_end = if row == start_row {
                char_to_byte_index(line, self.cursor_col)
            } else {
                line.len()
            };
            
            if let Some(byte_pos) = line[..search_end].find(&self.search_pattern) {
                self.cursor_row = row;
                self.cursor_col = line[..byte_pos].chars().count();
                self.status_message = format!("/{} (wrapped)", self.search_pattern);
                return;
            }
        }
        
        self.status_message = format!("Pattern not found: {}", self.search_pattern);
    }
    
    /// Search previous occurrence (N)
    pub fn search_prev(&mut self) {
        if self.search_pattern.is_empty() {
            self.status_message = "No search pattern".to_string();
            return;
        }
        
        let start_row = self.cursor_row;
        let start_col = self.cursor_col;
        
        // Search backward from current position
        for row in (0..=start_row).rev() {
            let line = &self.lines[row];
            let search_end = if row == start_row {
                char_to_byte_index(line, start_col)
            } else {
                line.len()
            };
            
            if let Some(byte_pos) = line[..search_end].rfind(&self.search_pattern) {
                self.cursor_row = row;
                self.cursor_col = line[..byte_pos].chars().count();
                self.status_message = format!("?{}", self.search_pattern);
                return;
            }
        }
        
        // Wrap around
        for row in (start_row..self.lines.len()).rev() {
            let line = &self.lines[row];
            let search_start = if row == start_row {
                char_to_byte_index(line, start_col + 1)
            } else {
                0
            };
            
            if search_start < line.len() {
                if let Some(byte_pos) = line[search_start..].rfind(&self.search_pattern) {
                    let actual_byte_pos = search_start + byte_pos;
                    self.cursor_row = row;
                    self.cursor_col = line[..actual_byte_pos].chars().count();
                    self.status_message = format!("?{} (wrapped)", self.search_pattern);
                    return;
                }
            }
        }
        
        self.status_message = format!("Pattern not found: {}", self.search_pattern);
    }
    
    /// Search word under cursor (*)
    pub fn search_word_under_cursor(&mut self) {
        let line = self.get_current_line();
        let chars: Vec<char> = line.chars().collect();
        
        if self.cursor_col >= chars.len() {
            return;
        }
        
        // Find word boundaries
        let mut start = self.cursor_col;
        let mut end = self.cursor_col;
        
        while start > 0 && is_word_char(chars[start - 1]) {
            start -= 1;
        }
        while end < chars.len() && is_word_char(chars[end]) {
            end += 1;
        }
        
        if start < end {
            let word: String = chars[start..end].iter().collect();
            self.search_pattern = word;
            self.search_direction = true;
            self.search_next();
        }
    }

    // Insert mode commands
    pub fn insert_char(&mut self, c: char) {
        let cursor_col = self.cursor_col;
        let line = self.get_current_line_mut();
        insert_char_at(line, cursor_col, c);
        self.cursor_col += 1;
        self.modified = true;
    }

    pub fn insert_newline(&mut self) {
        let cursor_col = self.cursor_col;
        let line = self.get_current_line_mut();
        let rest = split_off_at_char(line, cursor_col);
        self.lines.insert(self.cursor_row + 1, rest);
        self.cursor_row += 1;
        self.cursor_col = 0;
        self.modified = true;
    }

    pub fn backspace(&mut self) {
        if self.cursor_col > 0 {
            let cursor_col = self.cursor_col;
            let line = self.get_current_line_mut();
            remove_char_at(line, cursor_col - 1);
            self.cursor_col -= 1;
            self.modified = true;
        } else if self.cursor_row > 0 {
            // Join with previous line
            let current_line = self.lines.remove(self.cursor_row);
            self.cursor_row -= 1;
            self.cursor_col = char_count(&self.lines[self.cursor_row]);
            self.lines[self.cursor_row].push_str(&current_line);
            self.modified = true;
        }
    }

    // Mode transitions
    pub fn enter_insert_mode(&mut self) {
        self.mode = VimMode::Insert;
        self.pending_op = PendingOperator::None;
        self.count_buffer.clear();
        self.status_message = "-- INSERT --".to_string();
    }

    pub fn enter_normal_mode(&mut self) {
        self.mode = VimMode::Normal;
        self.pending_op = PendingOperator::None;
        self.count_buffer.clear();
        self.command_buffer.clear();
        self.status_message = "-- NORMAL --".to_string();
        self.clamp_cursor_col();
    }

    pub fn enter_command_mode(&mut self) {
        self.mode = VimMode::Command;
        self.pending_op = PendingOperator::None;
        self.count_buffer.clear();
        self.command_buffer.clear();
        self.status_message = ":".to_string();
    }
    
    pub fn enter_visual_mode(&mut self) {
        self.mode = VimMode::Visual;
        self.visual_start_row = self.cursor_row;
        self.visual_start_col = self.cursor_col;
        self.pending_op = PendingOperator::None;
        self.count_buffer.clear();
        self.status_message = "-- VISUAL --".to_string();
    }
    
    pub fn enter_visual_line_mode(&mut self) {
        self.mode = VimMode::VisualLine;
        self.visual_start_row = self.cursor_row;
        self.visual_start_col = 0;
        self.pending_op = PendingOperator::None;
        self.count_buffer.clear();
        self.status_message = "-- VISUAL LINE --".to_string();
    }
    
    /// Get visual selection range (start_row, start_col, end_row, end_col)
    pub fn get_visual_selection(&self) -> (usize, usize, usize, usize) {
        let (sr, sc, er, ec) = if self.cursor_row < self.visual_start_row 
            || (self.cursor_row == self.visual_start_row && self.cursor_col < self.visual_start_col) {
            (self.cursor_row, self.cursor_col, self.visual_start_row, self.visual_start_col)
        } else {
            (self.visual_start_row, self.visual_start_col, self.cursor_row, self.cursor_col)
        };
        
        if self.mode == VimMode::VisualLine {
            (sr, 0, er, char_count(&self.lines[er]))
        } else {
            (sr, sc, er, ec)
        }
    }
    
    /// Delete visual selection
    pub fn delete_visual_selection(&mut self) {
        self.save_undo();
        let (sr, sc, er, ec) = self.get_visual_selection();
        
        if self.mode == VimMode::VisualLine {
            self.clipboard = self.lines.drain(sr..=er).collect();
            self.clipboard_is_line = true;
            if self.lines.is_empty() {
                self.lines.push(String::new());
            }
        } else if sr == er {
            // Single line selection
            let line = &mut self.lines[sr];
            let chars: Vec<char> = line.chars().collect();
            let deleted: String = chars[sc..=ec.min(chars.len().saturating_sub(1))].iter().collect();
            self.clipboard = vec![deleted];
            self.clipboard_is_line = false;
            *line = format!("{}{}",
                chars[..sc].iter().collect::<String>(),
                chars[(ec + 1).min(chars.len())..].iter().collect::<String>()
            );
        } else {
            // Multi-line selection
            let mut selected_text = Vec::new();
            
            // First line partial
            let first_chars: Vec<char> = self.lines[sr].chars().collect();
            selected_text.push(first_chars[sc..].iter().collect());
            
            // Middle lines
            for row in (sr + 1)..er {
                selected_text.push(self.lines[row].clone());
            }
            
            // Last line partial
            let last_chars: Vec<char> = self.lines[er].chars().collect();
            selected_text.push(last_chars[..=ec.min(last_chars.len().saturating_sub(1))].iter().collect());
            
            self.clipboard = selected_text;
            self.clipboard_is_line = false;
            
            // Merge first and last lines
            let remaining: String = last_chars[(ec + 1).min(last_chars.len())..].iter().collect();
            self.lines[sr] = format!("{}{}", first_chars[..sc].iter().collect::<String>(), remaining);
            
            // Remove middle lines
            for _ in (sr + 1)..=er {
                if sr + 1 < self.lines.len() {
                    self.lines.remove(sr + 1);
                }
            }
        }
        
        self.cursor_row = sr.min(self.lines.len().saturating_sub(1));
        self.cursor_col = sc;
        self.modified = true;
        self.enter_normal_mode();
    }
    
    /// Yank visual selection
    pub fn yank_visual_selection(&mut self) {
        let (sr, sc, er, ec) = self.get_visual_selection();
        
        if self.mode == VimMode::VisualLine {
            self.clipboard = self.lines[sr..=er].to_vec();
            self.clipboard_is_line = true;
            self.status_message = format!("{} lines yanked", self.clipboard.len());
        } else if sr == er {
            let line = &self.lines[sr];
            let chars: Vec<char> = line.chars().collect();
            let yanked: String = chars[sc..=ec.min(chars.len().saturating_sub(1))].iter().collect();
            self.clipboard = vec![yanked];
            self.clipboard_is_line = false;
            self.status_message = "Yanked".to_string();
        } else {
            let mut selected_text = Vec::new();
            let first_chars: Vec<char> = self.lines[sr].chars().collect();
            selected_text.push(first_chars[sc..].iter().collect());
            for row in (sr + 1)..er {
                selected_text.push(self.lines[row].clone());
            }
            let last_chars: Vec<char> = self.lines[er].chars().collect();
            selected_text.push(last_chars[..=ec.min(last_chars.len().saturating_sub(1))].iter().collect());
            self.clipboard = selected_text;
            self.clipboard_is_line = false;
            self.status_message = "Yanked".to_string();
        }
        
        self.cursor_row = sr;
        self.cursor_col = sc;
        self.enter_normal_mode();
    }

    pub fn append_command_char(&mut self, c: char) {
        self.command_buffer.push(c);
        if self.status_message.starts_with('/') || self.status_message.starts_with('?') {
            let prefix = self.status_message.chars().next().unwrap();
            self.status_message = format!("{}{}", prefix, self.command_buffer);
        } else {
            self.status_message = format!(":{}", self.command_buffer);
        }
    }

    pub fn backspace_command(&mut self) {
        self.command_buffer.pop();
        if self.status_message.starts_with('/') || self.status_message.starts_with('?') {
            let prefix = self.status_message.chars().next().unwrap();
            self.status_message = format!("{}{}", prefix, self.command_buffer);
        } else {
            self.status_message = format!(":{}", self.command_buffer);
        }
    }
    
    /// Tilde - toggle case of character under cursor
    pub fn toggle_case(&mut self) {
        self.save_undo();
        let cursor_col = self.cursor_col;
        let line = self.get_current_line_mut();
        let chars: Vec<char> = line.chars().collect();
        
        if cursor_col < chars.len() {
            let c = chars[cursor_col];
            let toggled = if c.is_lowercase() {
                c.to_uppercase().next().unwrap_or(c)
            } else {
                c.to_lowercase().next().unwrap_or(c)
            };
            
            *line = format!("{}{}{}",
                chars[..cursor_col].iter().collect::<String>(),
                toggled,
                chars[cursor_col + 1..].iter().collect::<String>()
            );
            self.cursor_col = (cursor_col + 1).min(char_count(line).saturating_sub(1));
            self.modified = true;
        }
    }
    
    /// Get clipboard content as text (for system clipboard copy)
    pub fn get_clipboard_text(&self) -> String {
        if self.clipboard.is_empty() {
            String::new()
        } else if self.clipboard_is_line {
            self.clipboard.join("\n")
        } else {
            self.clipboard.join("")
        }
    }
    
    /// Get visual selection as text (for copying)
    pub fn get_visual_selection_text(&self) -> String {
        if self.mode != VimMode::Visual && self.mode != VimMode::VisualLine {
            return String::new();
        }
        
        let (sr, sc, er, ec) = self.get_visual_selection();
        
        if self.mode == VimMode::VisualLine {
            self.lines[sr..=er].join("\n")
        } else if sr == er {
            let line = &self.lines[sr];
            let chars: Vec<char> = line.chars().collect();
            chars[sc..=ec.min(chars.len().saturating_sub(1))].iter().collect()
        } else {
            let mut result = Vec::new();
            // First line from sc to end
            let first_line = &self.lines[sr];
            let first_chars: Vec<char> = first_line.chars().collect();
            result.push(first_chars[sc..].iter().collect::<String>());
            // Middle lines (full)
            for row in (sr + 1)..er {
                result.push(self.lines[row].clone());
            }
            // Last line from start to ec
            let last_line = &self.lines[er];
            let last_chars: Vec<char> = last_line.chars().collect();
            result.push(last_chars[..=ec.min(last_chars.len().saturating_sub(1))].iter().collect());
            result.join("\n")
        }
    }
}

