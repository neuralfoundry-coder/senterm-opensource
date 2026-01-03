use std::io;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::time::{Duration, Instant};
use anyhow::Result;

mod app;
mod ui;
mod fs;
mod system;
mod viewer;
mod config;
mod logging;
mod navigation;
mod events;
mod process;
mod plugin;

use app::App;
use crate::app::AppMode;
use ui::ui;

#[tokio::main]
async fn main() -> Result<()> {
    // Setup logging
    let _guard = logging::init();
    tracing::info!("Application starting");

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app
    let mut app = App::new();
    let res = run_app(&mut terminal, &mut app).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    // Save config before exiting
    if let Err(e) = app.config.save() {
        eprintln!("Failed to save config: {}", e);
    }

    if let Err(err) = res {
        println!("{:?}", err);
    }

    Ok(())
}

#[tracing::instrument(skip(terminal, app))]
async fn run_app<B: ratatui::backend::Backend>(terminal: &mut Terminal<B>, app: &mut App) -> io::Result<()> {
    tracing::info!("Starting main event loop");
    let mut last_tick = Instant::now();

    loop {
        terminal.draw(|f| ui(f, app))?;
        
        // Update console PTY size based on actual terminal area
        if app.show_console && app.console.is_running {
            let size = terminal.size()?;
            // Console panel is 40% width, full height minus header(1) and footer(1) and borders(2) and help line(1)
            let console_cols = ((size.width as f32 * 0.40) as u16).saturating_sub(4); // -4 for borders and margins
            let console_rows = size.height.saturating_sub(5); // -1 header -1 footer -2 borders -1 help line
            
            // Only resize if size actually changed
            if app.console.size != (console_cols, console_rows) {
                app.console.resize(console_cols, console_rows);
            }
        }
        
        // Update shell popup PTY size based on actual terminal area
        if app.show_shell && app.shell.is_running {
            let size = terminal.size()?;
            // Shell popup is 85% width x 75% height, minus borders(2) and help line(1)
            let shell_cols = ((size.width as f32 * 0.85) as u16).saturating_sub(4);
            let shell_rows = ((size.height as f32 * 0.75) as u16).saturating_sub(4);
            
            // Only resize if size actually changed
            if app.shell.size != (shell_cols, shell_rows) {
                app.shell.resize(shell_cols, shell_rows);
            }
        }
        
        // Update process viewer visible height based on terminal size
        if app.show_process_viewer {
            let size = terminal.size()?;
            // Calculate visible height: 85% of screen height - header(2) - details(6) - footer(1) - borders(4)
            let visible_height = ((size.height as f32 * 0.85) as usize).saturating_sub(13);
            app.process_viewer.set_visible_height(visible_height.max(5));
        }

        let poll_timeout = Duration::from_millis(100);

        // Check if shell/console PTY exited (e.g., user typed 'exit')
        if app.show_shell && app.shell.is_running {
            if !app.shell.check_running() {
                // Shell process exited, close shell mode
                app.show_shell = false;
                app.status_message = Some("Shell exited".to_string());
            }
        }
        if app.show_console && app.console.is_running {
            if !app.console.check_running() {
                // Console process exited, close console mode
                app.show_console = false;
                app.console_focus = false;
                app.status_message = Some("Console exited".to_string());
            }
        }

        if crossterm::event::poll(poll_timeout)?{
            let event = event::read()?;
            
            match event {
                Event::Mouse(mouse_event) => {
                    if let AppMode::FileManager = app.mode {
                        let size = terminal.size()?;
                        handle_mouse_event(mouse_event, app, size.width, size.height);
                    }
                },
                Event::Key(key) => {
                    // Global Hotkeys
                    // Ctrl + Shift + Alt + K
                    // Note: Shift + k usually produces 'K', so we check for 'K' with Control and Alt modifiers.
                    // We also check for 'k' with all three modifiers to be safe.
                    let is_help_hotkey = (key.code == KeyCode::Char('K') && key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL | crossterm::event::KeyModifiers::ALT)) ||
                                         (key.code == KeyCode::Char('k') && key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL | crossterm::event::KeyModifiers::ALT | crossterm::event::KeyModifiers::SHIFT));

                    if is_help_hotkey {
                        app.toggle_help();
                        continue; // Skip other processing
                    }

                    // Settings hotkey: Ctrl/Cmd + Shift + Alt + H
                    let is_settings_hotkey = (key.code == KeyCode::Char('H') && key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL | crossterm::event::KeyModifiers::ALT)) ||
                                             (key.code == KeyCode::Char('h') && key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL | crossterm::event::KeyModifiers::ALT | crossterm::event::KeyModifiers::SHIFT)) ||
                                             (key.code == KeyCode::Char('H') && key.modifiers.contains(crossterm::event::KeyModifiers::SUPER | crossterm::event::KeyModifiers::ALT)) ||
                                             (key.code == KeyCode::Char('h') && key.modifiers.contains(crossterm::event::KeyModifiers::SUPER | crossterm::event::KeyModifiers::ALT | crossterm::event::KeyModifiers::SHIFT));

                    if is_settings_hotkey {
                        app.mode = AppMode::Settings;
                        continue;
                    }

                    // Console panel toggle hotkey: F5 or Ctrl+T
                    let is_console_toggle = key.code == KeyCode::F(5) ||
                        ((key.code == KeyCode::Char('t') || key.code == KeyCode::Char('T')) && 
                         key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL));
                    if is_console_toggle {
                        app.toggle_console();
                        continue;
                    }
                    
                    // Pane management - handle BEFORE console to always work
                    // F3: Add pane, F4: Remove pane (Shift+F3 doesn't work well in some terminals)
                    if let AppMode::FileManager = app.mode {
                        if key.code == KeyCode::F(3) {
                            app.add_pane();
                            continue;
                        }
                        if key.code == KeyCode::F(4) {
                            app.remove_pane();
                            continue;
                        }
                    }
                    
                    // F5: Toggle console (Shell) panel - global hotkey
                    if key.code == KeyCode::F(5) {
                        app.toggle_console();
                        continue;
                    }
                    
                    // Handle console input when console is open and focused
                    if app.show_console && app.console_focus {
                        crate::events::handle_console_keys(app, key.code, key.modifiers);
                        continue;
                    }
                    
                    // Shell toggle hotkey: F12 or ` (backtick)
                    let is_shell_hotkey = key.code == KeyCode::F(12) || 
                                          (key.code == KeyCode::Char('`') && !app.show_shell);
                    
                    if is_shell_hotkey {
                        app.toggle_shell();
                        continue;
                    }

                    // Handle shell input when shell is open (highest priority modal)
                    if app.show_shell {
                        crate::events::handle_shell_keys(app, key.code, key.modifiers);
                        continue;
                    }

                    // Process viewer toggle hotkey: F9
                    if key.code == KeyCode::F(9) && !app.show_process_viewer {
                        app.toggle_process_viewer();
                        continue;
                    }
                    
                    // Settings toggle hotkey: F8
                    if key.code == KeyCode::F(8) {
                        if let AppMode::Settings = app.mode {
                            app.mode = AppMode::FileManager;
                        } else {
                            app.mode = AppMode::Settings;
                        }
                        continue;
                    }

                    // Handle process viewer input when open
                    if app.show_process_viewer {
                        crate::events::handle_process_viewer_keys(app, key.code);
                        continue;
                    }

                    if app.show_help {
                        if key.code == KeyCode::Esc || key.code == KeyCode::Char('q') {
                            app.toggle_help();
                        }
                        continue; // Modal blocks other input
                    }

                    // Handle bookmark selection
                    if app.show_bookmarks {
                        match key.code {
                            KeyCode::Char('1'..='9') => {
                                if let KeyCode::Char(c) = key.code {
                                    let idx = c.to_digit(10).unwrap() as usize - 1;
                                    if idx < app.config.bookmarks.len() {
                                        app.active_fs_mut().current_dir = app.config.bookmarks[idx].clone();
                                        app.show_bookmarks = false;
                                        app.status_message = Some(format!("Jumped to bookmark {}", idx + 1));
                                    }
                                }
                            },
                            KeyCode::Char('d') => {
                                // Delete bookmark mode - press number to delete
                                if let KeyCode::Char(c) = key.code {
                                    if c.is_ascii_digit() {
                                        let idx = c.to_digit(10).unwrap() as usize - 1;
                                        if idx < app.config.bookmarks.len() {
                                            app.config.bookmarks.remove(idx);
                                            let _ = app.config.save();
                                            app.status_message = Some(format!("Deleted bookmark {}", idx + 1));
                                        }
                                    }
                                }
                            },
                            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('B') => {
                                app.show_bookmarks = false;
                            },
                            _ => {}
                        }
                        continue; // Modal blocks other input
                    }

                    // Handle dialog input (blocks other input when active)
                    if crate::events::handle_dialog_keys(app, key.code) {
                        continue;
                    }

                match key.code {
                    KeyCode::Char('q') => {
                        if let AppMode::Viewer = app.mode {
                            // In Viewer mode, delegate to viewer handler if editing
                            if app.viewer_editing {
                                crate::events::handle_viewer_keys(app, key.code, key.modifiers);
                            } else {
                                // Close viewer if not editing
                                app.mode = AppMode::FileManager;
                                app.viewer_content = None;
                                app.viewer_scroll = 0;
                                app.text_editor = None;
                            }
                        } else {
                            app.should_quit = true;
                        }
                    },
                    KeyCode::Esc => {
                        // Esc behavior depends on mode
                        match app.mode {
                            AppMode::Viewer => {
                                // If editing, delegate to vim handler; otherwise close viewer
                                if app.viewer_editing {
                                    crate::events::handle_viewer_keys(app, KeyCode::Esc, crossterm::event::KeyModifiers::empty());
                                } else {
                                    // Close viewer and return to file manager
                                    app.mode = AppMode::FileManager;
                                    app.viewer_content = None;
                                    app.viewer_scroll = 0;
                                    app.text_editor = None;
                                }
                            },
                            AppMode::Settings => {
                                // Close settings and return to file manager
                                app.mode = AppMode::FileManager;
                            },
                            _ => {
                                // Other modes: show quit confirmation dialog
                                app.dialog = crate::app::DialogMode::QuitConfirm;
                            }
                        }
                    },
                    KeyCode::Char('[') => app.toggle_mode(false),
                    KeyCode::Char(']') => app.toggle_mode(true),
                    // F3/Shift+F3 handled earlier (before console handler)
                    KeyCode::Tab => {
                        if let AppMode::FileManager = app.mode {
                            // Cycle through panes and console (if open)
                            app.cycle_focus_forward();
                        } else {
                            app.toggle_mode(true);
                        }
                    },
                    KeyCode::BackTab => {
                        if let AppMode::FileManager = app.mode {
                            // Cycle backward through panes and console (if open)
                            app.cycle_focus_backward();
                        } else {
                            app.toggle_mode(false);
                        }
                    },
                    KeyCode::Left if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => {
                        if let AppMode::FileManager = app.mode {
                            app.switch_pane_left();
                        }
                    },
                    KeyCode::Right if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => {
                        if let AppMode::FileManager = app.mode {
                            app.switch_pane_right();
                        }
                    },
                    _ => {
                        // Try clipboard operations first (Ctrl/Cmd + C/X/V)
                        if !crate::events::handle_clipboard_operations(app, key.code, key.modifiers) {
                            // If not a clipboard operation, handle mode-specific keys
                            match app.mode {
                                AppMode::FileManager => {
                                    crate::events::handle_file_manager_keys(app, key.code);
                                },
                                AppMode::Viewer => {
                                    crate::events::handle_viewer_keys(app, key.code, key.modifiers);
                                },
                                AppMode::SystemMonitor => {},
                                AppMode::Setup => {
                                    crate::events::handle_setup_keys(app, key.code);
                                },
                                AppMode::Settings => {
                                    crate::events::handle_settings_keys(app, key.code, key.modifiers);
                                },
                            }
                        }
                    } // End of match app.mode
                } // End of match key.code
                }, // End of Event::Key
                _ => {} // Ignore other events
            } // End of match event
        } // End of if poll

        // Calculate delta time and call on_tick
        let now = Instant::now();
        let dt = now.duration_since(last_tick);
        last_tick = now;
        app.on_tick(dt);

        if app.should_quit {
            return Ok(());
        }

        // Handle external game launch
        if app.launch_external_game {
            app.launch_external_game = false;
            
            // Restore terminal before launching external game
            disable_raw_mode()?;
            execute!(
                io::stdout(),
                LeaveAlternateScreen,
                DisableMouseCapture
            )?;

            // Try to find senterm-games binary
            let game_result = launch_external_game();
            
            // Re-enter terminal mode
            enable_raw_mode()?;
            execute!(
                io::stdout(),
                EnterAlternateScreen,
                EnableMouseCapture
            )?;
            terminal.clear()?;

            // Set status message based on result
            match game_result {
                Ok(_) => {
                    app.status_message = Some("Returned from senterm-games".to_string());
                }
                Err(e) => {
                    app.status_message = Some(format!("Game launch error: {}", e));
                }
            }
        }
    }
}

fn launch_external_game() -> io::Result<()> {
    use std::process::Command;

    // Try different locations for senterm-games binary
    let possible_paths = [
        // Development path (workspace target)
        std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.join("senterm-games"))),
        // Same directory as current executable
        std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.join("senterm-games"))),
        // System PATH
        Some(std::path::PathBuf::from("senterm-games")),
        // Cargo target directory (for development)
        Some(std::path::PathBuf::from("./target/debug/senterm-games")),
        Some(std::path::PathBuf::from("./target/release/senterm-games")),
    ];

    for path_opt in possible_paths.into_iter().flatten() {
        if path_opt.exists() || path_opt.to_string_lossy() == "senterm-games" {
            tracing::info!("Attempting to launch senterm-games from: {:?}", path_opt);
            
            let mut child = Command::new(&path_opt)
                .spawn();

            match &mut child {
                Ok(c) => {
                    // Wait for the game to finish
                    let status = c.wait()?;
                    tracing::info!("senterm-games exited with status: {}", status);
                    return Ok(());
                }
                Err(e) if e.kind() != io::ErrorKind::NotFound => {
                    return Err(io::Error::new(e.kind(), format!("Failed to launch senterm-games: {}", e)));
                }
                _ => continue,
            }
        }
    }

    Err(io::Error::new(
        io::ErrorKind::NotFound,
        "senterm-games binary not found. Please build it with: cargo build -p senterm-games",
    ))
}

fn handle_mouse_event(mouse: crossterm::event::MouseEvent, app: &mut App, width: u16, height: u16) {
    use crossterm::event::MouseEventKind;

    if !matches!(mouse.kind, MouseEventKind::Down(crossterm::event::MouseButton::Left)) {
        return; // Only handle left clicks
    }

    // Calculate layout similar to UI (3 rows: title=5, content=remaining, status=3)
    let title_height = 5;  // Updated for ASCII art title
    let status_height = 3;
    let content_start_y = title_height;
    let content_height = height.saturating_sub(title_height + status_height);

    // Check if click is in content area
    if mouse.row < content_start_y || mouse.row >= content_start_y + content_height {
        return;
    }

    // Get visible columns using navigation module (no preview)
    let nav_columns = crate::navigation::calculate_visible_columns(app.active_fs(), 5);
    let total_columns = nav_columns.total_columns;

    if total_columns == 0 {
        return;
    }

    let column_width = width / total_columns as u16;
    let clicked_column = (mouse.column / column_width).min((total_columns - 1) as u16) as usize;

    // Update active column
    app.active_fs_mut().active_column_index = clicked_column;

    // Calculate item index within column (row - content_start_y - 1 for border)
    let item_row = mouse.row.saturating_sub(content_start_y + 1); // +1 for top border

    // Get the directory for the clicked column
    if let Some(clicked_dir) = crate::navigation::get_active_directory(app.active_fs()) {
        let entries = crate::fs::FileSystem::get_entries_for_dir(&clicked_dir);
        if (item_row as usize) < entries.len() {
            app.active_fs_mut().set_selection(clicked_dir, item_row as usize);
        }
    }
}
