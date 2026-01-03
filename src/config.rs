use serde::{Deserialize, Serialize};
use ratatui::style::Color;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SortOption {
    Name,
    Size,
    Modified,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub name: String,
    pub bg: Color,
    pub fg: Color,
    pub selection_bg: Color,
    pub selection_fg: Color,
    pub border: Color,
    
    // Semantic fields
    pub header_bg: Color,
    pub header_fg: Color,
    pub footer_bg: Color,
    pub footer_fg: Color,
    pub directory_fg: Color,
    pub file_fg: Color,
    pub symlink_fg: Color,
    pub executable_fg: Color,
    pub accent_color: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self::elegant_dark() // Default to Elegant Dark
    }
}

impl Theme {
    /// Get all available themes
    pub fn all_themes() -> Vec<Theme> {
        vec![
            Self::elegant_dark(),
            Self::elegant_light(),
            Self::monokai(),
            Self::dracula(),
            Self::solarized_dark(),
            Self::solarized_light(),
            Self::nord(),
            Self::gruvbox_dark(),
            Self::one_dark(),
            Self::tokyo_night(),
        ]
    }

    /// Get theme by name
    #[allow(dead_code)]
    pub fn by_name(name: &str) -> Option<Theme> {
        Self::all_themes().into_iter().find(|t| t.name == name)
    }

    pub fn elegant_dark() -> Self {
        Self {
            name: "Elegant Dark".to_string(),
            bg: Color::Rgb(20, 24, 32),
            fg: Color::Rgb(220, 220, 230),
            selection_bg: Color::Rgb(45, 55, 75),
            selection_fg: Color::White,
            border: Color::Rgb(60, 70, 90),
            header_bg: Color::Rgb(30, 35, 45),
            header_fg: Color::Rgb(220, 220, 230),
            footer_bg: Color::Rgb(15, 20, 25),
            footer_fg: Color::Rgb(150, 160, 180),
            directory_fg: Color::Rgb(110, 170, 250),
            file_fg: Color::Rgb(220, 220, 230),
            symlink_fg: Color::Rgb(100, 220, 210),
            executable_fg: Color::Rgb(120, 230, 120),
            accent_color: Color::Rgb(255, 190, 80),
        }
    }

    pub fn elegant_light() -> Self {
        Self {
            name: "Elegant Light".to_string(),
            bg: Color::Rgb(250, 250, 250),
            fg: Color::Rgb(40, 44, 52),
            selection_bg: Color::Rgb(220, 230, 245),
            selection_fg: Color::Black,
            border: Color::Rgb(200, 200, 210),
            header_bg: Color::Rgb(240, 242, 245),
            header_fg: Color::Rgb(40, 44, 52),
            footer_bg: Color::Rgb(235, 235, 240),
            footer_fg: Color::Rgb(80, 85, 95),
            directory_fg: Color::Rgb(0, 90, 200),
            file_fg: Color::Rgb(40, 44, 52),
            symlink_fg: Color::Rgb(0, 140, 140),
            executable_fg: Color::Rgb(0, 140, 0),
            accent_color: Color::Rgb(230, 110, 0),
        }
    }

    pub fn monokai() -> Self {
        Self {
            name: "Monokai".to_string(),
            bg: Color::Rgb(39, 40, 34),
            fg: Color::Rgb(248, 248, 242),
            selection_bg: Color::Rgb(73, 72, 62),
            selection_fg: Color::Rgb(248, 248, 242),
            border: Color::Rgb(117, 113, 94),
            header_bg: Color::Rgb(49, 50, 44),
            header_fg: Color::Rgb(248, 248, 242),
            footer_bg: Color::Rgb(29, 30, 24),
            footer_fg: Color::Rgb(117, 113, 94),
            directory_fg: Color::Rgb(102, 217, 239),  // Cyan
            file_fg: Color::Rgb(248, 248, 242),
            symlink_fg: Color::Rgb(174, 129, 255),    // Purple
            executable_fg: Color::Rgb(166, 226, 46),  // Green
            accent_color: Color::Rgb(249, 38, 114),   // Pink
        }
    }

    pub fn dracula() -> Self {
        Self {
            name: "Dracula".to_string(),
            bg: Color::Rgb(40, 42, 54),
            fg: Color::Rgb(248, 248, 242),
            selection_bg: Color::Rgb(68, 71, 90),
            selection_fg: Color::Rgb(248, 248, 242),
            border: Color::Rgb(98, 114, 164),
            header_bg: Color::Rgb(50, 52, 64),
            header_fg: Color::Rgb(248, 248, 242),
            footer_bg: Color::Rgb(30, 32, 44),
            footer_fg: Color::Rgb(98, 114, 164),
            directory_fg: Color::Rgb(139, 233, 253),  // Cyan
            file_fg: Color::Rgb(248, 248, 242),
            symlink_fg: Color::Rgb(255, 121, 198),    // Pink
            executable_fg: Color::Rgb(80, 250, 123),  // Green
            accent_color: Color::Rgb(189, 147, 249),  // Purple
        }
    }

    pub fn solarized_dark() -> Self {
        Self {
            name: "Solarized Dark".to_string(),
            bg: Color::Rgb(0, 43, 54),
            fg: Color::Rgb(131, 148, 150),
            selection_bg: Color::Rgb(7, 54, 66),
            selection_fg: Color::Rgb(147, 161, 161),
            border: Color::Rgb(88, 110, 117),
            header_bg: Color::Rgb(7, 54, 66),
            header_fg: Color::Rgb(147, 161, 161),
            footer_bg: Color::Rgb(0, 43, 54),
            footer_fg: Color::Rgb(88, 110, 117),
            directory_fg: Color::Rgb(38, 139, 210),   // Blue
            file_fg: Color::Rgb(131, 148, 150),
            symlink_fg: Color::Rgb(42, 161, 152),     // Cyan
            executable_fg: Color::Rgb(133, 153, 0),   // Green
            accent_color: Color::Rgb(203, 75, 22),    // Orange
        }
    }

    pub fn solarized_light() -> Self {
        Self {
            name: "Solarized Light".to_string(),
            bg: Color::Rgb(253, 246, 227),
            fg: Color::Rgb(101, 123, 131),
            selection_bg: Color::Rgb(238, 232, 213),
            selection_fg: Color::Rgb(88, 110, 117),
            border: Color::Rgb(147, 161, 161),
            header_bg: Color::Rgb(238, 232, 213),
            header_fg: Color::Rgb(88, 110, 117),
            footer_bg: Color::Rgb(253, 246, 227),
            footer_fg: Color::Rgb(147, 161, 161),
            directory_fg: Color::Rgb(38, 139, 210),   // Blue
            file_fg: Color::Rgb(101, 123, 131),
            symlink_fg: Color::Rgb(42, 161, 152),     // Cyan
            executable_fg: Color::Rgb(133, 153, 0),   // Green
            accent_color: Color::Rgb(203, 75, 22),    // Orange
        }
    }

    pub fn nord() -> Self {
        Self {
            name: "Nord".to_string(),
            bg: Color::Rgb(46, 52, 64),
            fg: Color::Rgb(216, 222, 233),
            selection_bg: Color::Rgb(67, 76, 94),
            selection_fg: Color::Rgb(236, 239, 244),
            border: Color::Rgb(76, 86, 106),
            header_bg: Color::Rgb(59, 66, 82),
            header_fg: Color::Rgb(229, 233, 240),
            footer_bg: Color::Rgb(46, 52, 64),
            footer_fg: Color::Rgb(76, 86, 106),
            directory_fg: Color::Rgb(136, 192, 208),  // Frost Blue
            file_fg: Color::Rgb(216, 222, 233),
            symlink_fg: Color::Rgb(180, 142, 173),    // Purple
            executable_fg: Color::Rgb(163, 190, 140), // Green
            accent_color: Color::Rgb(235, 203, 139),  // Yellow
        }
    }

    pub fn gruvbox_dark() -> Self {
        Self {
            name: "Gruvbox Dark".to_string(),
            bg: Color::Rgb(40, 40, 40),
            fg: Color::Rgb(235, 219, 178),
            selection_bg: Color::Rgb(80, 73, 69),
            selection_fg: Color::Rgb(251, 241, 199),
            border: Color::Rgb(102, 92, 84),
            header_bg: Color::Rgb(50, 48, 47),
            header_fg: Color::Rgb(235, 219, 178),
            footer_bg: Color::Rgb(29, 32, 33),
            footer_fg: Color::Rgb(146, 131, 116),
            directory_fg: Color::Rgb(131, 165, 152),  // Aqua
            file_fg: Color::Rgb(235, 219, 178),
            symlink_fg: Color::Rgb(211, 134, 155),    // Purple
            executable_fg: Color::Rgb(184, 187, 38),  // Green
            accent_color: Color::Rgb(254, 128, 25),   // Orange
        }
    }

    pub fn one_dark() -> Self {
        Self {
            name: "One Dark".to_string(),
            bg: Color::Rgb(40, 44, 52),
            fg: Color::Rgb(171, 178, 191),
            selection_bg: Color::Rgb(62, 68, 81),
            selection_fg: Color::Rgb(198, 205, 218),
            border: Color::Rgb(76, 82, 99),
            header_bg: Color::Rgb(50, 54, 62),
            header_fg: Color::Rgb(171, 178, 191),
            footer_bg: Color::Rgb(33, 37, 43),
            footer_fg: Color::Rgb(92, 99, 112),
            directory_fg: Color::Rgb(97, 175, 239),   // Blue
            file_fg: Color::Rgb(171, 178, 191),
            symlink_fg: Color::Rgb(198, 120, 221),    // Purple
            executable_fg: Color::Rgb(152, 195, 121), // Green
            accent_color: Color::Rgb(229, 192, 123),  // Yellow
        }
    }

    pub fn tokyo_night() -> Self {
        Self {
            name: "Tokyo Night".to_string(),
            bg: Color::Rgb(26, 27, 38),
            fg: Color::Rgb(169, 177, 214),
            selection_bg: Color::Rgb(52, 59, 88),
            selection_fg: Color::Rgb(192, 202, 245),
            border: Color::Rgb(65, 72, 104),
            header_bg: Color::Rgb(36, 40, 59),
            header_fg: Color::Rgb(169, 177, 214),
            footer_bg: Color::Rgb(22, 22, 30),
            footer_fg: Color::Rgb(86, 95, 137),
            directory_fg: Color::Rgb(122, 162, 247),  // Blue
            file_fg: Color::Rgb(169, 177, 214),
            symlink_fg: Color::Rgb(187, 154, 247),    // Purple
            executable_fg: Color::Rgb(158, 206, 106), // Green
            accent_color: Color::Rgb(224, 175, 104),  // Orange
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub theme: Theme,
    pub show_parent_dirs: usize, // How many parent levels to show
    pub first_run: bool,
    pub bookmarks: Vec<PathBuf>, // Bookmarked directories
    pub sort_option: SortOption, // File sorting option
    #[serde(default = "default_max_ui_trees")]
    pub max_ui_trees: usize, // Maximum number of UI trees (default 3, max 10)
}

fn default_max_ui_trees() -> usize {
    3
}

impl Default for Config {
    fn default() -> Self {
        Self {
            theme: Theme::default(),
            show_parent_dirs: 5,
            first_run: false,
            bookmarks: Vec::new(),
            sort_option: SortOption::Name,
            max_ui_trees: default_max_ui_trees(),
        }
    }
}

impl Config {
    /// Get the config file path (~/.config/senterm/config.toml)
    fn config_path() -> Option<std::path::PathBuf> {
        if let Some(config_dir) = dirs::config_dir() {
            let app_config_dir = config_dir.join("senterm");
            Some(app_config_dir.join("config.toml"))
        } else {
            None
        }
    }

    /// Load configuration from file, or return default if file doesn't exist
    pub fn load() -> Self {
        if let Some(config_path) = Self::config_path() {
            if config_path.exists() {
                if let Ok(contents) = std::fs::read_to_string(&config_path) {
                    if let Ok(config) = toml::from_str::<Config>(&contents) {
                        tracing::info!("Loaded config from {:?}", config_path);
                        return config;
                    } else {
                        tracing::warn!("Failed to parse config file, using defaults");
                    }
                } else {
                    tracing::warn!("Failed to read config file, using defaults");
                }
            } else {
                tracing::info!("Config file not found, using defaults");
            }
        }
        Self::default()
    }

    /// Save configuration to file
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(config_path) = Self::config_path() {
            // Ensure directory exists
            if let Some(parent) = config_path.parent() {
                std::fs::create_dir_all(parent)?;
            }

            let toml_string = toml::to_string_pretty(self)?;
            std::fs::write(&config_path, toml_string)?;
            tracing::info!("Saved config to {:?}", config_path);
            Ok(())
        } else {
            Err("Could not determine config directory".into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sort_option_variants() {
        let name = SortOption::Name;
        let size = SortOption::Size;
        let modified = SortOption::Modified;

        assert_eq!(name, SortOption::Name);
        assert_eq!(size, SortOption::Size);
        assert_eq!(modified, SortOption::Modified);
        assert_ne!(name, size);
    }

    #[test]
    fn test_theme_default() {
        let theme = Theme::default();
        assert_eq!(theme.name, "Elegant Dark");
    }

    #[test]
    fn test_all_themes_count() {
        let themes = Theme::all_themes();
        assert_eq!(themes.len(), 10);
    }

    #[test]
    fn test_all_themes_unique_names() {
        let themes = Theme::all_themes();
        let names: Vec<_> = themes.iter().map(|t| &t.name).collect();
        
        // Check all names are unique
        let mut unique_names = names.clone();
        unique_names.sort();
        unique_names.dedup();
        assert_eq!(names.len(), unique_names.len(), "Theme names should be unique");
    }

    #[test]
    fn test_theme_by_name_found() {
        let theme = Theme::by_name("Dracula");
        assert!(theme.is_some());
        assert_eq!(theme.unwrap().name, "Dracula");
    }

    #[test]
    fn test_theme_by_name_not_found() {
        let theme = Theme::by_name("NonExistentTheme");
        assert!(theme.is_none());
    }

    #[test]
    fn test_theme_by_name_all_themes() {
        let expected_names = [
            "Elegant Dark",
            "Elegant Light",
            "Monokai",
            "Dracula",
            "Solarized Dark",
            "Solarized Light",
            "Nord",
            "Gruvbox Dark",
            "One Dark",
            "Tokyo Night",
        ];

        for name in expected_names {
            let theme = Theme::by_name(name);
            assert!(theme.is_some(), "Theme '{}' should exist", name);
        }
    }

    #[test]
    fn test_theme_elegant_dark() {
        let theme = Theme::elegant_dark();
        assert_eq!(theme.name, "Elegant Dark");
        // Verify it has valid colors (not checking exact values, just structure)
        assert!(matches!(theme.bg, Color::Rgb(_, _, _)));
    }

    #[test]
    fn test_theme_tokyo_night() {
        let theme = Theme::tokyo_night();
        assert_eq!(theme.name, "Tokyo Night");
    }

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert_eq!(config.theme.name, "Elegant Dark");
        assert_eq!(config.show_parent_dirs, 5);
        assert!(!config.first_run);
        assert!(config.bookmarks.is_empty());
        assert_eq!(config.sort_option, SortOption::Name);
        assert_eq!(config.max_ui_trees, 3);
    }

    #[test]
    fn test_default_max_ui_trees() {
        assert_eq!(default_max_ui_trees(), 3);
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let toml_string = toml::to_string_pretty(&config);
        assert!(toml_string.is_ok());
        
        let serialized = toml_string.unwrap();
        assert!(serialized.contains("theme"));
        assert!(serialized.contains("Elegant Dark"));
    }

    #[test]
    fn test_config_deserialization() {
        let toml_str = r#"
            show_parent_dirs = 3
            first_run = true
            sort_option = "Size"
            max_ui_trees = 5
            bookmarks = []

            [theme]
            name = "Dracula"
            bg = { Rgb = [40, 42, 54] }
            fg = { Rgb = [248, 248, 242] }
            selection_bg = { Rgb = [68, 71, 90] }
            selection_fg = { Rgb = [248, 248, 242] }
            border = { Rgb = [98, 114, 164] }
            header_bg = { Rgb = [50, 52, 64] }
            header_fg = { Rgb = [248, 248, 242] }
            footer_bg = { Rgb = [30, 32, 44] }
            footer_fg = { Rgb = [98, 114, 164] }
            directory_fg = { Rgb = [139, 233, 253] }
            file_fg = { Rgb = [248, 248, 242] }
            symlink_fg = { Rgb = [255, 121, 198] }
            executable_fg = { Rgb = [80, 250, 123] }
            accent_color = { Rgb = [189, 147, 249] }
        "#;

        let config: Result<Config, _> = toml::from_str(toml_str);
        assert!(config.is_ok());
        
        let config = config.unwrap();
        assert_eq!(config.show_parent_dirs, 3);
        assert!(config.first_run);
        assert_eq!(config.sort_option, SortOption::Size);
        assert_eq!(config.max_ui_trees, 5);
    }

    #[test]
    fn test_sort_option_serialization() {
        let name = SortOption::Name;
        let size = SortOption::Size;
        let modified = SortOption::Modified;

        assert_eq!(serde_json::to_string(&name).unwrap(), "\"Name\"");
        assert_eq!(serde_json::to_string(&size).unwrap(), "\"Size\"");
        assert_eq!(serde_json::to_string(&modified).unwrap(), "\"Modified\"");
    }

    #[test]
    fn test_bookmarks_modification() {
        let mut config = Config::default();
        assert!(config.bookmarks.is_empty());

        config.bookmarks.push(PathBuf::from("/home/user/Documents"));
        assert_eq!(config.bookmarks.len(), 1);

        config.bookmarks.push(PathBuf::from("/home/user/Projects"));
        assert_eq!(config.bookmarks.len(), 2);
    }
}
