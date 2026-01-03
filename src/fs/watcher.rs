//! File system watcher for real-time file change monitoring
//! 
//! Uses the `notify` crate to watch directories and detect changes,
//! allowing the UI to refresh in real-time when files are modified.

#![allow(dead_code)]

use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::time::{Duration, SystemTime};

/// File change event
#[derive(Debug, Clone)]
pub struct FileChange {
    pub path: PathBuf,
    pub kind: FileChangeKind,
    pub timestamp: SystemTime,
}

/// Kind of file change
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileChangeKind {
    Created,
    Modified,
    Deleted,
    Renamed,
    Other,
}

impl From<&EventKind> for FileChangeKind {
    fn from(kind: &EventKind) -> Self {
        match kind {
            EventKind::Create(_) => FileChangeKind::Created,
            EventKind::Modify(_) => FileChangeKind::Modified,
            EventKind::Remove(_) => FileChangeKind::Deleted,
            EventKind::Any => FileChangeKind::Other,
            EventKind::Access(_) => FileChangeKind::Other,
            EventKind::Other => FileChangeKind::Other,
        }
    }
}

/// File system watcher
pub struct FileWatcher {
    /// The notify watcher instance
    watcher: RecommendedWatcher,
    /// Channel receiver for file events
    rx: Receiver<Result<Event, notify::Error>>,
    /// Currently watched paths
    watched_paths: Vec<PathBuf>,
}

impl FileWatcher {
    /// Create a new file watcher
    pub fn new() -> Result<Self, notify::Error> {
        let (tx, rx) = mpsc::channel();
        
        let watcher = RecommendedWatcher::new(
            move |res| {
                let _ = tx.send(res);
            },
            Config::default()
                .with_poll_interval(Duration::from_millis(500))
                .with_compare_contents(false),
        )?;
        
        Ok(Self {
            watcher,
            rx,
            watched_paths: Vec::new(),
        })
    }
    
    /// Watch a directory for changes
    pub fn watch(&mut self, path: &PathBuf) -> Result<(), notify::Error> {
        // Avoid watching the same path twice
        if self.watched_paths.contains(path) {
            return Ok(());
        }
        
        self.watcher.watch(path.as_ref(), RecursiveMode::NonRecursive)?;
        self.watched_paths.push(path.clone());
        
        tracing::debug!("Started watching: {}", path.display());
        Ok(())
    }
    
    /// Watch a directory recursively
    pub fn watch_recursive(&mut self, path: &PathBuf) -> Result<(), notify::Error> {
        if self.watched_paths.contains(path) {
            return Ok(());
        }
        
        self.watcher.watch(path.as_ref(), RecursiveMode::Recursive)?;
        self.watched_paths.push(path.clone());
        
        tracing::debug!("Started watching recursively: {}", path.display());
        Ok(())
    }
    
    /// Stop watching a directory
    pub fn unwatch(&mut self, path: &PathBuf) -> Result<(), notify::Error> {
        self.watcher.unwatch(path.as_ref())?;
        self.watched_paths.retain(|p| p != path);
        
        tracing::debug!("Stopped watching: {}", path.display());
        Ok(())
    }
    
    /// Stop watching all directories
    pub fn unwatch_all(&mut self) {
        for path in self.watched_paths.drain(..) {
            let _ = self.watcher.unwatch(path.as_ref());
        }
    }
    
    /// Poll for file changes (non-blocking)
    pub fn poll_changes(&self) -> Vec<FileChange> {
        let mut changes = Vec::new();
        
        loop {
            match self.rx.try_recv() {
                Ok(Ok(event)) => {
                    let kind = FileChangeKind::from(&event.kind);
                    
                    for path in event.paths {
                        changes.push(FileChange {
                            path,
                            kind,
                            timestamp: SystemTime::now(),
                        });
                    }
                }
                Ok(Err(e)) => {
                    tracing::warn!("File watcher error: {}", e);
                }
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    tracing::warn!("File watcher channel disconnected");
                    break;
                }
            }
        }
        
        // Deduplicate changes (same path, same kind within short time)
        let mut deduped: Vec<FileChange> = Vec::new();
        for change in changes {
            let exists = deduped.iter().any(|c| c.path == change.path && c.kind == change.kind);
            if !exists {
                deduped.push(change);
            }
        }
        
        deduped
    }
    
    /// Get list of watched paths
    pub fn watched_paths(&self) -> &[PathBuf] {
        &self.watched_paths
    }
    
    /// Check if a path is being watched
    pub fn is_watching(&self, path: &PathBuf) -> bool {
        self.watched_paths.contains(path)
    }
}

impl Default for FileWatcher {
    fn default() -> Self {
        Self::new().expect("Failed to create file watcher")
    }
}

impl Drop for FileWatcher {
    fn drop(&mut self) {
        self.unwatch_all();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[test]
    fn test_watcher_creation() {
        let watcher = FileWatcher::new();
        assert!(watcher.is_ok());
    }
    
    #[test]
    fn test_watch_directory() {
        let dir = tempdir().unwrap();
        let mut watcher = FileWatcher::new().unwrap();
        
        let result = watcher.watch(&dir.path().to_path_buf());
        assert!(result.is_ok());
        assert!(watcher.is_watching(&dir.path().to_path_buf()));
    }
}

