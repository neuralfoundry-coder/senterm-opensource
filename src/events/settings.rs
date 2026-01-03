//! Settings mode event handling

use crossterm::event::{KeyCode, KeyModifiers};
use crate::app::{App, SettingsTab};
use crate::config::Theme;

/// Handle settings mode key events
pub fn handle_settings_keys(app: &mut App, key_code: KeyCode, _modifiers: KeyModifiers) {
    // Tab switching with 1 and 2
    match key_code {
        KeyCode::Char('1') => {
            app.settings_tab = SettingsTab::Theme;
            return;
        },
        KeyCode::Char('2') => {
            app.settings_tab = SettingsTab::Interface;
            return;
        },
        _ => {}
    }
    
    // Handle based on current tab
    match app.settings_tab {
        SettingsTab::Theme => {
            handle_settings_theme_keys(app, key_code);
        },
        SettingsTab::Interface => {
            handle_settings_interface_keys(app, key_code);
        }
    }
}

/// Handle theme settings keys
fn handle_settings_theme_keys(app: &mut App, key_code: KeyCode) {
    let all_themes = Theme::all_themes();
    let theme_count = all_themes.len();

    match key_code {
        KeyCode::Up | KeyCode::Char('k') => {
            if app.settings_theme_index > 0 {
                app.settings_theme_index -= 1;
            } else {
                app.settings_theme_index = theme_count - 1;
            }
        },
        KeyCode::Down | KeyCode::Char('j') => {
            if app.settings_theme_index < theme_count - 1 {
                app.settings_theme_index += 1;
            } else {
                app.settings_theme_index = 0;
            }
        },
        KeyCode::Enter => {
            if let Some(theme) = all_themes.into_iter().nth(app.settings_theme_index) {
                app.config.theme = theme;
                let _ = app.config.save();
                app.status_message = Some(format!("Theme changed to: {}", app.config.theme.name));
            }
        },
        KeyCode::Char(c) if c.is_ascii_digit() && c != '1' && c != '2' => {
            // Quick select themes 3-9, 0
            let index = if c == '0' { 9 } else { (c as usize) - ('1' as usize) };
            if index < theme_count && index >= 2 {
                app.settings_theme_index = index;
                if let Some(theme) = Theme::all_themes().into_iter().nth(index) {
                    app.config.theme = theme;
                    let _ = app.config.save();
                    app.status_message = Some(format!("Theme changed to: {}", app.config.theme.name));
                }
            }
        },
        _ => {}
    }
}

/// Handle interface settings keys
fn handle_settings_interface_keys(app: &mut App, key_code: KeyCode) {
    match key_code {
        KeyCode::Up | KeyCode::Char('k') => {
            if app.config.max_ui_trees < 10 {
                app.config.max_ui_trees += 1;
                let _ = app.config.save();
                app.status_message = Some(format!("Maximum UI trees set to {}", app.config.max_ui_trees));
            } else {
                app.status_message = Some("Maximum limit reached (10 trees)".to_string());
            }
        },
        KeyCode::Down | KeyCode::Char('j') => {
            if app.config.max_ui_trees > 1 {
                app.config.max_ui_trees -= 1;
                let _ = app.config.save();
                app.status_message = Some(format!("Maximum UI trees set to {}", app.config.max_ui_trees));
                
                // If current pane count exceeds new limit, reduce it
                let max_panes = app.config.max_ui_trees;
                if app.pane_count > max_panes {
                    app.pane_count = max_panes;
                    // Adjust active pane if it no longer exists
                    match (app.pane_count, app.active_pane) {
                        (1, crate::app::Pane::Center) | (1, crate::app::Pane::Right) => app.active_pane = crate::app::Pane::Left,
                        (2, crate::app::Pane::Right) => app.active_pane = crate::app::Pane::Center,
                        _ => {}
                    }
                    app.status_message = Some(format!("Reduced to {} panes and max trees to {}", app.pane_count, app.config.max_ui_trees));
                }
            } else {
                app.status_message = Some("Minimum limit reached (1 tree)".to_string());
            }
        },
        _ => {}
    }
}
