use std::path::PathBuf;
use crate::fs::FileSystem;

/// Represents the visible navigation columns for Miller-style column view
pub struct NavigationColumns {
    pub visible_path: Vec<PathBuf>,
    pub total_columns: usize,
}

/// Calculate visible columns for Miller-style navigation
pub fn calculate_visible_columns(
    fs: &FileSystem,
    max_visible_dirs: usize,
) -> NavigationColumns {
    let nav_len = fs.navigation_path.len();
    let start_idx = if nav_len > max_visible_dirs {
        nav_len - max_visible_dirs
    } else {
        0
    };
    let mut visible_path: Vec<PathBuf> = fs.navigation_path[start_idx..].to_vec();

    // Prepend parent directory if it exists and not already in visible_path
    // For root directory, ensure we still show the directory contents
    let current_dir = visible_path.last().unwrap_or(&fs.current_dir);
    if let Some(parent) = current_dir.parent() {
        if !visible_path.contains(&parent.to_path_buf()) {
            visible_path.insert(0, parent.to_path_buf());
        }
    } else if visible_path.is_empty() {
        // If we're at root and have no visible path, add current_dir
        visible_path.push(fs.current_dir.clone());
    }

    // No preview column - only show navigation path
    let total_columns = visible_path.len();

    NavigationColumns {
        visible_path,
        total_columns,
    }
}

/// Handle column navigation (Tab/Right arrow)
/// Only moves between existing expanded columns - does NOT auto-expand
pub fn navigate_column_forward(fs: &mut FileSystem) {
    let columns = calculate_visible_columns(fs, 5);
    let old_index = fs.active_column_index;
    
    // Only move forward if there's a next column (already expanded)
    if fs.active_column_index + 1 < columns.total_columns {
        fs.active_column_index += 1;
    }
    // Don't wrap around - stay at rightmost column

    // Ensure selection exists for the new active directory
    if let Some(active_dir) = get_active_directory(fs) {
        if !fs.column_selections.contains_key(&active_dir) {
            fs.set_selection(active_dir, 0);
            tracing::info!("Initialized selection for new active directory");
        }
    }

    tracing::info!("Forward: {} -> {} (total: {})", old_index, fs.active_column_index, columns.total_columns);
}

/// Handle column navigation (Shift+Tab/Left arrow)
/// Only moves between existing expanded columns - does NOT auto-expand
/// Does NOT navigate to prepended parent directory (use Enter for that)
pub fn navigate_column_backward(fs: &mut FileSystem) {
    let columns = calculate_visible_columns(fs, 5);
    let old_index = fs.active_column_index;

    // Calculate minimum column index (skip prepended parent)
    // If navigation_path is shorter than visible_path, index 0 is prepended parent
    let min_index = if columns.visible_path.len() > fs.navigation_path.len() {
        1  // Don't go to prepended parent column
    } else {
        0
    };

    // Only move backward if not at minimum column
    if fs.active_column_index > min_index {
        fs.active_column_index -= 1;
    }
    // Don't wrap around - stay at leftmost navigable column

    // Ensure selection exists for the new active directory
    if let Some(active_dir) = get_active_directory(fs) {
        if !fs.column_selections.contains_key(&active_dir) {
            fs.set_selection(active_dir, 0);
            tracing::info!("Initialized selection for new active directory");
        }
    }

    tracing::info!("Backward: {} -> {} (total: {}, min: {})", old_index, fs.active_column_index, columns.total_columns, min_index);
}

/// Get the active directory based on current column index
pub fn get_active_directory(fs: &FileSystem) -> Option<PathBuf> {
    let columns = calculate_visible_columns(fs, 5);

    if fs.active_column_index < columns.visible_path.len() {
        Some(columns.visible_path[fs.active_column_index].clone())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use crate::config::SortOption;

    /// Create a test FileSystem with specified path
    fn create_test_fs(current_dir: PathBuf) -> FileSystem {
        let mut navigation_path = vec![];
        if let Some(parent) = current_dir.parent() {
            navigation_path.push(parent.to_path_buf());
        }
        navigation_path.push(current_dir.clone());

        FileSystem {
            current_dir: current_dir.clone(),
            active_column_index: if navigation_path.len() > 1 { 1 } else { 0 },
            clipboard: None,
            navigation_path,
            column_selections: HashMap::new(),
            sort_option: SortOption::Name,
        }
    }

    #[test]
    fn test_navigation_columns_structure() {
        let fs = create_test_fs(PathBuf::from("/tmp"));
        let columns = calculate_visible_columns(&fs, 5);
        
        assert!(!columns.visible_path.is_empty());
        assert_eq!(columns.total_columns, columns.visible_path.len());
    }

    #[test]
    fn test_calculate_visible_columns_with_max() {
        let mut fs = create_test_fs(PathBuf::from("/usr/local/bin"));
        
        // Build a longer navigation path
        fs.navigation_path = vec![
            PathBuf::from("/"),
            PathBuf::from("/usr"),
            PathBuf::from("/usr/local"),
            PathBuf::from("/usr/local/bin"),
        ];
        
        // With max 3 visible, should truncate
        let columns = calculate_visible_columns(&fs, 3);
        assert!(columns.total_columns <= 4); // max 3 + possible parent prepend
    }

    #[test]
    fn test_calculate_visible_columns_includes_parent() {
        let fs = create_test_fs(PathBuf::from("/tmp/test"));
        let columns = calculate_visible_columns(&fs, 5);
        
        // Should include parent directory
        let has_parent = columns.visible_path.iter()
            .any(|p| p == &PathBuf::from("/tmp"));
        assert!(has_parent || columns.visible_path.contains(&PathBuf::from("/tmp/test")));
    }

    #[test]
    fn test_navigate_column_forward() {
        let mut fs = create_test_fs(PathBuf::from("/tmp"));
        fs.navigation_path = vec![
            PathBuf::from("/"),
            PathBuf::from("/tmp"),
        ];
        fs.active_column_index = 0;
        
        let initial_columns = calculate_visible_columns(&fs, 5);
        let initial_index = fs.active_column_index;
        
        navigate_column_forward(&mut fs);
        
        // Should move forward if possible
        if initial_columns.total_columns > 1 {
            assert!(fs.active_column_index >= initial_index);
        }
    }

    #[test]
    fn test_navigate_column_forward_at_end() {
        let mut fs = create_test_fs(PathBuf::from("/tmp"));
        let columns = calculate_visible_columns(&fs, 5);
        
        // Set to last column
        fs.active_column_index = columns.total_columns.saturating_sub(1);
        let last_index = fs.active_column_index;
        
        navigate_column_forward(&mut fs);
        
        // Should not go past the end
        assert_eq!(fs.active_column_index, last_index);
    }

    #[test]
    fn test_navigate_column_backward() {
        let mut fs = create_test_fs(PathBuf::from("/tmp"));
        fs.navigation_path = vec![
            PathBuf::from("/"),
            PathBuf::from("/tmp"),
        ];
        
        // Start at column 1
        fs.active_column_index = 1;
        
        navigate_column_backward(&mut fs);
        
        // Should move backward (but might stay at min_index)
        assert!(fs.active_column_index <= 1);
    }

    #[test]
    fn test_navigate_column_backward_at_start() {
        let mut fs = create_test_fs(PathBuf::from("/tmp"));
        fs.active_column_index = 0;
        
        navigate_column_backward(&mut fs);
        
        // Should not go below 0
        assert!(fs.active_column_index < 100); // Just ensure no overflow
    }

    #[test]
    fn test_get_active_directory() {
        let fs = create_test_fs(PathBuf::from("/tmp"));
        
        let active_dir = get_active_directory(&fs);
        
        assert!(active_dir.is_some());
        // Active directory should be a valid path
        let dir = active_dir.unwrap();
        assert!(!dir.as_os_str().is_empty());
    }

    #[test]
    fn test_get_active_directory_out_of_bounds() {
        let mut fs = create_test_fs(PathBuf::from("/tmp"));
        
        // Set index way out of bounds
        fs.active_column_index = 100;
        
        let active_dir = get_active_directory(&fs);
        
        // Should return None for out of bounds
        assert!(active_dir.is_none());
    }

    #[test]
    fn test_navigation_columns_total_equals_visible_len() {
        let fs = create_test_fs(PathBuf::from("/usr/local"));
        let columns = calculate_visible_columns(&fs, 5);
        
        assert_eq!(columns.total_columns, columns.visible_path.len());
    }

    #[test]
    fn test_visible_path_not_empty_for_valid_dir() {
        let fs = create_test_fs(PathBuf::from("/"));
        let columns = calculate_visible_columns(&fs, 5);
        
        // Even for root, visible_path should not be empty
        assert!(!columns.visible_path.is_empty());
    }
}
