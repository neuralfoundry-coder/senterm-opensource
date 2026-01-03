//! Structured Logging System for Senterm
//!
//! Features:
//! - JSON format for log analysis
//! - Daily log file rotation (logs/YYYY-MM-DD.log)
//! - Automatic cleanup of logs older than 90 days (3 months)
//! - Structured fields for filtering and searching

use chrono::{Local, Duration};
use std::fs;
use std::path::{Path, PathBuf};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Log retention period in days (3 months)
const LOG_RETENTION_DAYS: i64 = 90;

/// Initialize the logging system
/// 
/// Creates logs in: logs/YYYY-MM-DD.log
/// Returns a guard that must be kept alive for logging to work
pub fn init() -> WorkerGuard {
    let log_dir = get_log_directory();

    // Ensure logs directory exists
    if !log_dir.exists() {
        let _ = fs::create_dir_all(&log_dir);
    }

    // Cleanup old logs on startup
    cleanup_old_logs(&log_dir, LOG_RETENTION_DAYS);

    // Daily rolling file appender
    let file_appender = tracing_appender::rolling::daily(&log_dir, "senterm");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    // Build subscriber with JSON format
    tracing_subscriber::registry()
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("senterm=debug,warn"))
        )
        .with(
            fmt::layer()
                .with_writer(non_blocking)
                .with_ansi(false)
                .json()  // JSON structured format
                .with_current_span(true)
                .with_span_list(true)
                .with_file(true)
                .with_line_number(true)
                .with_thread_ids(true)
                .with_thread_names(true)
                .flatten_event(true)
        )
        .init();

    tracing::info!(
        event = "logging_initialized",
        log_dir = %log_dir.display(),
        retention_days = LOG_RETENTION_DAYS,
        "Logging system initialized"
    );

    guard
}

/// Get the log directory path
/// Uses logs/ in the current directory
fn get_log_directory() -> PathBuf {
    // Try to use config dir, fallback to current dir
    if let Some(config_dir) = dirs::config_dir() {
        let senterm_logs = config_dir.join("senterm").join("logs");
        if fs::create_dir_all(&senterm_logs).is_ok() {
            return senterm_logs;
        }
    }
    
    PathBuf::from("logs")
}

/// Clean up log files older than the specified retention period
fn cleanup_old_logs(log_dir: &Path, retention_days: i64) {
    let cutoff_date = Local::now() - Duration::days(retention_days);
    
    let entries = match fs::read_dir(log_dir) {
        Ok(entries) => entries,
        Err(e) => {
            eprintln!("Failed to read log directory: {}", e);
            return;
        }
    };

    let mut deleted_count = 0;
    let mut deleted_bytes = 0u64;

    for entry in entries.flatten() {
        let path = entry.path();
        
        // Only process .log files
        if path.extension().map(|e| e != "log").unwrap_or(true) {
            continue;
        }

        // Check file modification time
        if let Ok(metadata) = fs::metadata(&path) {
            if let Ok(modified) = metadata.modified() {
                let modified_time: chrono::DateTime<Local> = modified.into();
                
                if modified_time < cutoff_date {
                    let file_size = metadata.len();
                    if fs::remove_file(&path).is_ok() {
                        deleted_count += 1;
                        deleted_bytes += file_size;
                    }
                }
            }
        }
    }

    if deleted_count > 0 {
        eprintln!(
            "Cleaned up {} old log file(s), freed {} bytes",
            deleted_count,
            deleted_bytes
        );
    }
}

/// Log helper macros and utilities
pub mod macros {
    /// Log a tool call with structured fields
    #[macro_export]
    macro_rules! log_tool_call {
        ($tool:expr, $status:expr, $($field:tt)*) => {
            tracing::info!(
                category = "tool",
                tool = $tool,
                status = $status,
                $($field)*
            )
        };
    }

    /// Log a user input event
    #[macro_export]
    macro_rules! log_input {
        ($input_type:expr, $($field:tt)*) => {
            tracing::debug!(
                category = "input",
                input_type = $input_type,
                $($field)*
            )
        };
    }

    /// Log a permission event
    #[macro_export]
    macro_rules! log_permission {
        ($action:expr, $result:expr, $($field:tt)*) => {
            tracing::info!(
                category = "permission",
                action = $action,
                result = $result,
                $($field)*
            )
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_cleanup_old_logs_does_not_delete_recent() {
        let temp_dir = tempdir().unwrap();
        let log_dir = temp_dir.path();

        // Create a recent log file (just created, so it's recent)
        let recent_log = log_dir.join("senterm.2024-12-16.log");
        let mut file = File::create(&recent_log).unwrap();
        writeln!(file, "recent log content").unwrap();

        // Run cleanup with 90 days retention
        // Since the file was just created, it should NOT be deleted
        cleanup_old_logs(log_dir, 90);

        // Recent log should remain (was just created)
        assert!(recent_log.exists());
    }

    #[test]
    fn test_cleanup_old_logs_ignores_non_log_files() {
        let temp_dir = tempdir().unwrap();
        let log_dir = temp_dir.path();

        // Create a non-log file
        let other_file = log_dir.join("config.toml");
        let mut file = File::create(&other_file).unwrap();
        writeln!(file, "config content").unwrap();

        // Run cleanup with 0 days retention
        cleanup_old_logs(log_dir, 0);

        // Non-log file should remain
        assert!(other_file.exists());
    }

    #[test]
    fn test_get_log_directory() {
        let log_dir = get_log_directory();
        // Should return a valid path
        assert!(!log_dir.as_os_str().is_empty());
    }
}
