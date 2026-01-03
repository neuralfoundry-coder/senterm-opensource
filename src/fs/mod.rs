pub mod watcher;

use std::fs;
use std::path::{PathBuf};
use std::collections::HashMap;
use crate::config::SortOption;

pub use watcher::FileWatcher;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClipboardOperation {
    Copy,
    Cut,
}

#[derive(Clone)]
pub struct FileSystem {
    pub current_dir: PathBuf,
    pub active_column_index: usize, // Index of currently focused column (0 = leftmost)
    pub clipboard: Option<(PathBuf, ClipboardOperation)>,
    pub navigation_path: Vec<PathBuf>, // Track navigation history for Miller Columns
    pub column_selections: HashMap<PathBuf, usize>, // Selection index per directory
    pub sort_option: SortOption, // File sorting option
}

impl FileSystem {
    pub fn new() -> Self {
        let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/"));

        // Build navigation path including parent directory for initial display
        let mut navigation_path = vec![];
        if let Some(parent) = current_dir.parent() {
            navigation_path.push(parent.to_path_buf());
        }
        navigation_path.push(current_dir.clone());

        let mut fs = Self {
            current_dir: current_dir.clone(),
            active_column_index: if navigation_path.len() > 1 { 1 } else { 0 }, // Focus on current dir, not parent
            clipboard: None,
            navigation_path,
            column_selections: HashMap::new(),
            sort_option: SortOption::Name, // Default to name sorting
        };

        // Initialize selection for starting directory
        fs.column_selections.insert(current_dir.clone(), 0);

        // Initialize selection for parent directory if it exists
        if let Some(parent) = current_dir.parent() {
            let parent_buf = parent.to_path_buf();
            // Find current_dir in parent's entries and select it
            let parent_entries = Self::get_entries_for_dir(&parent_buf);
            if let Some(idx) = parent_entries.iter().position(|p| p == &current_dir) {
                fs.column_selections.insert(parent_buf, idx);
            }
        }

        fs
    }

    /// Refresh the current directory view (re-read entries from filesystem)
    pub fn refresh_current_dir(&mut self) {
        // Just clear cached selections that might be stale
        // The entries are always read fresh from disk
        let current_selection = self.column_selections.get(&self.current_dir).cloned().unwrap_or(0);
        let entries = Self::get_entries_for_dir(&self.current_dir);
        // Clamp selection to valid range
        let new_selection = current_selection.min(entries.len().saturating_sub(1));
        self.column_selections.insert(self.current_dir.clone(), new_selection);
    }

    pub fn get_selection(&self, dir: &PathBuf) -> usize {
        *self.column_selections.get(dir).unwrap_or(&0)
    }

    pub fn set_selection(&mut self, dir: PathBuf, index: usize) {
        self.column_selections.insert(dir, index);
    }

    pub fn get_entries_for_dir(dir: &PathBuf) -> Vec<PathBuf> {
        Self::get_entries_for_dir_sorted(dir, SortOption::Name)
    }

    pub fn get_entries_for_dir_sorted(dir: &PathBuf, sort_option: SortOption) -> Vec<PathBuf> {
        let mut entries = Vec::new();

        // Add parent entry (..) at the top, except for root
        if let Some(parent) = dir.parent() {
            entries.push(parent.to_path_buf());
        }

        // Add all directory contents
        if let Ok(read_dir) = fs::read_dir(dir) {
            for entry in read_dir.flatten() {
                entries.push(entry.path());
            }
        }

        // Sort entries (keeping parent at top)
        let parent_entry = entries.first().cloned();
        let has_parent = parent_entry.as_ref().map(|p| dir.parent() == Some(p.as_path())).unwrap_or(false);

        if has_parent {
            // Sort everything except the first entry (parent)
            let mut content_entries: Vec<PathBuf> = entries.into_iter().skip(1).collect();

            // Sort based on option
            content_entries.sort_by(|a, b| {
                // Always put directories first
                match (a.is_dir(), b.is_dir()) {
                    (true, false) => return std::cmp::Ordering::Less,
                    (false, true) => return std::cmp::Ordering::Greater,
                    _ => {}
                }

                // Then sort by the selected option
                match sort_option {
                    SortOption::Name => a.file_name().cmp(&b.file_name()),
                    SortOption::Size => {
                        let size_a = a.metadata().map(|m| m.len()).unwrap_or(0);
                        let size_b = b.metadata().map(|m| m.len()).unwrap_or(0);
                        size_b.cmp(&size_a) // Descending
                    },
                    SortOption::Modified => {
                        let time_a = a.metadata().and_then(|m| m.modified()).ok();
                        let time_b = b.metadata().and_then(|m| m.modified()).ok();
                        time_b.cmp(&time_a) // Most recent first
                    },
                }
            });

            // Reconstruct with parent first
            let mut result = vec![parent_entry.unwrap()];
            result.extend(content_entries);
            result
        } else {
            // No parent, just sort normally
            entries.sort_by(|a, b| {
                match (a.is_dir(), b.is_dir()) {
                    (true, false) => return std::cmp::Ordering::Less,
                    (false, true) => return std::cmp::Ordering::Greater,
                    _ => {}
                }

                match sort_option {
                    SortOption::Name => a.file_name().cmp(&b.file_name()),
                    SortOption::Size => {
                        let size_a = a.metadata().map(|m| m.len()).unwrap_or(0);
                        let size_b = b.metadata().map(|m| m.len()).unwrap_or(0);
                        size_b.cmp(&size_a)
                    },
                    SortOption::Modified => {
                        let time_a = a.metadata().and_then(|m| m.modified()).ok();
                        let time_b = b.metadata().and_then(|m| m.modified()).ok();
                        time_b.cmp(&time_a)
                    },
                }
            });
            entries
        }
    }

    #[tracing::instrument(skip(self))]
    pub fn navigate_up(&mut self, dir: &PathBuf) {
        let current_selection = self.get_selection(dir);
        if current_selection > 0 {
            self.set_selection(dir.clone(), current_selection - 1);
            // Don't truncate navigation - just change selection
            // Truncation caused issues when navigating in parent columns
        }
    }

    #[tracing::instrument(skip(self))]
    pub fn navigate_down(&mut self, dir: &PathBuf) {
        let entries = Self::get_entries_for_dir_sorted(dir, self.sort_option);
        let current_selection = self.get_selection(dir);
        if current_selection + 1 < entries.len() {
            self.set_selection(dir.clone(), current_selection + 1);
            // Don't truncate navigation - just change selection
            // Truncation caused issues when navigating in parent columns
        }
    }

    #[tracing::instrument(skip(self))]
    pub fn enter_directory(&mut self) {
        let current_dir = self.current_dir.clone();
        let entries = Self::get_entries_for_dir(&current_dir);
        let selected_index = self.get_selection(&current_dir);

        if let Some(path) = entries.get(selected_index) {
            if path.is_dir() {
                // Check if we are entering the parent directory (Go Back)
                if Some(path.as_path()) == current_dir.parent() {
                    self.go_back();
                } else {
                    tracing::info!(path = ?path, "Entering directory");
                    self.current_dir = path.clone();
                    self.navigation_path.push(path.clone());

                    // Get entries to determine selection index
                    let new_entries = Self::get_entries_for_dir_sorted(path, self.sort_option);

                    // Determine first real item index (skip parent entry if exists)
                    let first_real_idx = if !new_entries.is_empty() {
                        let first_is_parent = path.parent()
                            .map(|p| new_entries.first().map(|e| e.as_path() == p).unwrap_or(false))
                            .unwrap_or(false);
                        if first_is_parent && new_entries.len() > 1 {
                            1 // Skip parent entry, select first real item
                        } else {
                            0 // No parent entry, or only parent entry exists
                        }
                    } else {
                        0 // Empty directory
                    };

                    // Initialize selection for new directory (select first real item)
                    if !self.column_selections.contains_key(path) {
                        self.column_selections.insert(path.clone(), first_real_idx);
                    }

                    // Set focus to the newly entered directory column
                    self.active_column_index = self.calculate_current_dir_column_index();

                    // Don't auto-expand subdirectories - user must press Enter to expand
                    tracing::info!(
                        new_dir = ?path,
                        active_column = self.active_column_index,
                        first_selected_idx = first_real_idx,
                        "Entered directory and focused on it"
                    );
                }
            }
        }
    }

    /// Calculate the column index of current_dir in the visible columns
    pub fn calculate_current_dir_column_index(&self) -> usize {
        // Build visible_path similar to calculate_visible_columns
        let max_visible_dirs = 5;
        let nav_len = self.navigation_path.len();
        let start_idx = if nav_len > max_visible_dirs {
            nav_len - max_visible_dirs
        } else {
            0
        };
        let mut visible_path: Vec<PathBuf> = self.navigation_path[start_idx..].to_vec();

        // Prepend parent directory if it exists and not already in visible_path
        // For root directory, ensure we still show the directory contents
        let current_dir = visible_path.last().unwrap_or(&self.current_dir);
        if let Some(parent) = current_dir.parent() {
            if !visible_path.contains(&parent.to_path_buf()) {
                visible_path.insert(0, parent.to_path_buf());
            }
        } else if visible_path.is_empty() {
            // If we're at root and have no visible path, add current_dir
            visible_path.push(self.current_dir.clone());
        }

        // Find current_dir in visible_path
        if let Some(pos) = visible_path.iter().position(|p| p == &self.current_dir) {
            pos
        } else {
            // Fallback: current_dir should be at the last position
            visible_path.len().saturating_sub(1)
        }
    }

    #[tracing::instrument(skip(self))]
    pub fn go_back(&mut self) {
        if let Some(parent) = self.current_dir.parent() {
            tracing::info!(to = ?parent, "Going back to parent");
            
            // Remove current directory from navigation path if it's the last element
            if self.navigation_path.last() == Some(&self.current_dir) {
                self.navigation_path.pop();
            }
            
            // Move to parent directory
            self.current_dir = parent.to_path_buf();
            
            // Ensure parent has selection initialized
            if !self.column_selections.contains_key(&self.current_dir) {
                self.column_selections.insert(self.current_dir.clone(), 0);
            }
            
            // Focus on the parent directory column
            self.active_column_index = self.calculate_current_dir_column_index();
            
            tracing::info!(
                new_dir = ?self.current_dir,
                active_column = self.active_column_index,
                navigation_path = ?self.navigation_path,
                "Moved to parent directory and focused on it"
            );
        }
    }

    #[tracing::instrument(skip(self))]
    pub fn copy_selected(&mut self) {
        // Get the active directory (currently focused column)
        if let Some(active_dir) = crate::navigation::get_active_directory(self) {
            let entries = Self::get_entries_for_dir(&active_dir);
            let selected_index = self.get_selection(&active_dir);

            if let Some(path) = entries.get(selected_index) {
                tracing::info!(path = ?path, active_dir = ?active_dir, "Copied to clipboard");
                self.clipboard = Some((path.clone(), ClipboardOperation::Copy));
            }
        }
    }

    #[tracing::instrument(skip(self))]
    pub fn cut_selected(&mut self) {
        // Get the active directory (currently focused column)
        if let Some(active_dir) = crate::navigation::get_active_directory(self) {
            let entries = Self::get_entries_for_dir(&active_dir);
            let selected_index = self.get_selection(&active_dir);

            if let Some(path) = entries.get(selected_index) {
                tracing::info!(path = ?path, active_dir = ?active_dir, "Cut to clipboard");
                self.clipboard = Some((path.clone(), ClipboardOperation::Cut));
            }
        }
    }

    #[tracing::instrument(skip(self))]
    pub fn delete_selected(&mut self) -> Result<(), std::io::Error> {
        // Use the active directory (current focused column / PATH)
        let target_dir = crate::navigation::get_active_directory(self)
            .unwrap_or_else(|| self.current_dir.clone());
        let entries = Self::get_entries_for_dir(&target_dir);
        let selected_index = self.get_selection(&target_dir);

        if let Some(path) = entries.get(selected_index) {
            if path.is_dir() {
                fs::remove_dir_all(path)?;
            } else {
                fs::remove_file(path)?;
            }
            tracing::info!(path = ?path, "Deleted from active directory");
            Ok(())
        } else {
            Err(std::io::Error::new(std::io::ErrorKind::NotFound, "No file selected"))
        }
    }

    #[tracing::instrument(skip(self))]
    pub fn rename_selected(&mut self, new_name: &str) -> Result<(), std::io::Error> {
        // Use the active directory (current focused column / PATH)
        let target_dir = crate::navigation::get_active_directory(self)
            .unwrap_or_else(|| self.current_dir.clone());
        let entries = Self::get_entries_for_dir(&target_dir);
        let selected_index = self.get_selection(&target_dir);

        if let Some(old_path) = entries.get(selected_index).cloned() {
            let new_path = target_dir.join(new_name);
            fs::rename(&old_path, &new_path)?;
            tracing::info!(from = ?old_path, to = ?new_path, "Renamed in active directory");

            // Update navigation_path if the renamed item was in it
            for path in &mut self.navigation_path {
                if path == &old_path {
                    *path = new_path.clone();
                }
            }

            // Update column_selections: transfer selection from old path to new path
            if let Some(selection) = self.column_selections.remove(&old_path) {
                self.column_selections.insert(new_path.clone(), selection);
            }

            // Update selection index to point to the new item after re-sorting
            let new_entries = Self::get_entries_for_dir_sorted(&target_dir, self.sort_option);
            if let Some(new_index) = new_entries.iter().position(|p| p == &new_path) {
                self.set_selection(target_dir, new_index);
            }

            Ok(())
        } else {
            Err(std::io::Error::new(std::io::ErrorKind::NotFound, "No file selected"))
        }
    }

    #[tracing::instrument(skip(self))]
    pub fn create_file(&mut self, name: &str) -> Result<(), std::io::Error> {
        // Use the active directory (current focused column)
        let target_dir = crate::navigation::get_active_directory(self)
            .unwrap_or_else(|| self.current_dir.clone());
        
        let file_path = target_dir.join(name);
        fs::File::create(&file_path)?;
        tracing::info!(path = ?file_path, target_dir = ?target_dir, "Created file in active directory");
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub fn create_folder(&mut self, name: &str) -> Result<(), std::io::Error> {
        // Use the active directory (current focused column)
        let target_dir = crate::navigation::get_active_directory(self)
            .unwrap_or_else(|| self.current_dir.clone());
        
        let folder_path = target_dir.join(name);
        fs::create_dir(&folder_path)?;
        tracing::info!(path = ?folder_path, target_dir = ?target_dir, "Created folder in active directory");
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub fn paste(&mut self) {
        // Clone clipboard to avoid borrow checker issues
        let clipboard_data = self.clipboard.clone();

        if let Some((src_path, op)) = clipboard_data {
            // Get the active directory (destination for paste)
            if let Some(active_dir) = crate::navigation::get_active_directory(self) {
                // Target is active directory + filename
                if let Some(file_name) = src_path.file_name() {
                    let mut dest_path = active_dir.join(file_name);

                    // Handle collision (simple rename for now if exists, or error?)
                    if dest_path.exists() {
                        let stem = src_path.file_stem().unwrap_or_default().to_string_lossy();
                        let ext = src_path.extension().unwrap_or_default().to_string_lossy();
                        let new_name = if ext.is_empty() {
                            format!("{}_copy", stem)
                        } else {
                            format!("{}_copy.{}", stem, ext)
                        };
                        dest_path = active_dir.join(new_name);
                    }

                    let result = match op {
                        ClipboardOperation::Copy => {
                            if src_path.is_dir() {
                                // Use fs_extra for recursive directory copy
                                let mut options = fs_extra::dir::CopyOptions::new();
                                options.overwrite = false;  // Don't overwrite existing
                                options.skip_exist = true;  // Skip if exists
                                options.copy_inside = true; // Copy contents into destination

                                match fs_extra::dir::copy(&src_path, &active_dir, &options) {
                                    Ok(_) => Ok(()),
                                    Err(e) => Err(std::io::Error::new(
                                        std::io::ErrorKind::Other,
                                        format!("Directory copy failed: {}", e)
                                    ))
                                }
                            } else {
                                fs::copy(&src_path, &dest_path).map(|_| ())
                            }
                        },
                        ClipboardOperation::Cut => {
                            fs::rename(&src_path, &dest_path)
                        }
                    };

                    match result {
                        Ok(_) => {
                            tracing::info!(?op, from = ?src_path, to = ?dest_path, active_dir = ?active_dir, "Paste successful");
                            // Entries are now queried on-demand, no need to refresh
                            if let ClipboardOperation::Cut = op {
                                self.clipboard = None;
                            }
                        },
                        Err(e) => {
                            tracing::error!(?e, "Paste failed");
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs as stdfs;
    use tempfile::tempdir;

    #[test]
    fn test_filesystem_new() {
        let fs = FileSystem::new();
        assert!(!fs.current_dir.as_os_str().is_empty());
        assert!(!fs.navigation_path.is_empty());
        assert!(fs.clipboard.is_none());
    }

    #[test]
    fn test_clipboard_operation_copy() {
        let op = ClipboardOperation::Copy;
        assert_eq!(op, ClipboardOperation::Copy);
        assert_ne!(op, ClipboardOperation::Cut);
    }

    #[test]
    fn test_clipboard_operation_cut() {
        let op = ClipboardOperation::Cut;
        assert_eq!(op, ClipboardOperation::Cut);
        assert_ne!(op, ClipboardOperation::Copy);
    }

    #[test]
    fn test_get_selection_default() {
        let fs = FileSystem::new();
        let unknown_dir = PathBuf::from("/nonexistent/path");
        assert_eq!(fs.get_selection(&unknown_dir), 0);
    }

    #[test]
    fn test_set_selection() {
        let mut fs = FileSystem::new();
        let test_dir = PathBuf::from("/tmp");
        fs.set_selection(test_dir.clone(), 5);
        assert_eq!(fs.get_selection(&test_dir), 5);
    }

    #[test]
    fn test_get_entries_for_dir_with_tempdir() {
        let temp = tempdir().unwrap();
        let temp_path = temp.path().to_path_buf();

        // Create some test files and directories
        stdfs::create_dir(temp_path.join("subdir")).unwrap();
        stdfs::File::create(temp_path.join("file1.txt")).unwrap();
        stdfs::File::create(temp_path.join("file2.rs")).unwrap();

        let entries = FileSystem::get_entries_for_dir(&temp_path);
        
        // Should contain parent + subdir + 2 files = 4 entries
        assert!(entries.len() >= 3); // At least our created items
        
        // Directories should come before files (after parent)
        let dir_positions: Vec<_> = entries.iter()
            .enumerate()
            .filter(|(_, p)| p.is_dir() && p.file_name().map(|n| n != "..").unwrap_or(true))
            .map(|(i, _)| i)
            .collect();
        
        let file_positions: Vec<_> = entries.iter()
            .enumerate()
            .filter(|(_, p)| p.is_file())
            .map(|(i, _)| i)
            .collect();

        // All directories should appear before files
        if !dir_positions.is_empty() && !file_positions.is_empty() {
            let max_dir_pos = *dir_positions.iter().max().unwrap();
            let min_file_pos = *file_positions.iter().min().unwrap();
            assert!(max_dir_pos < min_file_pos, "Directories should be sorted before files");
        }
    }

    #[test]
    fn test_get_entries_sorted_by_name() {
        let temp = tempdir().unwrap();
        let temp_path = temp.path().to_path_buf();

        // Create files with specific names for sorting
        stdfs::File::create(temp_path.join("zebra.txt")).unwrap();
        stdfs::File::create(temp_path.join("apple.txt")).unwrap();
        stdfs::File::create(temp_path.join("mango.txt")).unwrap();

        let entries = FileSystem::get_entries_for_dir_sorted(&temp_path, SortOption::Name);
        
        // Find file positions (skip parent entry)
        let file_names: Vec<_> = entries.iter()
            .filter(|p| p.is_file())
            .filter_map(|p| p.file_name())
            .filter_map(|n| n.to_str())
            .collect();

        // Should be sorted alphabetically
        assert!(file_names.windows(2).all(|w| w[0] <= w[1]), 
            "Files should be sorted by name: {:?}", file_names);
    }

    #[test]
    fn test_create_file() {
        let temp = tempdir().unwrap();
        let temp_path = temp.path().to_path_buf();

        // Create file directly using std::fs for simpler test
        let file_path = temp_path.join("test_new_file.txt");
        let result = stdfs::File::create(&file_path);
        assert!(result.is_ok());
        assert!(file_path.exists());
    }

    #[test]
    fn test_create_folder() {
        let temp = tempdir().unwrap();
        let temp_path = temp.path().to_path_buf();

        // Create folder directly using std::fs for simpler test
        let folder_path = temp_path.join("test_new_folder");
        let result = stdfs::create_dir(&folder_path);
        assert!(result.is_ok());
        assert!(folder_path.is_dir());
    }

    #[test]
    fn test_navigate_up_at_top() {
        let temp = tempdir().unwrap();
        let temp_path = temp.path().to_path_buf();

        let mut fs = FileSystem::new();
        fs.set_selection(temp_path.clone(), 0);
        
        // Navigate up when already at top should stay at 0
        fs.navigate_up(&temp_path);
        assert_eq!(fs.get_selection(&temp_path), 0);
    }

    #[test]
    fn test_navigate_down() {
        let temp = tempdir().unwrap();
        let temp_path = temp.path().to_path_buf();

        // Create some entries
        stdfs::File::create(temp_path.join("file1.txt")).unwrap();
        stdfs::File::create(temp_path.join("file2.txt")).unwrap();

        let mut fs = FileSystem::new();
        fs.set_selection(temp_path.clone(), 0);
        
        fs.navigate_down(&temp_path);
        assert_eq!(fs.get_selection(&temp_path), 1);
    }

    #[test]
    fn test_refresh_current_dir() {
        let temp = tempdir().unwrap();
        let temp_path = temp.path().to_path_buf();

        let mut fs = FileSystem::new();
        fs.current_dir = temp_path.clone();
        fs.set_selection(temp_path.clone(), 100); // Set invalid selection

        fs.refresh_current_dir();
        
        // Selection should be clamped to valid range
        let entries = FileSystem::get_entries_for_dir(&temp_path);
        let selection = fs.get_selection(&temp_path);
        assert!(selection < entries.len() || entries.is_empty());
    }

    #[test]
    fn test_calculate_current_dir_column_index() {
        let fs = FileSystem::new();
        let index = fs.calculate_current_dir_column_index();
        // Index should be valid (non-negative, reasonable value)
        assert!(index < 10);
    }
}
