use crate::fs::{FileSystem, FileWatcher};
use crate::system::SystemManager;
use crate::config::Config;
use std::path::PathBuf;
use std::time::Instant;
use std::sync::{Arc, Mutex};
use std::io::Write;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppMode {
    FileManager,
    SystemMonitor,
    #[allow(dead_code)]
    Setup, // Reserved for future setup/onboarding mode
    Settings,
    Viewer, // File viewer popup
}

/// Active pane in split mode (supports up to 3 panes)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pane {
    Left,
    Center,
    Right,
}

pub enum DialogMode {
    None,
    Rename { current_name: String, new_name: String },
    Delete { path_name: String },
    NewFile { name: String },
    NewFolder { name: String },
    Search { query: String, results: Vec<(PathBuf, usize)> }, // (파일 경로, 디렉토리 내 인덱스)
    Command { input: String }, // 명령어 모드 (:game, :help 등)
    QuitConfirm, // 종료 확인 다이얼로그 (ESC)
}

pub struct App {
    pub should_quit: bool,
    pub mode: AppMode,
    // Split pane file managers (up to 3 panes)
    pub fs_left: FileSystem,
    pub fs_center: FileSystem,
    pub fs_right: FileSystem,
    pub active_pane: Pane,
    pub pane_count: usize,  // 1, 2, or 3
    // Other managers
    pub system: SystemManager,
    pub config: Config,
    pub show_help: bool,
    pub show_bookmarks: bool,
    pub viewer_content: Option<crate::viewer::ViewerContent>,
    pub viewer_scroll: usize,
    pub viewer_editing: bool, // True when in vim edit mode
    pub text_editor: Option<crate::viewer::TextEditor>,
    pub dialog: DialogMode,
    pub status_message: Option<String>,
    pub temp_message: Option<(String, Instant)>, // Temporary message with timer (auto-dismiss after 0.5s)
    // Shell popup state (legacy popup mode)
    pub show_shell: bool,
    pub shell: ShellState,
    // Console panel state (right side panel - Shell)
    pub show_console: bool,
    pub console_focus: bool,
    pub console: ShellState,
    // File watcher for real-time updates (reserved for future use)
    #[allow(dead_code)]
    pub file_watcher: Option<FileWatcher>,
    // Process viewer popup state
    pub show_process_viewer: bool,
    pub process_viewer: crate::process::ProcessViewer,
    // External game launcher flag
    pub launch_external_game: bool,
    // Settings state
    pub settings_theme_index: usize,
    pub settings_tab: SettingsTab,
    // Viewer state
    pub viewer_wrap_mode: bool,
}

/// Settings tab
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SettingsTab {
    #[default]
    Theme,
    Interface,
}

/// PTY-based shell state for full terminal emulation
pub struct ShellState {
    pub master: Option<Box<dyn portable_pty::MasterPty + Send>>,
    pub writer: Option<Arc<Mutex<Box<dyn Write + Send>>>>,
    pub child: Option<Box<dyn portable_pty::Child + Send + Sync>>,
    pub parser: Arc<Mutex<vt100::Parser>>,
    pub working_dir: PathBuf,
    pub size: (u16, u16),  // (cols, rows)
    pub is_running: bool,
    // Background thread for non-blocking PTY reading
    pub output_receiver: Option<std::sync::mpsc::Receiver<Vec<u8>>>,
    pub reader_thread: Option<std::thread::JoinHandle<()>>,
}

impl std::fmt::Debug for ShellState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ShellState")
            .field("working_dir", &self.working_dir)
            .field("size", &self.size)
            .field("is_running", &self.is_running)
            .finish()
    }
}

impl ShellState {
    pub fn new(working_dir: PathBuf) -> Self {
        ShellState {
            master: None,
            writer: None,
            child: None,
            parser: Arc::new(Mutex::new(vt100::Parser::new(24, 80, 0))),
            working_dir,
            size: (80, 24),
            is_running: false,
            output_receiver: None,
            reader_thread: None,
        }
    }
    
    /// Start a new PTY session with shell
    pub fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        use portable_pty::{CommandBuilder, PtySize, native_pty_system};
        
        let pty_system = native_pty_system();
        
        let pair = pty_system.openpty(PtySize {
            rows: self.size.1,
            cols: self.size.0,
            pixel_width: 0,
            pixel_height: 0,
        })?;
        
        // Build shell command
        #[cfg(target_os = "windows")]
        let mut cmd = CommandBuilder::new("cmd.exe");
        
        #[cfg(not(target_os = "windows"))]
        let mut cmd = {
            // Try zsh first, fall back to bash, then sh
            let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());
            CommandBuilder::new(shell)
        };
        
        cmd.cwd(&self.working_dir);
        
        // Spawn the shell
        let child = pair.slave.spawn_command(cmd)?;
        
        // Get reader and writer from master
        let mut reader = pair.master.try_clone_reader()?;
        let writer = pair.master.take_writer()?;
        
        // Create channel for non-blocking communication
        let (tx, rx) = std::sync::mpsc::channel::<Vec<u8>>();
        
        // Spawn background thread to read from PTY
        let reader_thread = std::thread::spawn(move || {
            let mut buffer = [0u8; 4096];
            loop {
                match reader.read(&mut buffer) {
                    Ok(0) => break, // EOF - shell exited
                    Ok(n) => {
                        if tx.send(buffer[..n].to_vec()).is_err() {
                            break; // Receiver dropped
                        }
                    }
                    Err(_) => break, // Error reading
                }
            }
        });
        
        self.master = Some(pair.master);
        self.writer = Some(Arc::new(Mutex::new(writer)));
        self.child = Some(child);
        self.output_receiver = Some(rx);
        self.reader_thread = Some(reader_thread);
        self.is_running = true;
        
        // Reset parser with correct size
        *self.parser.lock().unwrap() = vt100::Parser::new(self.size.1, self.size.0, 0);
        
        Ok(())
    }
    
    /// Stop the PTY session
    pub fn stop(&mut self) {
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
        }
        self.master = None;
        self.writer = None;
        self.output_receiver = None;
        // Thread will exit when reader gets EOF or error after child is killed
        if let Some(handle) = self.reader_thread.take() {
            let _ = handle.join();
        }
        self.is_running = false;
    }
    
    /// Write data to PTY
    pub fn write(&mut self, data: &[u8]) -> std::io::Result<()> {
        if let Some(writer) = &self.writer {
            let mut writer = writer.lock().unwrap();
            writer.write_all(data)?;
            writer.flush()?;
        }
        Ok(())
    }
    
    /// Read available data from PTY and process through parser (non-blocking)
    pub fn read_and_parse(&mut self) -> std::io::Result<()> {
        if let Some(receiver) = &self.output_receiver {
            // Non-blocking: process all available data from channel
            while let Ok(data) = receiver.try_recv() {
                let mut parser = self.parser.lock().unwrap();
                parser.process(&data);
            }
        }
        Ok(())
    }
    
    /// Resize the PTY
    pub fn resize(&mut self, cols: u16, rows: u16) {
        self.size = (cols, rows);
        
        if let Some(master) = &self.master {
            let _ = master.resize(portable_pty::PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            });
        }
        
        let mut parser = self.parser.lock().unwrap();
        parser.set_size(rows, cols);
    }
    
    /// Check if child process is still running
    pub fn check_running(&mut self) -> bool {
        if let Some(child) = &mut self.child {
            match child.try_wait() {
                Ok(Some(_)) => {
                    // Process exited
                    self.is_running = false;
                }
                Ok(None) => {
                    // Still running
                }
                Err(_) => {
                    self.is_running = false;
                }
            }
        }
        self.is_running
    }
}

impl App {
    #[tracing::instrument]
    pub fn new() -> Self {
        tracing::info!("Initializing App state");
        let config = Config::load();

        let mut fs_left = FileSystem::new();
        fs_left.sort_option = config.sort_option;

        let mut fs_center = FileSystem::new();
        fs_center.sort_option = config.sort_option;

        let mut fs_right = FileSystem::new();
        fs_right.sort_option = config.sort_option;

        let current_dir = fs_left.current_dir.clone();
        App {
            mode: AppMode::FileManager,
            fs_left,
            fs_center,
            fs_right,
            active_pane: Pane::Left,
            pane_count: 1,
            system: SystemManager::new(),
            should_quit: false,
            config,
            show_help: false,
            show_bookmarks: false,
            viewer_content: None,
            viewer_scroll: 0,
            viewer_editing: false,
            text_editor: None,
            dialog: DialogMode::None,
            status_message: None,
            temp_message: None,
            show_shell: false,
            shell: ShellState::new(current_dir.clone()),
            show_console: false,
            console_focus: false,
            console: ShellState::new(current_dir.clone()),
            file_watcher: FileWatcher::new().ok(),
            show_process_viewer: false,
            process_viewer: crate::process::ProcessViewer::new(),
            launch_external_game: false,
            settings_theme_index: 0,
            settings_tab: SettingsTab::default(),
            viewer_wrap_mode: true,
        }
    }
    
    /// Get reference to the active file system
    pub fn active_fs(&self) -> &FileSystem {
        match self.active_pane {
            Pane::Left => &self.fs_left,
            Pane::Center => &self.fs_center,
            Pane::Right => &self.fs_right,
        }
    }
    
    /// Get mutable reference to the active file system
    pub fn active_fs_mut(&mut self) -> &mut FileSystem {
        match self.active_pane {
            Pane::Left => &mut self.fs_left,
            Pane::Center => &mut self.fs_center,
            Pane::Right => &mut self.fs_right,
        }
    }
    
    /// Refresh all file systems (after file operations)
    pub fn refresh_both_panes(&mut self) {
        self.fs_left.refresh_current_dir();
        self.fs_center.refresh_current_dir();
        self.fs_right.refresh_current_dir();
    }
    
    /// Add a pane (F3) - configurable max panes
    pub fn add_pane(&mut self) {
        let max_panes = self.config.max_ui_trees.min(10); // Cap at 10
        if self.pane_count < max_panes {
            self.pane_count += 1;
            self.status_message = Some(format!(
                "{} panes (F3:Add, F4:Remove, Tab/Ctrl+←→:Switch)",
                self.pane_count
            ));
        } else {
            self.status_message = Some(format!("Maximum {} panes", max_panes));
        }
    }
    
    /// Remove a pane (F4) - min 1 pane
    pub fn remove_pane(&mut self) {
        if self.pane_count > 1 {
            self.pane_count -= 1;
            // Adjust active pane if it no longer exists
            match (self.pane_count, self.active_pane) {
                (1, Pane::Center) | (1, Pane::Right) => self.active_pane = Pane::Left,
                (2, Pane::Right) => self.active_pane = Pane::Center,
                _ => {}
            }
            self.status_message = Some(format!(
                "{} pane{} (F3:Add, F4:Remove)",
                self.pane_count,
                if self.pane_count > 1 { "s" } else { "" }
            ));
        } else {
            self.status_message = Some("Minimum 1 pane".to_string());
        }
    }
    
    /// Switch to the next pane (Tab)
    pub fn switch_pane(&mut self) {
        if self.pane_count > 1 {
            self.active_pane = match (self.pane_count, self.active_pane) {
                (2, Pane::Left) => Pane::Center,
                (2, Pane::Center) => Pane::Left,
                (3, Pane::Left) => Pane::Center,
                (3, Pane::Center) => Pane::Right,
                (3, Pane::Right) => Pane::Left,
                _ => Pane::Left,
            };
        }
    }
    
    /// Switch to the previous pane (BackTab)
    pub fn switch_pane_backward(&mut self) {
        if self.pane_count > 1 {
            self.active_pane = match (self.pane_count, self.active_pane) {
                (2, Pane::Left) => Pane::Center,
                (2, Pane::Center) => Pane::Left,
                (3, Pane::Left) => Pane::Right,
                (3, Pane::Center) => Pane::Left,
                (3, Pane::Right) => Pane::Center,
                _ => Pane::Left,
            };
        }
    }
    
    /// Switch to left pane (Ctrl+Left)
    pub fn switch_pane_left(&mut self) {
        if self.pane_count > 1 {
            self.active_pane = match self.active_pane {
                Pane::Center => Pane::Left,
                Pane::Right if self.pane_count == 3 => Pane::Center,
                Pane::Right if self.pane_count == 2 => Pane::Left,
                _ => self.active_pane,
            };
        }
    }
    
    /// Switch to right pane (Ctrl+Right)
    pub fn switch_pane_right(&mut self) {
        if self.pane_count > 1 {
            self.active_pane = match (self.pane_count, self.active_pane) {
                (2, Pane::Left) => Pane::Center,
                (3, Pane::Left) => Pane::Center,
                (3, Pane::Center) => Pane::Right,
                _ => self.active_pane,
            };
        }
    }
    
    /// Check if split mode is active (for backward compatibility)
    #[allow(dead_code)]
    pub fn split_mode(&self) -> bool {
        self.pane_count > 1
    }

    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
    }
    
    pub fn toggle_shell(&mut self) {
        if self.show_shell {
            // Closing shell - stop PTY session
            self.shell.stop();
            self.show_shell = false;
        } else {
            // Opening shell - start PTY session
            self.shell.working_dir = self.active_fs().current_dir.clone();
            match self.shell.start() {
                Ok(_) => {
                    self.show_shell = true;
                    tracing::info!("Shell started at {:?}", self.shell.working_dir);
                }
                Err(e) => {
                    self.set_temp_message(format!("Failed to start shell: {}", e));
                    tracing::error!("Failed to start shell: {}", e);
                }
            }
        }
    }
    
    /// Toggle console panel (right side panel with PTY passthrough)
    pub fn toggle_console(&mut self) {
        if self.show_console {
            // Closing console - stop PTY session
            self.console.stop();
            self.show_console = false;
            self.console_focus = false;
        } else {
            // Opening console - start PTY session with current tree's path
            self.console.working_dir = self.active_fs().current_dir.clone();
            match self.console.start() {
                Ok(_) => {
                    self.show_console = true;
                    self.console_focus = true;  // Auto-focus console when opened
                    tracing::info!("Console panel started at {:?}", self.console.working_dir);
                }
                Err(e) => {
                    self.set_temp_message(format!("Failed to start console: {}", e));
                    tracing::error!("Failed to start console: {}", e);
                }
            }
        }
    }
    
    /// Switch focus between file manager and console panel
    #[allow(dead_code)]
    pub fn toggle_console_focus(&mut self) {
        if self.show_console {
            self.console_focus = !self.console_focus;
        }
    }
    
    /// Process file watcher events and refresh UI (call from event loop)
    #[allow(dead_code)]
    pub fn process_file_watcher(&mut self) {
        if let Some(watcher) = &self.file_watcher {
            let changes = watcher.poll_changes();
            
            if !changes.is_empty() {
                // Refresh file panels if any changes were detected
                self.refresh_both_panes();
            }
        }
    }
    
    /// Start watching the current directories
    #[allow(dead_code)]
    pub fn start_watching_dirs(&mut self) {
        if let Some(watcher) = &mut self.file_watcher {
            // Watch all active pane directories
            let _ = watcher.watch(&self.fs_left.current_dir);
            if self.pane_count >= 2 {
                let _ = watcher.watch(&self.fs_center.current_dir);
            }
            if self.pane_count >= 3 {
                let _ = watcher.watch(&self.fs_right.current_dir);
            }
        }
    }
    
    /// Update watched directories when navigation changes
    #[allow(dead_code)]
    pub fn update_watched_dirs(&mut self) {
        if let Some(watcher) = &mut self.file_watcher {
            // Unwatch all and rewatch current directories
            watcher.unwatch_all();
            let _ = watcher.watch(&self.fs_left.current_dir);
            if self.pane_count >= 2 {
                let _ = watcher.watch(&self.fs_center.current_dir);
            }
            if self.pane_count >= 3 {
                let _ = watcher.watch(&self.fs_right.current_dir);
            }
        }
    }
    
    /// Cycle focus forward through panes and console (Tab)
    /// Order: Left → Center → Right → Console → Left ...
    pub fn cycle_focus_forward(&mut self) {
        if self.show_console {
            if self.console_focus {
                // Console focused → go to first pane
                self.console_focus = false;
                self.active_pane = Pane::Left;
            } else {
                // Pane focused → go to next pane or console
                match (self.pane_count, self.active_pane) {
                    // Single pane
                    (1, _) => {
                        self.console_focus = true;
                    },
                    // 2 panes: Left → Center → Console
                    (2, Pane::Left) => self.active_pane = Pane::Center,
                    (2, Pane::Center) => {
                        self.console_focus = true;
                    },
                    // 3 panes: Left → Center → Right → Console
                    (3, Pane::Left) => self.active_pane = Pane::Center,
                    (3, Pane::Center) => self.active_pane = Pane::Right,
                    (3, Pane::Right) => {
                        self.console_focus = true;
                    },
                    _ => {}
                }
            }
        } else {
            // No console, just cycle panes
            if self.pane_count > 1 {
                self.switch_pane();
            } else {
                crate::navigation::navigate_column_forward(self.active_fs_mut());
            }
        }
    }
    
    /// Cycle focus backward through panes and console (Shift+Tab)
    /// Order: Console → Right → Center → Left → Console ...
    pub fn cycle_focus_backward(&mut self) {
        if self.show_console {
            if self.console_focus {
                // Console focused → go to last pane
                self.console_focus = false;
                self.active_pane = match self.pane_count {
                    1 => Pane::Left,
                    2 => Pane::Center,
                    _ => Pane::Right,
                };
            } else {
                // Pane focused → go to previous pane or console
                match (self.pane_count, self.active_pane) {
                    // Single pane
                    (1, _) => {
                        self.console_focus = true;
                    },
                    // 2 panes: Center → Left, Left → Console
                    (2, Pane::Center) => self.active_pane = Pane::Left,
                    (2, Pane::Left) => {
                        self.console_focus = true;
                    },
                    // 3 panes: Right → Center → Left → Console
                    (3, Pane::Right) => self.active_pane = Pane::Center,
                    (3, Pane::Center) => self.active_pane = Pane::Left,
                    (3, Pane::Left) => {
                        self.console_focus = true;
                    },
                    _ => {}
                }
            }
        } else {
            // No console, just cycle panes backward
            if self.pane_count > 1 {
                self.switch_pane_backward();
            } else {
                crate::navigation::navigate_column_backward(self.active_fs_mut());
            }
        }
    }
    
    pub fn toggle_process_viewer(&mut self) {
        self.show_process_viewer = !self.show_process_viewer;
        if self.show_process_viewer {
            // Refresh process list when opening
            self.process_viewer.refresh();
            tracing::info!("Process viewer opened");
        }
    }

    pub fn on_tick(&mut self, _dt: std::time::Duration) {
        if let AppMode::SystemMonitor = self.mode {
            self.system.refresh();
        }
        
        // Handle shell PTY reading (popup mode)
        if self.show_shell && self.shell.is_running {
            let _ = self.shell.read_and_parse();
            
            // Check if shell process is still running
            if !self.shell.check_running() {
                self.set_temp_message("Shell process exited".to_string());
            }
        }
        
        // Handle console PTY reading (panel mode)
        if self.show_console && self.console.is_running {
            let _ = self.console.read_and_parse();
            
            // Check if console process is still running
            if !self.console.check_running() {
                self.set_temp_message("Console process exited".to_string());
            }
        }
        
        // Refresh process viewer periodically (every 1 second)
        if self.show_process_viewer {
            if self.process_viewer.last_refresh.elapsed() >= std::time::Duration::from_secs(1) {
                self.process_viewer.refresh();
            }
        }
        
        // Auto-clear temporary messages after 0.5 seconds
        if let Some((_, start_time)) = self.temp_message {
            if start_time.elapsed() >= std::time::Duration::from_millis(500) {
                self.temp_message = None;
            }
        }
    }

    #[tracing::instrument(skip(self))]
    pub fn toggle_mode(&mut self, forward: bool) {
        tracing::info!(from = ?self.mode, forward, "Toggling mode");
        let modes = [
            AppMode::FileManager,
            AppMode::SystemMonitor,
            AppMode::Settings,
        ];
        
        let current_index = modes.iter().position(|&m| m == self.mode).unwrap_or(0);
        let next_index = if forward {
            (current_index + 1) % modes.len()
        } else {
            (current_index + modes.len() - 1) % modes.len()
        };

        self.mode = modes[next_index];
    }
    
    /// Set a temporary message that will auto-dismiss after 0.5 seconds
    pub fn set_temp_message(&mut self, message: String) {
        self.temp_message = Some((message, Instant::now()));
    }
}
