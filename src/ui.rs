use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, BorderType, List, ListItem, Paragraph},
    Frame,
};
use crate::app::{App, AppMode};

// Helper function to truncate string (UTF-8 safe)
fn truncate_str(s: &str, max_len: usize) -> String {
    let char_count = s.chars().count();
    if char_count <= max_len {
        s.to_string()
    } else {
        format!("{}...", s.chars().take(max_len.saturating_sub(3)).collect::<String>())
    }
}

// Helper function to truncate path for display (UTF-8 safe)
fn truncate_path(path: &str, max_len: usize) -> String {
    let char_count = path.chars().count();
    if char_count <= max_len {
        format!("{:<width$}", path, width = max_len)
    } else {
        let start_len = max_len / 2 - 2;
        let end_len = max_len - start_len - 3;
        let start: String = path.chars().take(start_len).collect();
        let end: String = path.chars().skip(char_count - end_len).collect();
        format!("{:<width$}", format!("{}...{}", start, end), width = max_len)
    }
}

pub fn ui(f: &mut Frame, app: &App) {
    // Render background
    let theme = &app.config.theme;
    let background = Block::default().style(Style::default().bg(theme.bg));
    f.render_widget(background, f.area());

    match app.mode {
        AppMode::FileManager => draw_file_manager(f, app),
        AppMode::SystemMonitor => draw_system_monitor(f, app),
        AppMode::Setup => draw_setup(f, app),
        AppMode::Settings => draw_settings(f, app),
        AppMode::Viewer => {}, // Viewer is rendered as overlay below
    }
    
    // Render viewer popup overlay if in viewer mode
    if app.mode == AppMode::Viewer {
        draw_viewer_popup(f, app);
    }
    
    if app.show_help {
        draw_help_popup(f, app);
    }

    if app.show_bookmarks {
        draw_bookmarks_popup(f, app);
    }

    // Render dialog popups
    if !matches!(app.dialog, crate::app::DialogMode::None) {
        draw_dialog_popup(f, app);
    }
    
    // Render shell popup (above dialogs)
    if app.show_shell {
        draw_shell_popup(f, app);
    }
    
    // Render process viewer popup
    if app.show_process_viewer {
        draw_process_viewer_popup(f, app);
    }
    
    // Render temporary message popup (on top of everything else)
    if app.temp_message.is_some() {
        draw_temp_message_popup(f, app);
    }
}

fn draw_help_popup(f: &mut Frame, app: &App) {
    let theme = &app.config.theme;
    let area = centered_rect(60, 50, f.area());

    let block = Block::default()
        .title(" HELP - HOTKEYS ")
        .borders(Borders::ALL)
        .border_type(BorderType::Thick)
        .border_style(Style::default().fg(theme.border))
        .style(Style::default().bg(theme.bg).fg(theme.fg));

    let text = vec![
        ListItem::new(" GENERAL COMMANDS"),
        ListItem::new(" ─────────────────────────────────────────────────────"),
        ListItem::new("  [ / ]              : Switch Mode (Prev/Next)"),
        ListItem::new("  Q                  : Quit"),
        ListItem::new("  Ctrl+Shift+Alt+K   : Toggle Help"),
        ListItem::new(""),
        ListItem::new(" FILE MANAGER"),
        ListItem::new(" ─────────────────────────────────────────────────────"),
        ListItem::new("  TAB / Shift+TAB    : Switch Column Focus"),
        ListItem::new("  Arrow Keys         : Navigate"),
        ListItem::new("  ENTER              : Open Directory"),
        ListItem::new("  BACKSPACE          : Go to Parent Directory"),
        ListItem::new("  /                  : Search Files"),
        ListItem::new("  s                  : Cycle Sort (Name/Size/Date)"),
        ListItem::new("  b                  : Bookmark Current Directory"),
        ListItem::new("  B (Shift+b)        : Show Bookmarks"),
        ListItem::new("  c/x/p              : Copy/Cut/Paste"),
        ListItem::new(""),
        ListItem::new(" SETTINGS"),
        ListItem::new(" ─────────────────────────────────────────────────────"),
        ListItem::new("  1-2                : Switch Theme"),
        ListItem::new(""),
    ];

    let list = List::new(text).block(block);

    f.render_widget(ratatui::widgets::Clear, area); // Clear background
    f.render_widget(list, area);
}

fn draw_bookmarks_popup(f: &mut Frame, app: &App) {
    let theme = &app.config.theme;
    let area = centered_rect(70, 60, f.area());

    f.render_widget(ratatui::widgets::Clear, area);

    let bookmark_items: Vec<ListItem> = if app.config.bookmarks.is_empty() {
        vec![
            ListItem::new(""),
            ListItem::new("  No bookmarks yet!"),
            ListItem::new(""),
            ListItem::new("  Press 'b' in File Manager to bookmark"),
            ListItem::new("  the current directory."),
            ListItem::new(""),
        ]
    } else {
        let mut items = vec![
            ListItem::new(" BOOKMARKED DIRECTORIES"),
            ListItem::new(" ─────────────────────────────────────────────────────"),
            ListItem::new(""),
        ];

        for (idx, bookmark) in app.config.bookmarks.iter().enumerate().take(9) {
            let path_str = bookmark.display().to_string();
            items.push(ListItem::new(format!(
                "  [{}] {}",
                idx + 1,
                truncate_path(&path_str, 50)
            )));
        }

        items.push(ListItem::new(""));
        items.push(ListItem::new(" ─────────────────────────────────────────────────────"));
        items.push(ListItem::new("  1-9: Jump to bookmark  |  d+NUM: Delete bookmark"));
        items.push(ListItem::new("  ESC/Q/B: Close"));

        items
    };

    let block = Block::default()
        .title(" BOOKMARKS ")
        .borders(Borders::ALL)
        .border_type(BorderType::Thick)
        .border_style(Style::default().fg(theme.accent_color))
        .style(Style::default().bg(theme.bg).fg(theme.fg));

    let list = List::new(bookmark_items).block(block);
    f.render_widget(list, area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: ratatui::layout::Rect) -> ratatui::layout::Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

fn draw_file_manager(f: &mut Frame, app: &App) {
    use crate::app::Pane;
    
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),  // Header (single line)
            Constraint::Min(0),     // Content
            Constraint::Length(1),  // Footer
        ])
        .split(f.area());

    let theme = &app.config.theme;

    // Simple single-line header
    let path_str = app.active_fs().current_dir.display().to_string();
    let pane_indicator = if app.pane_count > 1 {
        match app.active_pane {
            Pane::Left => "[L]",
            Pane::Center => "[C]",
            Pane::Right => "[R]",
        }
    } else { "" };
    
    let title_prefix = match app.pane_count {
        1 => " SenTerm",
        2 => " SenTerm[2]",
        _ => " SenTerm[3]",
    };
    
    // Build panel indicators
    let mut panel_indicators = String::new();
    if app.show_console { panel_indicators.push_str(" │ [Console]"); }
    
    let header_text = format!("{} │ {}{}{}", title_prefix, pane_indicator, truncate_path(&path_str, 50), panel_indicators);
    let header = Paragraph::new(header_text)
        .style(Style::default().fg(theme.header_fg).bg(theme.header_bg));
    f.render_widget(header, chunks[0]);

    // Determine layout based on which panels are shown
    // - Console open: 2-way split (Tree 60% | Console 40%)
    // - No panels: Full tree
    let is_any_panel_focused = app.console_focus;
    
    let content_area = if app.show_console {
        // 2-way split: Tree | Console
        let main_split = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(60),  // File manager area
                Constraint::Percentage(40),  // Console panel
            ])
            .split(chunks[1]);
        
        draw_console_panel(f, app, main_split[1], theme);
        
        main_split[0]
    } else {
        // Full tree
        chunks[1]
    };

    // Content area - split or single pane (file manager)
    match app.pane_count {
        1 => {
            draw_single_pane(f, &app.fs_left, content_area, !is_any_panel_focused, theme);
        },
        2 => {
            let panes = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(50),
                    Constraint::Percentage(50),
                ])
                .split(content_area);

            draw_single_pane(f, &app.fs_left, panes[0], app.active_pane == Pane::Left && !is_any_panel_focused, theme);
            draw_single_pane(f, &app.fs_center, panes[1], app.active_pane == Pane::Center && !is_any_panel_focused, theme);
        },
        _ => {
            let panes = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(33),
                    Constraint::Percentage(34),
                    Constraint::Percentage(33),
                ])
                .split(content_area);

            draw_single_pane(f, &app.fs_left, panes[0], app.active_pane == Pane::Left && !is_any_panel_focused, theme);
            draw_single_pane(f, &app.fs_center, panes[1], app.active_pane == Pane::Center && !is_any_panel_focused, theme);
            draw_single_pane(f, &app.fs_right, panes[2], app.active_pane == Pane::Right && !is_any_panel_focused, theme);
        }
    }

    // Status bar - dynamic based on shown panels
    let status_text = if let Some(msg) = &app.status_message {
        format!(" {}", msg)
    } else {
        let mut hints = vec!["F3:Split"];
        if app.show_console {
            hints.push("Tab:Cycle");
        }
        hints.push(if app.show_console { "F5:Close" } else { "F5:Console" });
        hints.push("Q:Quit");
        format!(" {}", hints.join(" │ "))
    };

    let status_block = Block::default().style(Style::default().bg(theme.footer_bg));
    f.render_widget(status_block, chunks[2]);
    
    let status = Paragraph::new(status_text)
        .style(Style::default().fg(theme.footer_fg).bg(theme.footer_bg));
    f.render_widget(status, chunks[2]);
}

/// Draw a single file manager pane
fn draw_single_pane(f: &mut Frame, fs: &crate::fs::FileSystem, area: ratatui::layout::Rect, is_active: bool, theme: &crate::config::Theme) {
    // Draw pane border first
    let pane_border_style = if is_active {
        Style::default().fg(theme.accent_color).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    
    let pane_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(pane_border_style);
    f.render_widget(pane_block, area);
    
    // Inner area for content
    let inner_area = area.inner(ratatui::layout::Margin { vertical: 1, horizontal: 1 });
    
    // Build columns using navigation module
    let nav_columns = crate::navigation::calculate_visible_columns(fs, 5);
    let visible_path = &nav_columns.visible_path;
    let total_columns = nav_columns.total_columns;
    
    if total_columns == 0 {
        return;
    }
    
    // Create equal-width columns
    let column_constraints: Vec<Constraint> = (0..total_columns)
        .map(|_| Constraint::Percentage((100 / total_columns) as u16))
        .collect();
    
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(column_constraints)
        .split(inner_area);

    let mut col_idx = 0;
    
    // Render each level in the visible path
    for (level, dir_path) in visible_path.iter().enumerate() {
        let is_active_column = col_idx == fs.active_column_index;
        let entries = crate::fs::FileSystem::get_entries_for_dir_sorted(dir_path, fs.sort_option);
        
        let items: Vec<ListItem> = entries
            .iter()
            .enumerate()
            .map(|(idx, path)| {
                let is_parent_entry = if let Some(parent) = dir_path.parent() {
                    path == &parent.to_path_buf()
                } else {
                    false
                };

                let file_name = if is_parent_entry {
                    "..".to_string()
                } else {
                    path.file_name().unwrap_or_default().to_string_lossy().to_string()
                };

                let is_dir = path.is_dir();
                let is_symlink = path.is_symlink();
                
                let is_executable = if let Some(ext) = path.extension() {
                    let ext_str = ext.to_string_lossy().to_lowercase();
                    matches!(ext_str.as_str(), "exe" | "bat" | "sh" | "bin" | "app" | "cmd")
                } else {
                    false
                };

                let icon = if is_dir { "■ " } else if is_symlink { "↗ " } else { "· " };
                let display_text = format!("{} {}", icon, file_name);

                let mut style = Style::default().bg(theme.bg);

                if is_dir {
                    style = style.fg(theme.directory_fg);
                } else if is_symlink {
                    style = style.fg(theme.symlink_fg);
                } else if is_executable {
                    style = style.fg(theme.executable_fg);
                } else {
                    style = style.fg(theme.file_fg);
                }

                // Clipboard highlighting
                if let Some((clip_path, op)) = &fs.clipboard {
                    if clip_path == path {
                        match op {
                            crate::fs::ClipboardOperation::Copy => {
                                style = style.fg(Color::Yellow);
                            },
                            crate::fs::ClipboardOperation::Cut => {
                                style = style.fg(Color::Red).add_modifier(Modifier::DIM);
                            }
                        }
                    }
                }

                // Selection highlighting
                let current_selection = fs.get_selection(dir_path);
                if idx == current_selection {
                    if is_active_column && is_active {
                        style = style.bg(theme.selection_bg).fg(theme.selection_fg).add_modifier(Modifier::BOLD);
                    } else {
                        style = style.add_modifier(Modifier::DIM);
                    }
                }

                // Highlight path to next level
                if level + 1 < visible_path.len() {
                    if path == &visible_path[level + 1] {
                        style = style.add_modifier(Modifier::BOLD).fg(theme.accent_color);
                    }
                }

                // Dim inactive pane
                if !is_active {
                    style = style.add_modifier(Modifier::DIM);
                }

                ListItem::new(display_text).style(style)
            })
            .collect();

        let border_style = if is_active_column && is_active {
            Style::default().fg(theme.accent_color).add_modifier(Modifier::BOLD)
        } else if is_active {
            Style::default().fg(theme.border)
        } else {
            Style::default().fg(theme.border).add_modifier(Modifier::DIM)
        };

        let dir_name = dir_path.file_name().unwrap_or_default().to_str().unwrap_or("...");
        
        let list = List::new(items)
            .block(Block::default()
                .borders(Borders::LEFT)
                .border_style(border_style)
                .title(dir_name));
        f.render_widget(list, columns[col_idx]);
        col_idx += 1;
    }
}

/// Draw the console panel on the right side (PTY passthrough)
fn draw_console_panel(f: &mut Frame, app: &App, area: ratatui::layout::Rect, theme: &crate::config::Theme) {
    // Only draw shell console
    draw_console_shell(f, app, area, theme);
}

fn draw_console_shell(f: &mut Frame, app: &App, area: ratatui::layout::Rect, theme: &crate::config::Theme) {
    // Border style based on focus - use theme colors
    let border_style = if app.console_focus {
        Style::default().fg(theme.accent_color).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme.border)
    };
    
    // Title with working directory
    let title = if app.console.is_running {
        format!(" SHELL - {} ", truncate_path(&app.console.working_dir.display().to_string(), 30))
    } else {
        " SHELL - Not Running ".to_string()
    };
    
    // Main block - use theme colors
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(border_style)
        .style(Style::default().bg(theme.bg).fg(theme.fg))
        .title(title);
    
    f.render_widget(block.clone(), area);
    
    // Inner area
    let inner_area = area.inner(ratatui::layout::Margin { vertical: 1, horizontal: 1 });
    
    // Split into terminal area and help line
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(3),      // Terminal area
            Constraint::Length(1),   // Help line
        ])
        .split(inner_area);
    
    let terminal_area = layout[0];
    
    // Render terminal content from vt100 parser
    if let Ok(parser) = app.console.parser.try_lock() {
        let screen = parser.screen();
        
        // Render each row
        for row in 0..terminal_area.height {
            let row_idx = row as usize;
            if row_idx >= screen.size().0 as usize {
                break;
            }
            
            let mut spans: Vec<ratatui::text::Span> = Vec::new();
            let mut current_text = String::new();
            let mut current_style = Style::default();
            
            for col in 0..terminal_area.width {
                let col_idx = col as usize;
                if col_idx >= screen.size().1 as usize {
                    break;
                }
                
                let cell = screen.cell(row_idx as u16, col_idx as u16);
                
                if let Some(cell) = cell {
                    // Convert vt100 colors to ratatui colors with theme support
                    let style = convert_vt100_style(&cell, theme);
                    
                    if style != current_style && !current_text.is_empty() {
                        spans.push(ratatui::text::Span::styled(current_text.clone(), current_style));
                        current_text.clear();
                    }
                    
                    current_style = style;
                    current_text.push(cell.contents().chars().next().unwrap_or(' '));
                } else {
                    current_text.push(' ');
                }
            }
            
            if !current_text.is_empty() {
                spans.push(ratatui::text::Span::styled(current_text, current_style));
            }
            
            let line = ratatui::text::Line::from(spans);
            let para = Paragraph::new(line);
            f.render_widget(para, ratatui::layout::Rect {
                x: terminal_area.x,
                y: terminal_area.y + row,
                width: terminal_area.width,
                height: 1,
            });
        }
        
        // Set cursor position from vt100 screen if console has focus
        if app.console_focus && app.console.is_running {
            let (cursor_row, cursor_col) = screen.cursor_position();
            let cursor_x = terminal_area.x + cursor_col;
            let cursor_y = terminal_area.y + cursor_row;
            
            if cursor_x < terminal_area.x + terminal_area.width && 
               cursor_y < terminal_area.y + terminal_area.height {
                f.set_cursor_position((cursor_x, cursor_y));
            }
        }
    }
    
    // Help line
    let help_text = if app.console_focus {
        " Esc:Unfocus | F5:Close "
    } else {
        " Tab:Focus | F5:Close "
    };
    let help_para = Paragraph::new(help_text)
        .style(Style::default().fg(theme.footer_fg).bg(theme.footer_bg));
    f.render_widget(help_para, layout[1]);
}

fn draw_viewer_popup(f: &mut Frame, app: &App) {
    let theme = &app.config.theme;

    // Use full screen for file viewer
    let area = f.area();

    // Clear background
    f.render_widget(ratatui::widgets::Clear, area);
    
    // Main block with border
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Thick)
        .border_style(Style::default().fg(theme.border))
        .style(Style::default().bg(theme.bg).fg(theme.fg));
    f.render_widget(block, area);

    // Inner area for content (excluding borders)
    let inner_area = area.inner(ratatui::layout::Margin { vertical: 1, horizontal: 1 });

    // Split into Header, Content, Footer
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),  // Header
            Constraint::Length(1),  // Separator
            Constraint::Min(0),     // Content
            Constraint::Length(1),  // Footer
        ])
        .split(inner_area);

    // Header Content (Filename, etc.)
    let header_text = if let Some(editor) = &app.text_editor {
        if let Some(path) = &editor.file_path {
             format!(" FILE: {} {}", path.display(), if editor.modified { "[+]" } else { "" })
        } else {
            " NEW FILE ".to_string()
        }
    } else if let Some(content) = &app.viewer_content {
        match content {
            crate::viewer::ViewerContent::Image(path) => format!(" IMAGE: {}", path.display()),
            _ => " FILE VIEWER ".to_string(),
        }
    } else {
         " FILE VIEWER ".to_string()
    };

    let header = Paragraph::new(header_text)
        .style(Style::default().fg(theme.accent_color).add_modifier(Modifier::BOLD));
    f.render_widget(header, layout[0]);

    // Separator
    let separator = Block::default().borders(Borders::BOTTOM).border_style(Style::default().fg(theme.border));
    f.render_widget(separator, layout[1]);

    // Content Area
    let content_area = layout[2];

    // Check if in edit mode
    if app.viewer_editing {
        if let Some(editor) = &app.text_editor {
            draw_editor_content(f, editor, content_area, theme);
            
            // Status line in Footer
            let status_text = if editor.mode == crate::viewer::VimMode::Command {
                format!(":{}", editor.command_buffer)
            } else {
                format!(" {} | Line {}/{}, Col {} | {}", 
                    editor.status_message,
                    editor.cursor_row + 1,
                    editor.lines.len(),
                    editor.cursor_col + 1,
                    match editor.editor_style {
                        crate::viewer::EditorStyle::Nano => "NANO",
                        crate::viewer::EditorStyle::Vim => match editor.mode {
                            crate::viewer::VimMode::Normal => "NORMAL",
                            crate::viewer::VimMode::Insert => "INSERT",
                            crate::viewer::VimMode::Command => "COMMAND",
                            crate::viewer::VimMode::Visual => "VISUAL",
                            crate::viewer::VimMode::VisualLine => "V-LINE",
                        },
                    }
                )
            };
            let footer = Paragraph::new(status_text)
                .style(Style::default().fg(theme.fg).bg(theme.selection_bg));
            f.render_widget(footer, layout[3]);
        }
    } else {
        draw_viewer_readonly_content(f, app, content_area, theme);
        
        // Footer for ReadOnly
        let wrap_indicator = if app.viewer_wrap_mode { "[W]" } else { "" };
        let footer_text = format!(" g/G:Top/Bottom | j/k:↑↓ | d/u:Half | w:Wrap{} | i:Edit | ESC:Close ", wrap_indicator);
        let footer = Paragraph::new(footer_text)
            .style(Style::default().fg(theme.footer_fg).bg(theme.footer_bg))
            .alignment(ratatui::layout::Alignment::Right);
        f.render_widget(footer, layout[3]);
    }
}

fn draw_viewer_readonly_content(f: &mut Frame, app: &App, area: ratatui::layout::Rect, theme: &crate::config::Theme) {
    // Check if content is syntax highlighted code
    if let Some(crate::viewer::ViewerContent::HighlightedCode { highlighted, .. }) = &app.viewer_content {
        draw_highlighted_code(f, app, area, theme, highlighted);
        return;
    }

    let content_text = match &app.viewer_content {
        Some(crate::viewer::ViewerContent::PlainText(s)) => s.clone(),
        Some(crate::viewer::ViewerContent::HighlightedCode { raw, .. }) => raw.clone(),
        Some(crate::viewer::ViewerContent::Markdown(s)) => s.clone(),
        Some(crate::viewer::ViewerContent::Image(path)) => {
            let mut info = String::new();
            info.push_str("\n  IMAGE PREVIEW\n");
            info.push_str("  ──────────────────────────────\n\n");
            info.push_str(&format!("  File: {}\n\n", path.display()));
            
            if let Some(ext) = path.extension() {
                info.push_str(&format!("  Format: {}\n\n", ext.to_string_lossy().to_uppercase()));
            }
            
            if let Ok(metadata) = std::fs::metadata(path) {
                let size = metadata.len();
                let size_str = if size < 1024 {
                    format!("{} B", size)
                } else if size < 1024 * 1024 {
                    format!("{:.2} KB", size as f64 / 1024.0)
                } else {
                    format!("{:.2} MB", size as f64 / (1024.0 * 1024.0))
                };
                info.push_str(&format!("  Size: {}\n", size_str));
            }
            info
        },
        Some(crate::viewer::ViewerContent::ImagePreviewContent(preview)) => {
            let mut info = String::new();
            info.push_str("\n  IMAGE PREVIEW\n");
            info.push_str("  ──────────────────────────────\n");
            info.push_str(&format!("  {}\n\n", preview.metadata()));
            info.push_str(&preview.content);
            info
        },
        Some(crate::viewer::ViewerContent::HexView(data, truncated)) => {
            crate::viewer::format_hex_view(data, *truncated)
        },
        Some(crate::viewer::ViewerContent::Error(e)) => format!("Error: {}", e),
        None => "No content loaded".to_string(),
    };

    let visible_height = area.height as usize;
    let line_num_width = 6; // "1234 │ "
    let content_width = area.width.saturating_sub(line_num_width as u16 + 1) as usize; // -1 for scrollbar

    if app.viewer_wrap_mode && content_width > 10 {
        // Wrap mode: wrap long lines
        let wrapped_lines = wrap_lines_with_numbers(&content_text, content_width);
        let total_lines = wrapped_lines.len();
        
        let visible = wrapped_lines
            .iter()
            .skip(app.viewer_scroll)
            .take(visible_height);

        let mut styled_lines = Vec::new();
        for (line_num_opt, line_content) in visible {
            let line_prefix = match line_num_opt {
                Some(n) => format!("{:>4} │ ", n),
                None => "     │ ".to_string(), // Continuation line
            };
            
            styled_lines.push(ListItem::new(ratatui::text::Line::from(vec![
                ratatui::text::Span::styled(line_prefix, Style::default().fg(theme.border)),
                ratatui::text::Span::raw(line_content.clone()),
            ])));
        }

        let list = List::new(styled_lines);
        f.render_widget(list, area);
        
        draw_scrollbar(f, area, app.viewer_scroll, total_lines, visible_height, theme);
    } else {
        // No wrap: original behavior
        let lines: Vec<&str> = content_text.lines().collect();
        let total_lines = lines.len();
        
        let visible_lines = lines
            .iter()
            .skip(app.viewer_scroll)
            .take(visible_height);

        let mut styled_lines = Vec::new();
        for (i, line) in visible_lines.enumerate() {
            let line_num = app.viewer_scroll + i + 1;
            let line_prefix = format!("{:>4} │ ", line_num);
            
            styled_lines.push(ListItem::new(ratatui::text::Line::from(vec![
                ratatui::text::Span::styled(line_prefix, Style::default().fg(theme.border)),
                ratatui::text::Span::raw(*line),
            ])));
        }

        let list = List::new(styled_lines);
        f.render_widget(list, area);
        
        draw_scrollbar(f, area, app.viewer_scroll, total_lines, visible_height, theme);
    }
}

/// Wrap lines and return (line_number_option, wrapped_text) pairs
/// line_number is Some for first segment of each line, None for continuation
fn wrap_lines_with_numbers(content: &str, max_width: usize) -> Vec<(Option<usize>, String)> {
    let mut result = Vec::new();
    
    for (line_idx, line) in content.lines().enumerate() {
        let line_num = line_idx + 1;
        
        if line.is_empty() {
            result.push((Some(line_num), String::new()));
            continue;
        }
        
        let chars: Vec<char> = line.chars().collect();
        let mut start = 0;
        let mut is_first = true;
        
        while start < chars.len() {
            let end = (start + max_width).min(chars.len());
            let segment: String = chars[start..end].iter().collect();
            
            if is_first {
                result.push((Some(line_num), segment));
                is_first = false;
            } else {
                result.push((None, segment));
            }
            
            start = end;
        }
    }
    
    result
}

/// Wrap text to fit within max_width, breaking at word boundaries when possible
#[allow(dead_code)]
fn wrap_text(text: &str, max_width: usize) -> Vec<String> {
    if max_width == 0 {
        return vec![text.to_string()];
    }
    
    let mut result = Vec::new();
    let mut current_line = String::new();
    let mut current_width = 0;
    
    for word in text.split_whitespace() {
        let word_width = word.chars().count();
        
        if current_width == 0 {
            // First word on line
            if word_width > max_width {
                // Word is too long, need to split it
                let chars: Vec<char> = word.chars().collect();
                let mut i = 0;
                while i < chars.len() {
                    let end = (i + max_width).min(chars.len());
                    let segment: String = chars[i..end].iter().collect();
                    result.push(segment);
                    i = end;
                }
            } else {
                current_line = word.to_string();
                current_width = word_width;
            }
        } else if current_width + 1 + word_width <= max_width {
            // Word fits on current line
            current_line.push(' ');
            current_line.push_str(word);
            current_width += 1 + word_width;
        } else {
            // Need to start a new line
            result.push(std::mem::take(&mut current_line));
            current_width = 0;
            
            if word_width > max_width {
                // Word is too long, need to split it
                let chars: Vec<char> = word.chars().collect();
                let mut i = 0;
                while i < chars.len() {
                    let end = (i + max_width).min(chars.len());
                    let segment: String = chars[i..end].iter().collect();
                    if i + max_width >= chars.len() {
                        // Last segment, keep as current line
                        current_line = segment;
                        current_width = end - i;
                    } else {
                        result.push(segment);
                    }
                    i = end;
                }
            } else {
                current_line = word.to_string();
                current_width = word_width;
            }
        }
    }
    
    // Don't forget the last line
    if !current_line.is_empty() {
        result.push(current_line);
    }
    
    // If text was empty or only whitespace
    if result.is_empty() {
        result.push(String::new());
    }
    
    result
}

// ============================================================================
// Markdown Parsing and Rendering
// ============================================================================

/// Markdown block types
#[allow(dead_code)]
#[derive(Debug, Clone)]
enum MarkdownBlock {
    Paragraph(String),
    CodeBlock { lang: Option<String>, code: String },
    Heading { level: u8, text: String },
    ListItem { ordered: bool, number: Option<usize>, text: String },
    Quote(String),
    HorizontalRule,
}

/// Parse markdown text into blocks
#[allow(dead_code)]
fn parse_markdown(text: &str) -> Vec<MarkdownBlock> {
    let mut blocks = Vec::new();
    let mut lines = text.lines().peekable();
    let mut in_code_block = false;
    let mut code_lang: Option<String> = None;
    let mut code_content = String::new();
    
    while let Some(line) = lines.next() {
        // Handle code blocks
        if line.starts_with("```") {
            if in_code_block {
                // End of code block
                blocks.push(MarkdownBlock::CodeBlock {
                    lang: code_lang.take(),
                    code: code_content.trim_end().to_string(),
                });
                code_content.clear();
                in_code_block = false;
            } else {
                // Start of code block
                in_code_block = true;
                let lang = line[3..].trim();
                code_lang = if lang.is_empty() { None } else { Some(lang.to_string()) };
            }
            continue;
        }
        
        if in_code_block {
            if !code_content.is_empty() {
                code_content.push('\n');
            }
            code_content.push_str(line);
            continue;
        }
        
        // Headings
        if line.starts_with('#') {
            let level = line.chars().take_while(|&c| c == '#').count() as u8;
            if level <= 6 {
                let text = line[level as usize..].trim().to_string();
                blocks.push(MarkdownBlock::Heading { level, text });
                continue;
            }
        }
        
        // Horizontal rule
        if line.trim() == "---" || line.trim() == "***" || line.trim() == "___" {
            blocks.push(MarkdownBlock::HorizontalRule);
            continue;
        }
        
        // Quote
        if line.starts_with('>') {
            let text = line[1..].trim().to_string();
            blocks.push(MarkdownBlock::Quote(text));
            continue;
        }
        
        // Unordered list
        if line.starts_with("- ") || line.starts_with("* ") || line.starts_with("+ ") {
            let text = line[2..].to_string();
            blocks.push(MarkdownBlock::ListItem { ordered: false, number: None, text });
            continue;
        }
        
        // Ordered list
        if let Some(dot_pos) = line.find(". ") {
            if let Ok(num) = line[..dot_pos].trim().parse::<usize>() {
                let text = line[dot_pos + 2..].to_string();
                blocks.push(MarkdownBlock::ListItem { ordered: true, number: Some(num), text });
                continue;
            }
        }
        
        // Regular paragraph (skip empty lines)
        if !line.trim().is_empty() {
            blocks.push(MarkdownBlock::Paragraph(line.to_string()));
        }
    }
    
    // Handle unclosed code block
    if in_code_block && !code_content.is_empty() {
        blocks.push(MarkdownBlock::CodeBlock {
            lang: code_lang,
            code: code_content.trim_end().to_string(),
        });
    }
    
    blocks
}

/// Parse inline formatting and return styled spans
#[allow(dead_code)]
fn parse_inline_markdown(text: &str) -> Vec<ratatui::text::Span<'static>> {
    let mut spans = Vec::new();
    let mut current_text = String::new();
    let mut chars = text.chars().peekable();
    
    while let Some(ch) = chars.next() {
        match ch {
            // Inline code
            '`' => {
                if !current_text.is_empty() {
                    spans.push(ratatui::text::Span::styled(
                        std::mem::take(&mut current_text),
                        Style::default().fg(Color::White)
                    ));
                }
                
                // Collect until closing backtick
                let mut code = String::new();
                while let Some(&next_ch) = chars.peek() {
                    if next_ch == '`' {
                        chars.next();
                        break;
                    }
                    code.push(chars.next().unwrap());
                }
                
                if !code.is_empty() {
                    spans.push(ratatui::text::Span::styled(
                        code,
                        Style::default().fg(Color::Yellow).bg(Color::DarkGray)
                    ));
                }
            }
            // Bold or italic
            '*' | '_' => {
                let marker = ch;
                let is_double = chars.peek() == Some(&marker);
                
                if is_double {
                    chars.next(); // consume second marker
                    
                    if !current_text.is_empty() {
                        spans.push(ratatui::text::Span::styled(
                            std::mem::take(&mut current_text),
                            Style::default().fg(Color::White)
                        ));
                    }
                    
                    // Collect until closing double marker (bold)
                    let mut bold_text = String::new();
                    while let Some(next_ch) = chars.next() {
                        if next_ch == marker && chars.peek() == Some(&marker) {
                            chars.next();
                            break;
                        }
                        bold_text.push(next_ch);
                    }
                    
                    if !bold_text.is_empty() {
                        spans.push(ratatui::text::Span::styled(
                            bold_text,
                            Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
                        ));
                    }
                } else {
                    if !current_text.is_empty() {
                        spans.push(ratatui::text::Span::styled(
                            std::mem::take(&mut current_text),
                            Style::default().fg(Color::White)
                        ));
                    }
                    
                    // Collect until closing single marker (italic)
                    let mut italic_text = String::new();
                    while let Some(next_ch) = chars.next() {
                        if next_ch == marker {
                            break;
                        }
                        italic_text.push(next_ch);
                    }
                    
                    if !italic_text.is_empty() {
                        spans.push(ratatui::text::Span::styled(
                            italic_text,
                            Style::default().fg(Color::White).add_modifier(Modifier::ITALIC)
                        ));
                    }
                }
            }
            _ => {
                current_text.push(ch);
            }
        }
    }
    
    // Don't forget remaining text
    if !current_text.is_empty() {
        spans.push(ratatui::text::Span::styled(
            current_text,
            Style::default().fg(Color::White)
        ));
    }
    
    if spans.is_empty() {
        spans.push(ratatui::text::Span::raw(""));
    }
    
    spans
}

/// Render markdown blocks to ratatui Lines
#[allow(dead_code)]
fn render_markdown_to_lines(
    text: &str,
    max_width: usize,
    indent: usize,
) -> Vec<ratatui::text::Line<'static>> {
    use crate::viewer::{highlight_code, HighlightedLine};
    
    let blocks = parse_markdown(text);
    let mut lines: Vec<ratatui::text::Line<'static>> = Vec::new();
    let indent_str = " ".repeat(indent);
    let content_width = max_width.saturating_sub(indent);
    
    for block in blocks {
        match block {
            MarkdownBlock::Heading { level, text } => {
                let prefix = "#".repeat(level as usize);
                let color = match level {
                    1 => Color::Cyan,
                    2 => Color::LightCyan,
                    _ => Color::Blue,
                };
                
                let mut spans = vec![
                    ratatui::text::Span::styled(
                        format!("{}{} ", indent_str, prefix),
                        Style::default().fg(color).add_modifier(Modifier::BOLD)
                    ),
                ];
                spans.extend(parse_inline_markdown(&text).into_iter().map(|mut s| {
                    s.style = s.style.fg(color).add_modifier(Modifier::BOLD);
                    s
                }));
                lines.push(ratatui::text::Line::from(spans));
            }
            
            MarkdownBlock::CodeBlock { lang, code } => {
                // Code block header
                let lang_display = lang.as_deref().unwrap_or("code");
                lines.push(ratatui::text::Line::from(vec![
                    ratatui::text::Span::styled(
                        format!("{}┌─ {} ", indent_str, lang_display),
                        Style::default().fg(Color::DarkGray)
                    ),
                    ratatui::text::Span::styled(
                        "─".repeat(content_width.saturating_sub(lang_display.len() + 4)),
                        Style::default().fg(Color::DarkGray)
                    ),
                ]));
                
                // Syntax highlighted code
                let ext = lang.as_deref().unwrap_or("txt");
                let highlighted: Vec<HighlightedLine> = highlight_code(&code, ext);
                
                for hl_line in highlighted {
                    let mut spans = vec![
                        ratatui::text::Span::styled(
                            format!("{}│ ", indent_str),
                            Style::default().fg(Color::DarkGray)
                        ),
                    ];
                    
                    for segment in hl_line.segments {
                        spans.push(ratatui::text::Span::styled(
                            segment.text,
                            Style::default().fg(segment.fg)
                        ));
                    }
                    
                    lines.push(ratatui::text::Line::from(spans));
                }
                
                // Code block footer
                lines.push(ratatui::text::Line::from(vec![
                    ratatui::text::Span::styled(
                        format!("{}└", indent_str),
                        Style::default().fg(Color::DarkGray)
                    ),
                    ratatui::text::Span::styled(
                        "─".repeat(content_width.saturating_sub(1)),
                        Style::default().fg(Color::DarkGray)
                    ),
                ]));
            }
            
            MarkdownBlock::ListItem { ordered, number, text } => {
                let marker = if ordered {
                    format!("{}. ", number.unwrap_or(1))
                } else {
                    "• ".to_string()
                };
                
                let mut spans = vec![
                    ratatui::text::Span::styled(
                        format!("{}{}", indent_str, marker),
                        Style::default().fg(Color::Yellow)
                    ),
                ];
                spans.extend(parse_inline_markdown(&text));
                lines.push(ratatui::text::Line::from(spans));
            }
            
            MarkdownBlock::Quote(text) => {
                let wrapped = wrap_text(&text, content_width.saturating_sub(2));
                for wrapped_line in wrapped {
                    lines.push(ratatui::text::Line::from(vec![
                        ratatui::text::Span::styled(
                            format!("{}│ ", indent_str),
                            Style::default().fg(Color::DarkGray)
                        ),
                        ratatui::text::Span::styled(
                            wrapped_line,
                            Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC)
                        ),
                    ]));
                }
            }
            
            MarkdownBlock::HorizontalRule => {
                lines.push(ratatui::text::Line::from(vec![
                    ratatui::text::Span::styled(
                        format!("{}{}", indent_str, "─".repeat(content_width)),
                        Style::default().fg(Color::DarkGray)
                    ),
                ]));
            }
            
            MarkdownBlock::Paragraph(text) => {
                let wrapped = wrap_text(&text, content_width);
                for wrapped_line in wrapped {
                    let spans = parse_inline_markdown(&wrapped_line);
                    let mut line_spans = vec![
                        ratatui::text::Span::raw(indent_str.clone()),
                    ];
                    line_spans.extend(spans);
                    lines.push(ratatui::text::Line::from(line_spans));
                }
            }
        }
    }
    
    lines
}

/// Draw scrollbar for viewer
fn draw_scrollbar(f: &mut Frame, area: ratatui::layout::Rect, scroll: usize, total_lines: usize, visible_height: usize, theme: &crate::config::Theme) {
    if total_lines > visible_height {
        let scrollbar_area = ratatui::layout::Rect {
            x: area.x + area.width.saturating_sub(1),
            y: area.y,
            width: 1,
            height: area.height,
        };
        
        let progress = scroll as f64 / (total_lines.saturating_sub(visible_height).max(1) as f64);
        let thumb_y = (progress * (area.height.saturating_sub(1) as f64)) as u16;
        
        for i in 0..area.height {
            let ch = if i == thumb_y { "█" } else { "│" };
            let style = if i == thumb_y { Style::default().fg(theme.accent_color) } else { Style::default().fg(theme.border) };
            f.render_widget(Paragraph::new(ch).style(style), 
                ratatui::layout::Rect { x: scrollbar_area.x, y: scrollbar_area.y + i, width: 1, height: 1 });
        }
    }
}

/// Render syntax-highlighted code with line numbers
fn draw_highlighted_code(
    f: &mut Frame, 
    app: &App, 
    area: ratatui::layout::Rect, 
    theme: &crate::config::Theme,
    highlighted: &[crate::viewer::HighlightedLine]
) {
    use ratatui::text::{Line, Span};

    let visible_height = area.height as usize;
    let line_num_width = 6;
    let content_width = area.width.saturating_sub(line_num_width as u16 + 1) as usize;

    if app.viewer_wrap_mode && content_width > 10 {
        // Wrap mode for highlighted code
        let wrapped = wrap_highlighted_lines(highlighted, content_width);
        let total_lines = wrapped.len();
        
        let visible = wrapped
            .iter()
            .skip(app.viewer_scroll)
            .take(visible_height);

        let mut styled_lines = Vec::new();
        for (line_num_opt, segments) in visible {
            let line_prefix = match line_num_opt {
                Some(n) => format!("{:>4} │ ", n),
                None => "     │ ".to_string(),
            };
            
            let mut spans = vec![
                Span::styled(line_prefix, Style::default().fg(theme.border)),
            ];
            
            for (text, color) in segments {
                spans.push(Span::styled(text.clone(), Style::default().fg(*color)));
            }
            
            styled_lines.push(ListItem::new(Line::from(spans)));
        }

        let list = List::new(styled_lines);
        f.render_widget(list, area);
        
        draw_scrollbar(f, area, app.viewer_scroll, total_lines, visible_height, theme);
    } else {
        // No wrap mode
        let total_lines = highlighted.len();
        
        let visible_lines = highlighted
            .iter()
            .skip(app.viewer_scroll)
            .take(visible_height);

        let mut styled_lines = Vec::new();
        for (i, line) in visible_lines.enumerate() {
            let line_num = app.viewer_scroll + i + 1;
            let line_prefix = format!("{:>4} │ ", line_num);
            
            let mut spans = vec![
                Span::styled(line_prefix, Style::default().fg(theme.border)),
            ];
            
            for segment in &line.segments {
                spans.push(Span::styled(
                    segment.text.clone(),
                    Style::default().fg(segment.fg),
                ));
            }
            
            styled_lines.push(ListItem::new(Line::from(spans)));
        }

        let list = List::new(styled_lines);
        f.render_widget(list, area);
        
        draw_scrollbar(f, area, app.viewer_scroll, total_lines, visible_height, theme);
    }
}

/// Wrap highlighted lines for wrap mode
/// Returns (line_number_option, segments) where segments are (text, color) pairs
fn wrap_highlighted_lines(
    highlighted: &[crate::viewer::HighlightedLine], 
    max_width: usize
) -> Vec<(Option<usize>, Vec<(String, Color)>)> {
    let mut result = Vec::new();
    
    for (line_idx, line) in highlighted.iter().enumerate() {
        let line_num = line_idx + 1;
        
        // Flatten all segments into one string with color info
        let mut all_chars: Vec<(char, Color)> = Vec::new();
        for segment in &line.segments {
            for ch in segment.text.chars() {
                all_chars.push((ch, segment.fg));
            }
        }
        
        if all_chars.is_empty() {
            result.push((Some(line_num), Vec::new()));
            continue;
        }
        
        let mut start = 0;
        let mut is_first = true;
        
        while start < all_chars.len() {
            let end = (start + max_width).min(all_chars.len());
            
            // Group consecutive chars with same color
            let mut segments: Vec<(String, Color)> = Vec::new();
            let mut current_text = String::new();
            let mut current_color = all_chars[start].1;
            
            for i in start..end {
                let (ch, color) = all_chars[i];
                if color == current_color {
                    current_text.push(ch);
                } else {
                    if !current_text.is_empty() {
                        segments.push((current_text, current_color));
                    }
                    current_text = String::new();
                    current_text.push(ch);
                    current_color = color;
                }
            }
            if !current_text.is_empty() {
                segments.push((current_text, current_color));
            }
            
            if is_first {
                result.push((Some(line_num), segments));
                is_first = false;
            } else {
                result.push((None, segments));
            }
            
            start = end;
        }
    }
    
    result
}

fn draw_editor_content(f: &mut Frame, editor: &crate::viewer::TextEditor, area: ratatui::layout::Rect, theme: &crate::config::Theme) {
    // Calculate visible range
    let content_height = area.height as usize;
    let scroll_offset = if editor.cursor_row >= content_height {
        editor.cursor_row.saturating_sub(content_height - 1)
    } else {
        0
    };

    let mut display_lines = Vec::new();
    for (idx, line) in editor.lines.iter().enumerate().skip(scroll_offset).take(content_height) {
        let line_num = idx + 1;
        let line_prefix = format!("{:>4} │ ", line_num);
        
        let line_content = if idx == editor.cursor_row {
             // Simple highlight for current line
             ratatui::text::Line::from(vec![
                ratatui::text::Span::styled(line_prefix, Style::default().fg(theme.accent_color).add_modifier(Modifier::BOLD)),
                ratatui::text::Span::styled(line, Style::default().bg(theme.selection_bg)),
             ])
        } else {
             ratatui::text::Line::from(vec![
                ratatui::text::Span::styled(line_prefix, Style::default().fg(theme.border)),
                ratatui::text::Span::raw(line),
             ])
        };
        
        display_lines.push(ListItem::new(line_content));
    }

    let list = List::new(display_lines);
    f.render_widget(list, area);

    // Render cursor
    if matches!(editor.mode, crate::viewer::VimMode::Insert | crate::viewer::VimMode::Normal) {
        let line_prefix_len = 7; // "1234 | " is 7 chars
        let cursor_x = area.x + line_prefix_len + editor.cursor_col as u16;
        let cursor_y = area.y + (editor.cursor_row - scroll_offset) as u16;
        
        if cursor_y < area.y + area.height {
            f.set_cursor_position((cursor_x, cursor_y));
        }
    }
}

fn draw_system_monitor(f: &mut Frame, app: &App) {
    let theme = &app.config.theme;
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(f.area());

    let title = Paragraph::new(" SYSTEM MONITOR")
        .style(Style::default().fg(theme.header_fg).bg(theme.header_bg).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::NONE).style(Style::default().bg(theme.header_bg)));
    f.render_widget(title, chunks[0]);

    let mut sys_info = String::new();
    sys_info.push_str(&format!("\n"));
    sys_info.push_str(&format!("  Total Memory: {:>20} KB\n", app.system.sys.total_memory()));
    sys_info.push_str(&format!("  Used Memory:  {:>20} KB\n", app.system.sys.used_memory()));
    sys_info.push_str(&format!("  CPU Usage:    {:>20.2}%\n", app.system.sys.global_cpu_usage()));

    let content = Paragraph::new(sys_info)
        .style(Style::default().fg(theme.fg).bg(theme.bg))
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border))
            .title(" Resources "));
    f.render_widget(content, chunks[1]);

    let status = Paragraph::new(" TAB: Switch Mode | Q: Quit")
        .style(Style::default().fg(theme.footer_fg).bg(theme.footer_bg));
    f.render_widget(status, chunks[2]);
}

fn draw_setup(f: &mut Frame, app: &App) {
    let theme = &app.config.theme;
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(20),
            Constraint::Percentage(60),
            Constraint::Percentage(20),
        ])
        .split(f.area());

    let title = Paragraph::new("WELCOME TO M-REMAKE SETUP")
        .style(Style::default().fg(theme.header_fg).bg(theme.header_bg).add_modifier(Modifier::BOLD))
        .alignment(ratatui::layout::Alignment::Center)
        .block(Block::default().borders(Borders::ALL).style(Style::default().fg(theme.border)));
    f.render_widget(title, chunks[0]);

    let content = Paragraph::new("\n\n  This is the first run setup.\n\n  Press ENTER to complete setup and\n  start using M-REMAKE.\n")
        .style(Style::default().fg(theme.fg).bg(theme.bg))
        .alignment(ratatui::layout::Alignment::Center)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border))
            .title(" Wizard "));
    f.render_widget(content, chunks[1]);
}

fn draw_dialog_popup(f: &mut Frame, app: &App) {
    use crate::app::DialogMode;

    let theme = &app.config.theme;
    let area = centered_rect(60, 30, f.area());

    f.render_widget(ratatui::widgets::Clear, area);

    let (title, text) = match &app.dialog {
        DialogMode::None => return,
        DialogMode::Rename { current_name, new_name } => {
            (
                " RENAME ",
                format!("\n  Current: {}\n  New:     {}\n\n  ENTER: Confirm  |  ESC: Cancel",
                         truncate_path(current_name, 40),
                         truncate_path(new_name, 40))
            )
        },
        DialogMode::Delete { path_name } => {
            (
                " DELETE CONFIRMATION ",
                format!("\n  Delete: {}\n\n  Y: Confirm  |  N/ESC: Cancel",
                         truncate_path(path_name, 40))
            )
        },
        DialogMode::NewFile { name } => {
            (
                " NEW FILE ",
                format!("\n  Name: {}\n\n  ENTER: Create  |  ESC: Cancel",
                         truncate_path(name, 40))
            )
        },
        DialogMode::NewFolder { name } => {
            (
                " NEW FOLDER ",
                format!("\n  Name: {}\n\n  ENTER: Create  |  ESC: Cancel",
                         truncate_path(name, 40))
            )
        },
        DialogMode::Search { query, results } => {
            let results_text = if results.is_empty() {
                if query.is_empty() {
                    "  (Type to search...)".to_string()
                } else {
                    "  No results found".to_string()
                }
            } else {
                results
                    .iter()
                    .take(5)
                    .map(|(path, _)| {
                        let name = path.file_name()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .to_string();
                        format!("  • {}", truncate_path(&name, 45))
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
            };

            (
                " SEARCH ",
                format!("\n  Query: {}\n\n  Results ({} found):\n{}\n\n  ENTER: Jump  |  ESC: Cancel",
                         truncate_path(query, 40),
                         results.len(),
                         results_text)
            )
        },
        DialogMode::Command { input } => {
            (
                " COMMAND MODE ",
                format!("\n  :{}\n\n  ENTER: Execute  |  ESC: Cancel",
                         truncate_path(input, 40))
            )
        },
        DialogMode::QuitConfirm => {
            (
                " QUIT CONFIRMATION ",
                "\n  Are you sure you want to quit?\n\n  Y: Quit  |  N/ESC: Cancel".to_string()
            )
        },
    };

    let para = Paragraph::new(text)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Thick)
            .border_style(Style::default().fg(theme.accent_color))
            .title(title))
        .style(Style::default().fg(theme.fg).bg(theme.bg));

    f.render_widget(para, area);
}

fn draw_settings(f: &mut Frame, app: &App) {
    use crate::app::SettingsTab;
    use ratatui::text::{Line, Span};

    let theme = &app.config.theme;
    let area = centered_rect(75, 80, f.area());
    
    f.render_widget(ratatui::widgets::Clear, area);

    // Main block
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Thick)
        .border_style(Style::default().fg(theme.border))
        .title(" SETTINGS ");
    f.render_widget(block, area);

    // Create layout with tabs
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Tab bar
            Constraint::Min(10),    // Content
            Constraint::Length(3),  // Footer
        ])
        .margin(1)
        .split(area);

    // Tab bar
    let tab_style_active = Style::default().fg(theme.accent_color).add_modifier(Modifier::BOLD);
    let tab_style_inactive = Style::default().fg(theme.fg);
    
    let tabs = Line::from(vec![
        Span::styled(" ", Style::default()),
        Span::styled(
            " [1] Theme ",
            if app.settings_tab == SettingsTab::Theme { tab_style_active } else { tab_style_inactive }
        ),
        Span::styled(" │ ", Style::default().fg(theme.border)),
        Span::styled(
            " [2] Interface ",
            if app.settings_tab == SettingsTab::Interface { tab_style_active } else { tab_style_inactive }
        ),
    ]);
    let tabs_para = Paragraph::new(tabs);
    f.render_widget(tabs_para, layout[0]);

    // Content based on tab
    match app.settings_tab {
        SettingsTab::Theme => {
            draw_settings_theme_tab(f, app, layout[1]);
        }
        SettingsTab::Interface => {
            draw_settings_interface_tab(f, app, layout[1]);
        }
    }

    // Footer
    let footer_text = match app.settings_tab {
        SettingsTab::Theme => " ↑/↓: Select  |  Enter: Apply  |  1-2: Tab  |  ESC: Close",
        SettingsTab::Interface => " ↑/↓: Change Value  |  1-2: Tab  |  ESC: Close",
    };
    let footer = Paragraph::new(footer_text)
        .style(Style::default().fg(theme.footer_fg));
    f.render_widget(footer, layout[2]);
}

fn draw_settings_theme_tab(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    use crate::config::Theme;
    use ratatui::text::{Line, Span};
    
    let theme = &app.config.theme;
    
    let inner_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(8),     // Theme list
            Constraint::Length(5),  // Preview
        ])
        .split(area);

    // Theme list
    let all_themes = Theme::all_themes();
    let items: Vec<ListItem> = all_themes
        .iter()
        .enumerate()
        .map(|(i, t)| {
            let is_current = t.name == app.config.theme.name;
            let is_selected = i == app.settings_theme_index;
            
            let marker = if is_selected { "►" } else { " " };
            let current_marker = if is_current { " ✓" } else { "" };
            
            let style = if is_selected {
                Style::default().fg(theme.selection_fg).bg(theme.selection_bg).add_modifier(Modifier::BOLD)
            } else if is_current {
                Style::default().fg(theme.accent_color)
            } else {
                Style::default().fg(theme.fg)
            };

            let preview_line = Line::from(vec![
                Span::styled(format!(" {} {:2}. ", marker, i + 1), style),
                Span::styled(format!("{:<20}", t.name), style),
                Span::styled("██", Style::default().fg(t.bg)),
                Span::styled("██", Style::default().fg(t.accent_color)),
                Span::styled("██", Style::default().fg(t.directory_fg)),
                Span::styled(current_marker, Style::default().fg(Color::Green)),
            ]);
            
            ListItem::new(preview_line)
        })
        .collect();

    let theme_list = List::new(items)
        .block(Block::default().borders(Borders::NONE));
    f.render_widget(theme_list, inner_layout[0]);

    // Preview section
    let selected_theme = &all_themes[app.settings_theme_index];
    let preview_text = vec![
        Line::from(vec![
            Span::styled(" Preview: ", Style::default().fg(theme.fg)),
            Span::styled(&selected_theme.name, Style::default().fg(selected_theme.accent_color).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled(" ├─ ", Style::default().fg(selected_theme.border)),
            Span::styled("Directory", Style::default().fg(selected_theme.directory_fg)),
            Span::styled("  ", Style::default()),
            Span::styled("File.txt", Style::default().fg(selected_theme.file_fg)),
            Span::styled("  ", Style::default()),
            Span::styled("link →", Style::default().fg(selected_theme.symlink_fg)),
        ]),
    ];
    
    let preview = Paragraph::new(preview_text)
        .style(Style::default().bg(selected_theme.bg))
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(selected_theme.border)));
    f.render_widget(preview, inner_layout[1]);
}

fn draw_settings_interface_tab(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    use ratatui::text::{Line, Span};
    
    let theme = &app.config.theme;
    
    let content_text = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled(" Interface Settings", Style::default().fg(theme.accent_color).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(" Maximum UI Trees: ", Style::default().fg(theme.fg).add_modifier(Modifier::BOLD)),
            Span::styled(format!("{}", app.config.max_ui_trees), Style::default().fg(theme.directory_fg)),
            Span::styled(" (Range: 1-10)", Style::default().fg(theme.footer_fg)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(" ↑/↓ to adjust the number of UI tree panes you can open", Style::default().fg(theme.fg)),
        ]),
        Line::from(vec![
            Span::styled(" Default: 3, Maximum: 10", Style::default().fg(theme.footer_fg)),
        ]),
        Line::from(""),
    ];
    
    let para = Paragraph::new(content_text)
        .style(Style::default().bg(theme.bg));
    f.render_widget(para, area);
}


fn draw_temp_message_popup(f: &mut Frame, app: &App) {
    if let Some((message, _)) = &app.temp_message {
        let theme = &app.config.theme;
        
        // Calculate message width based on content (with padding)
        let message_width = (message.chars().count() as u16 + 8).min(80);
        let area = centered_rect(message_width.max(30), 7, f.area());
        
        // Clear background
        f.render_widget(ratatui::widgets::Clear, area);
        
        // Create message box
        let para = Paragraph::new(message.as_str())
            .style(Style::default()
                .fg(Color::Yellow)
                .bg(theme.bg)
                .add_modifier(Modifier::BOLD))
            .block(Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Thick)
                .border_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
                .title(" Notification "))
            .alignment(ratatui::layout::Alignment::Center);
        
        f.render_widget(para, area);
    }
}

fn draw_shell_popup(f: &mut Frame, app: &App) {
    let theme = &app.config.theme;
    
    // 85% width, 75% height popup for better terminal experience
    let area = centered_rect(85, 75, f.area());
    
    // Clear background
    f.render_widget(ratatui::widgets::Clear, area);
    
    // Main block
    let title = if app.shell.is_running {
        format!(" SHELL - {} ", truncate_path(&app.shell.working_dir.display().to_string(), 50))
    } else {
        " SHELL - Not Running ".to_string()
    };
    
    // Use theme colors for shell popup
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Thick)
        .border_style(Style::default().fg(theme.accent_color))
        .style(Style::default().bg(theme.bg).fg(theme.fg))
        .title(title);
    
    f.render_widget(block.clone(), area);
    
    // Inner area
    let inner_area = area.inner(ratatui::layout::Margin { vertical: 1, horizontal: 1 });
    
    // Split into terminal area and help line
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(3),      // Terminal area
            Constraint::Length(1),   // Help line
        ])
        .split(inner_area);
    
    let terminal_area = layout[0];
    
    // Render terminal content from vt100 parser
    if let Ok(parser) = app.shell.parser.try_lock() {
        let screen = parser.screen();
        
        // Render each row
        for row in 0..terminal_area.height {
            let row_idx = row as usize;
            if row_idx >= screen.size().0 as usize {
                break;
            }
            
            let mut spans: Vec<ratatui::text::Span> = Vec::new();
            let mut current_text = String::new();
            let mut current_style = Style::default();
            
            for col in 0..terminal_area.width {
                let col_idx = col as usize;
                if col_idx >= screen.size().1 as usize {
                    break;
                }
                
                let cell = screen.cell(row_idx as u16, col_idx as u16);
                
                if let Some(cell) = cell {
                    // Convert vt100 colors to ratatui colors with theme support
                    let style = convert_vt100_style(&cell, theme);
                    
                    if style != current_style && !current_text.is_empty() {
                        spans.push(ratatui::text::Span::styled(current_text.clone(), current_style));
                        current_text.clear();
                    }
                    
                    current_style = style;
                    current_text.push(cell.contents().chars().next().unwrap_or(' '));
                } else {
                    current_text.push(' ');
                }
            }
            
            if !current_text.is_empty() {
                spans.push(ratatui::text::Span::styled(current_text, current_style));
            }
            
            let line = ratatui::text::Line::from(spans);
            let para = Paragraph::new(line);
            f.render_widget(para, ratatui::layout::Rect {
                x: terminal_area.x,
                y: terminal_area.y + row,
                width: terminal_area.width,
                height: 1,
            });
        }
        
        // Set cursor position from vt100 screen
        let (cursor_row, cursor_col) = screen.cursor_position();
        let cursor_x = terminal_area.x + cursor_col;
        let cursor_y = terminal_area.y + cursor_row;
        
        if cursor_x < terminal_area.x + terminal_area.width && 
           cursor_y < terminal_area.y + terminal_area.height {
            f.set_cursor_position((cursor_x, cursor_y));
        }
    }
    
    // Help line
    let help_text = " Full PTY Shell | F12/`:Close | All keys forwarded to shell ";
    let help_para = Paragraph::new(help_text)
        .style(Style::default().fg(theme.footer_fg).bg(theme.footer_bg));
    f.render_widget(help_para, layout[1]);
}

/// Convert vt100 cell style to ratatui Style with theme support
fn convert_vt100_style(cell: &vt100::Cell, theme: &crate::config::Theme) -> Style {
    let mut style = Style::default();
    
    // Foreground color - use theme.fg for default
    style = style.fg(convert_vt100_color(cell.fgcolor(), theme.fg));
    
    // Background color - use theme.bg for default
    style = style.bg(convert_vt100_color(cell.bgcolor(), theme.bg));
    
    // Attributes
    if cell.bold() {
        style = style.add_modifier(Modifier::BOLD);
    }
    if cell.italic() {
        style = style.add_modifier(Modifier::ITALIC);
    }
    if cell.underline() {
        style = style.add_modifier(Modifier::UNDERLINED);
    }
    if cell.inverse() {
        style = style.add_modifier(Modifier::REVERSED);
    }
    
    style
}

/// Convert vt100 color to ratatui Color with theme default fallback
fn convert_vt100_color(color: vt100::Color, default_color: Color) -> Color {
    match color {
        vt100::Color::Default => default_color,  // Use theme color instead of Reset
        vt100::Color::Idx(idx) => {
            // Standard 16 colors
            match idx {
                0 => Color::Black,
                1 => Color::Red,
                2 => Color::Green,
                3 => Color::Yellow,
                4 => Color::Blue,
                5 => Color::Magenta,
                6 => Color::Cyan,
                7 => Color::Gray,
                8 => Color::DarkGray,
                9 => Color::LightRed,
                10 => Color::LightGreen,
                11 => Color::LightYellow,
                12 => Color::LightBlue,
                13 => Color::LightMagenta,
                14 => Color::LightCyan,
                15 => Color::White,
                _ => Color::Indexed(idx),
            }
        }
        vt100::Color::Rgb(r, g, b) => Color::Rgb(r, g, b),
    }
}

fn draw_process_viewer_popup(f: &mut Frame, app: &App) {
    let theme = &app.config.theme;
    
    // 90% width, 85% height popup
    let area = centered_rect(90, 85, f.area());
    
    // Clear background
    f.render_widget(ratatui::widgets::Clear, area);
    
    // Main block - use theme colors
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Thick)
        .border_style(Style::default().fg(theme.accent_color))
        .style(Style::default().bg(theme.bg).fg(theme.fg))
        .title(" PROCESS TREE VIEWER ");
    
    f.render_widget(block.clone(), area);
    
    // Inner area
    let inner_area = area.inner(ratatui::layout::Margin { vertical: 1, horizontal: 1 });
    
    // Layout: Header, Process List, Details, Footer
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),   // Header (filter, sort, search)
            Constraint::Min(10),     // Process list
            Constraint::Length(6),   // Details panel
            Constraint::Length(1),   // Footer (help)
        ])
        .split(inner_area);
    
    // Header
    draw_process_header(f, app, layout[0]);
    
    // Process list
    draw_process_list(f, app, layout[1]);
    
    // Details panel
    draw_process_details(f, app, layout[2]);
    
    // Footer
    let footer_text = " ↑↓:Navigate  t:Toggle Tree  k:Kill  K:Force Kill  p:Parent  f:Filter  s:Sort  /:Search  F9/ESC:Close ";
    let footer = Paragraph::new(footer_text)
        .style(Style::default().fg(theme.footer_fg).bg(theme.footer_bg));
    f.render_widget(footer, layout[3]);
}

fn draw_process_header(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let viewer = &app.process_viewer;
    
    let header_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(25),  // Filter
            Constraint::Length(20),  // Sort
            Constraint::Min(20),     // Search
            Constraint::Length(15),  // Process count
        ])
        .split(area);
    
    // Filter
    let filter_text = format!(" Filter: [{}] ", viewer.filter.as_str());
    let filter = Paragraph::new(filter_text)
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD));
    f.render_widget(filter, header_layout[0]);
    
    // Sort
    let sort_arrow = if viewer.sort_ascending { "↑" } else { "↓" };
    let sort_text = format!(" Sort: [{}{}] ", viewer.sort_by.as_str(), sort_arrow);
    let sort = Paragraph::new(sort_text)
        .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));
    f.render_widget(sort, header_layout[1]);
    
    // Search
    let search_text = if viewer.search_mode {
        format!(" Search: {}_ ", viewer.search_query)
    } else if viewer.search_query.is_empty() {
        " Press / to search ".to_string()
    } else {
        format!(" Search: {} ", viewer.search_query)
    };
    let search_style = if viewer.search_mode {
        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    let search = Paragraph::new(search_text).style(search_style);
    f.render_widget(search, header_layout[2]);
    
    // Process count
    let count_text = format!(" {} procs ", viewer.tree_order.len());
    let count = Paragraph::new(count_text)
        .style(Style::default().fg(Color::White))
        .alignment(ratatui::layout::Alignment::Right);
    f.render_widget(count, header_layout[3]);
}

fn draw_process_list(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let viewer = &app.process_viewer;
    
    // Header row
    let header_height = 1;
    let list_area = ratatui::layout::Rect {
        x: area.x,
        y: area.y + header_height,
        width: area.width,
        height: area.height.saturating_sub(header_height),
    };
    
    // Column header
    let col_header = "  PID      NAME                           CPU%    MEM%    MEMORY     STATUS";
    let header_para = Paragraph::new(col_header)
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD | Modifier::UNDERLINED));
    f.render_widget(header_para, ratatui::layout::Rect {
        x: area.x,
        y: area.y,
        width: area.width,
        height: 1,
    });
    
    // Calculate visible height
    let visible_height = list_area.height as usize;
    
    // Get visible processes
    let visible = viewer.visible_processes(visible_height);
    
    for (row_idx, (global_idx, process)) in visible.iter().enumerate() {
        let is_selected = *global_idx == viewer.selected_index;
        let depth = viewer.get_depth(process.pid);
        
        // Tree prefix
        let tree_prefix = if depth > 0 {
            let mut prefix = String::new();
            for _ in 0..depth.saturating_sub(1) {
                prefix.push_str("│ ");
            }
            if process.children.is_empty() {
                prefix.push_str("└─");
            } else if process.is_expanded {
                prefix.push_str("├─");
            } else {
                prefix.push_str("├►");
            }
            prefix
        } else {
            String::new()
        };
        
        // Format process line (UTF-8 safe truncation)
        let name = if process.name.chars().count() > 25 {
            truncate_str(&process.name, 25)
        } else {
            process.name.clone()
        };
        
        let line = format!(
            "{:>6} {}{:<28} {:>5.1}%  {:>5.1}%  {:>8}  {}",
            process.pid,
            tree_prefix,
            name,
            process.cpu_usage,
            process.memory_percent,
            process.format_memory(),
            process.status
        );
        
        // Style based on selection and CPU usage
        let style = if is_selected {
            Style::default().bg(Color::Rgb(60, 60, 100)).fg(Color::White).add_modifier(Modifier::BOLD)
        } else if process.cpu_usage > 50.0 {
            Style::default().fg(Color::Red)
        } else if process.cpu_usage > 20.0 {
            Style::default().fg(Color::Yellow)
        } else if process.cpu_usage > 5.0 {
            Style::default().fg(Color::Green)
        } else {
            Style::default().fg(Color::Gray)
        };
        
        let line_para = Paragraph::new(line).style(style);
        f.render_widget(line_para, ratatui::layout::Rect {
            x: list_area.x,
            y: list_area.y + row_idx as u16,
            width: list_area.width,
            height: 1,
        });
    }
}

fn draw_process_details(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let viewer = &app.process_viewer;
    
    let block = Block::default()
        .borders(Borders::TOP)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(" Process Details ");
    
    let inner = block.inner(area);
    f.render_widget(block, area);
    
    if let Some(process) = viewer.selected_process() {
        // Layout for details
        let detail_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),  // Name and PID
                Constraint::Length(1),  // Command
                Constraint::Length(1),  // CPU bar
                Constraint::Length(1),  // Memory bar
                Constraint::Length(1),  // CPU history sparkline
            ])
            .split(inner);
        
        // Name and PID
        let info_text = format!(
            " {} (PID: {}) | Parent: {} | User: {} | Started: {}s ago",
            process.name,
            process.pid,
            process.parent_pid.map(|p| p.to_string()).unwrap_or_else(|| "-".to_string()),
            process.user.as_deref().unwrap_or("-"),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs().saturating_sub(process.start_time))
                .unwrap_or(0)
        );
        let info = Paragraph::new(info_text)
            .style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD));
        f.render_widget(info, detail_layout[0]);
        
        // Command
        let cmd_text = format!(" CMD: {}", process.cmd.join(" "));
        let cmd_display = if cmd_text.len() > detail_layout[1].width as usize {
            format!("{}...", &cmd_text[..detail_layout[1].width as usize - 3])
        } else {
            cmd_text
        };
        let cmd = Paragraph::new(cmd_display).style(Style::default().fg(Color::DarkGray));
        f.render_widget(cmd, detail_layout[1]);
        
        // CPU bar
        let cpu_bar_width = 30;
        let cpu_filled = ((process.cpu_usage / 100.0) * cpu_bar_width as f32) as usize;
        let cpu_bar: String = "█".repeat(cpu_filled.min(cpu_bar_width)) + 
                             &"░".repeat(cpu_bar_width.saturating_sub(cpu_filled));
        let cpu_text = format!(" CPU: {} {:>5.1}%", cpu_bar, process.cpu_usage);
        let cpu_color = if process.cpu_usage > 80.0 { Color::Red }
                       else if process.cpu_usage > 50.0 { Color::Yellow }
                       else { Color::Green };
        let cpu_para = Paragraph::new(cpu_text).style(Style::default().fg(cpu_color));
        f.render_widget(cpu_para, detail_layout[2]);
        
        // Memory bar
        let mem_bar_width = 30;
        let mem_filled = ((process.memory_percent / 100.0) * mem_bar_width as f32) as usize;
        let mem_bar: String = "█".repeat(mem_filled.min(mem_bar_width)) + 
                             &"░".repeat(mem_bar_width.saturating_sub(mem_filled));
        let mem_text = format!(" MEM: {} {:>5.1}% ({})", mem_bar, process.memory_percent, process.format_memory());
        let mem_color = if process.memory_percent > 80.0 { Color::Red }
                       else if process.memory_percent > 50.0 { Color::Yellow }
                       else { Color::Cyan };
        let mem_para = Paragraph::new(mem_text).style(Style::default().fg(mem_color));
        f.render_widget(mem_para, detail_layout[3]);
        
        // CPU and MEM history sparklines
        let sparkline_chars = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
        
        let mut history_parts: Vec<ratatui::text::Span> = Vec::new();
        
        // CPU history
        if !process.cpu_history.is_empty() {
            let max_cpu = process.cpu_history.iter().cloned().fold(0.1f32, f32::max);
            let cpu_sparkline: String = process.cpu_history.iter()
                .map(|&v| {
                    let normalized = (v / max_cpu).min(1.0);
                    let idx = (normalized * 7.0) as usize;
                    sparkline_chars[idx.min(7)]
                })
                .collect();
            
            history_parts.push(ratatui::text::Span::raw(" CPU: "));
            history_parts.push(ratatui::text::Span::styled(cpu_sparkline, Style::default().fg(Color::Green)));
        }
        
        // MEM history  
        if !process.mem_history.is_empty() {
            let max_mem = process.mem_history.iter().cloned().fold(0.1f32, f32::max);
            let mem_sparkline: String = process.mem_history.iter()
                .map(|&v| {
                    let normalized = (v / max_mem).min(1.0);
                    let idx = (normalized * 7.0) as usize;
                    sparkline_chars[idx.min(7)]
                })
                .collect();
            
            history_parts.push(ratatui::text::Span::raw("  MEM: "));
            history_parts.push(ratatui::text::Span::styled(mem_sparkline, Style::default().fg(Color::Cyan)));
        }
        
        if !history_parts.is_empty() {
            history_parts.insert(0, ratatui::text::Span::raw(format!(" History ({} samples):", process.cpu_history.len())));
            let history_line = ratatui::text::Line::from(history_parts);
            let history = Paragraph::new(history_line);
            f.render_widget(history, detail_layout[4]);
        }
    } else {
        let no_selection = Paragraph::new(" No process selected")
            .style(Style::default().fg(Color::DarkGray));
        f.render_widget(no_selection, inner);
    }
}
