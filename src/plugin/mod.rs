//! Plugin system for Senterm
//! 
//! Provides extensibility through Lua scripts and external tools.
//! 
//! # Plugin Types
//! - Lua scripts: Custom commands, file handlers, themes
//! - External tools: CLI tools that can be invoked
//! 
//! # Plugin Location
//! Plugins are loaded from `~/.config/senterm/plugins/`
//! 
//! # Plugin Structure
//! ```text
//! plugins/
//! ├── my_plugin/
//! │   ├── plugin.toml    # Plugin manifest
//! │   └── init.lua       # Main plugin script
//! └── simple_script.lua  # Single-file plugin
//! ```

#![allow(dead_code)]

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Plugin manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    /// Plugin name
    pub name: String,
    /// Plugin version
    pub version: String,
    /// Plugin description
    #[serde(default)]
    pub description: String,
    /// Plugin author
    #[serde(default)]
    pub author: String,
    /// Minimum Senterm version required
    #[serde(default)]
    pub min_version: Option<String>,
    /// Plugin entry point (default: init.lua)
    #[serde(default = "default_entry")]
    pub entry: String,
    /// Plugin commands
    #[serde(default)]
    pub commands: Vec<PluginCommand>,
    /// File type handlers
    #[serde(default)]
    pub handlers: Vec<FileHandler>,
    /// Plugin hooks
    #[serde(default)]
    pub hooks: Vec<PluginHook>,
}

fn default_entry() -> String {
    "init.lua".to_string()
}

/// Plugin command definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginCommand {
    /// Command name (e.g., "format")
    pub name: String,
    /// Command description
    #[serde(default)]
    pub description: String,
    /// Function to call in Lua
    pub function: String,
    /// Keybinding (optional)
    #[serde(default)]
    pub keybinding: Option<String>,
}

/// File type handler
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileHandler {
    /// File extensions to handle
    pub extensions: Vec<String>,
    /// Handler function in Lua
    pub function: String,
    /// Handler type: "viewer" or "opener"
    #[serde(default = "default_handler_type")]
    pub handler_type: String,
}

fn default_handler_type() -> String {
    "viewer".to_string()
}

/// Plugin hook
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginHook {
    /// Hook event name
    pub event: String,
    /// Function to call
    pub function: String,
}

/// Available hook events
pub enum HookEvent {
    /// Called when a file is selected
    FileSelected { path: PathBuf },
    /// Called when a directory is entered
    DirectoryEntered { path: PathBuf },
    /// Called when a file is opened
    FileOpened { path: PathBuf },
    /// Called when the app starts
    AppStarted,
    /// Called before the app exits
    AppExiting,
}

impl HookEvent {
    pub fn name(&self) -> &'static str {
        match self {
            HookEvent::FileSelected { .. } => "file_selected",
            HookEvent::DirectoryEntered { .. } => "directory_entered",
            HookEvent::FileOpened { .. } => "file_opened",
            HookEvent::AppStarted => "app_started",
            HookEvent::AppExiting => "app_exiting",
        }
    }
}

/// Loaded plugin information
#[derive(Debug, Clone)]
pub struct LoadedPlugin {
    /// Plugin manifest
    pub manifest: PluginManifest,
    /// Plugin directory
    pub path: PathBuf,
    /// Whether plugin is enabled
    pub enabled: bool,
}

/// Plugin manager
pub struct PluginManager {
    /// Loaded plugins
    plugins: HashMap<String, LoadedPlugin>,
    /// Plugin directory
    plugin_dir: PathBuf,
    /// Command registry
    commands: HashMap<String, (String, String)>, // (plugin_name, function)
    /// File handlers
    file_handlers: HashMap<String, (String, String)>, // extension -> (plugin_name, function)
}

impl PluginManager {
    /// Create a new plugin manager
    pub fn new() -> Result<Self> {
        let plugin_dir = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?
            .join("senterm")
            .join("plugins");
        
        std::fs::create_dir_all(&plugin_dir)?;
        
        Ok(Self {
            plugins: HashMap::new(),
            plugin_dir,
            commands: HashMap::new(),
            file_handlers: HashMap::new(),
        })
    }
    
    /// Load all plugins from the plugin directory
    pub fn load_all(&mut self) -> Result<()> {
        let entries = std::fs::read_dir(&self.plugin_dir)?;
        
        for entry in entries.flatten() {
            let path = entry.path();
            
            if path.is_dir() {
                // Directory plugin with manifest
                let manifest_path = path.join("plugin.toml");
                if manifest_path.exists() {
                    if let Err(e) = self.load_plugin(&path) {
                        tracing::warn!("Failed to load plugin from {:?}: {}", path, e);
                    }
                }
            } else if path.extension().map_or(false, |e| e == "lua") {
                // Single-file Lua plugin
                if let Err(e) = self.load_simple_plugin(&path) {
                    tracing::warn!("Failed to load plugin {:?}: {}", path, e);
                }
            }
        }
        
        tracing::info!("Loaded {} plugins", self.plugins.len());
        Ok(())
    }
    
    /// Load a plugin from a directory
    fn load_plugin(&mut self, path: &Path) -> Result<()> {
        let manifest_path = path.join("plugin.toml");
        let manifest_content = std::fs::read_to_string(&manifest_path)?;
        let manifest: PluginManifest = toml::from_str(&manifest_content)?;
        
        // Check version compatibility
        if let Some(min_version) = &manifest.min_version {
            let current = env!("CARGO_PKG_VERSION");
            if !version_compatible(current, min_version) {
                return Err(anyhow::anyhow!(
                    "Plugin {} requires Senterm version {} or higher (current: {})",
                    manifest.name, min_version, current
                ));
            }
        }
        
        // Verify entry point exists
        let entry_path = path.join(&manifest.entry);
        if !entry_path.exists() {
            return Err(anyhow::anyhow!(
                "Plugin entry point not found: {:?}",
                entry_path
            ));
        }
        
        // Register commands
        for cmd in &manifest.commands {
            self.commands.insert(
                cmd.name.clone(),
                (manifest.name.clone(), cmd.function.clone()),
            );
        }
        
        // Register file handlers
        for handler in &manifest.handlers {
            for ext in &handler.extensions {
                self.file_handlers.insert(
                    ext.clone(),
                    (manifest.name.clone(), handler.function.clone()),
                );
            }
        }
        
        let plugin = LoadedPlugin {
            manifest: manifest.clone(),
            path: path.to_path_buf(),
            enabled: true,
        };
        
        self.plugins.insert(manifest.name.clone(), plugin);
        tracing::info!("Loaded plugin: {}", manifest.name);
        
        Ok(())
    }
    
    /// Load a simple single-file plugin
    fn load_simple_plugin(&mut self, path: &Path) -> Result<()> {
        let name = path.file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| anyhow::anyhow!("Invalid plugin filename"))?
            .to_string();
        
        // Create a minimal manifest for single-file plugins
        let manifest = PluginManifest {
            name: name.clone(),
            version: "1.0.0".to_string(),
            description: format!("Single-file plugin: {}", path.display()),
            author: String::new(),
            min_version: None,
            entry: path.file_name().unwrap().to_string_lossy().to_string(),
            commands: Vec::new(),
            handlers: Vec::new(),
            hooks: Vec::new(),
        };
        
        let plugin = LoadedPlugin {
            manifest,
            path: path.parent().unwrap_or(Path::new(".")).to_path_buf(),
            enabled: true,
        };
        
        self.plugins.insert(name.clone(), plugin);
        tracing::info!("Loaded simple plugin: {}", name);
        
        Ok(())
    }
    
    /// Get list of loaded plugins
    pub fn list(&self) -> Vec<&LoadedPlugin> {
        self.plugins.values().collect()
    }
    
    /// Get a plugin by name
    pub fn get(&self, name: &str) -> Option<&LoadedPlugin> {
        self.plugins.get(name)
    }
    
    /// Enable or disable a plugin
    pub fn set_enabled(&mut self, name: &str, enabled: bool) -> Result<()> {
        let plugin = self.plugins.get_mut(name)
            .ok_or_else(|| anyhow::anyhow!("Plugin not found: {}", name))?;
        
        plugin.enabled = enabled;
        Ok(())
    }
    
    /// Get available commands
    pub fn get_commands(&self) -> &HashMap<String, (String, String)> {
        &self.commands
    }
    
    /// Check if a file extension has a handler
    pub fn has_handler(&self, extension: &str) -> bool {
        self.file_handlers.contains_key(extension)
    }
    
    /// Get handler for file extension
    pub fn get_handler(&self, extension: &str) -> Option<&(String, String)> {
        self.file_handlers.get(extension)
    }
    
    /// Get plugin directory
    pub fn plugin_dir(&self) -> &Path {
        &self.plugin_dir
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| Self {
            plugins: HashMap::new(),
            plugin_dir: PathBuf::new(),
            commands: HashMap::new(),
            file_handlers: HashMap::new(),
        })
    }
}

/// Simple version compatibility check
fn version_compatible(current: &str, required: &str) -> bool {
    let current_parts: Vec<u32> = current
        .split('.')
        .filter_map(|s| s.parse().ok())
        .collect();
    
    let required_parts: Vec<u32> = required
        .split('.')
        .filter_map(|s| s.parse().ok())
        .collect();
    
    for (c, r) in current_parts.iter().zip(required_parts.iter()) {
        if c > r {
            return true;
        }
        if c < r {
            return false;
        }
    }
    
    current_parts.len() >= required_parts.len()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_version_compatible() {
        assert!(version_compatible("1.0.0", "1.0.0"));
        assert!(version_compatible("1.1.0", "1.0.0"));
        assert!(version_compatible("2.0.0", "1.9.9"));
        assert!(!version_compatible("0.9.0", "1.0.0"));
        assert!(!version_compatible("1.0.0", "1.0.1"));
    }
}

