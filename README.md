# Senterm

> **A modern terminal-based file manager with Miller columns and integrated shell.**

<p align="center">
  <img src="images/senterm.png" alt="Senterm Screenshot" width="800">
</p>

A next-generation terminal-based file manager built with Rust for performance and safety.

![Rust](https://img.shields.io/badge/Rust-1.70+-orange.svg)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
![Platform](https://img.shields.io/badge/platform-Linux%20%7C%20macOS-lightgrey.svg)

---

## 🌟 Highlights

- **📁 Miller Columns** - Intuitive multi-column directory navigation
- **🖥️ Integrated Shell** - Full PTY shell panel alongside file manager
- **🖥️ Terminal Native** - Lightweight, keyboard-driven, works over SSH
- **🔄 Real-time Sync** - File changes reflect instantly across all views
- **🎨 Beautiful Themes** - 10 built-in color schemes (Tokyo Night, Dracula, Nord, and more)
- **🖼️ Image Preview** - View images directly in terminal (ASCII/Unicode blocks)
- **📊 System Monitor** - Real-time CPU, memory, and process viewer
- **📄 File Viewer** - Built-in viewer for text, markdown, DOCX, XLSX, HWP

---

## 🚀 Quick Installation

### One-Line Install (Auto-detect OS)

```bash
curl -sSfL https://raw.githubusercontent.com/neuralfoundry-coder/senterm-opensource/main/binaries/install-universal.sh | bash
```

### macOS (Universal: Intel + Apple Silicon)

```bash
curl -sSfL https://raw.githubusercontent.com/neuralfoundry-coder/senterm-opensource/main/binaries/install.sh | bash
```

### Linux (x86_64)

```bash
curl -sSfL https://raw.githubusercontent.com/neuralfoundry-coder/senterm-opensource/main/binaries/install-linux.sh | bash
```

### With specific version

```bash
# Universal (auto-detect)
curl -sSfL https://raw.githubusercontent.com/neuralfoundry-coder/senterm-opensource/main/binaries/install-universal.sh | bash -s -- --version 20251220

# macOS only
curl -sSfL https://raw.githubusercontent.com/neuralfoundry-coder/senterm-opensource/main/binaries/install.sh | bash -s -- --version 20251220

# Linux only
curl -sSfL https://raw.githubusercontent.com/neuralfoundry-coder/senterm-opensource/main/binaries/install-linux.sh | bash -s -- --version 20251220
```

### Usage

```bash
x              # Start in current directory
x <path>       # Start in specified path
```

### Uninstall

```bash
sudo rm /usr/local/bin/x
```

---

## 🔧 Build from Source

```bash
git clone https://github.com/neuralfoundry-coder/senterm-opensource.git
cd senterm-opensource
cargo build --release
./target/release/senterm
```

---

## ✨ Features

### Core File Management
| Feature | Description |
|---------|-------------|
| **Miller Columns** | Multi-column interface showing directory hierarchy |
| **File Operations** | Create, rename, delete, copy, cut, paste (recursive) |
| **File Viewer** | Built-in viewer for text, markdown, DOCX, XLSX, HWP |
| **Image Preview** | View PNG, JPEG, GIF images directly in terminal |
| **Search** | Recursive file search with real-time results |
| **Bookmarks** | Quick access to favorite directories |
| **Sorting** | By name, size, or modification date |
| **Multi-Pane** | Up to 3 simultaneous file panels (F3 to add) |

### 🖥️ Console Panel

Split-view console with full PTY shell:

- **F5** - Toggle console panel
- **Tab** - Cycle focus (files ↔ console)
- **Esc** - Return focus to file manager

### 📊 System Monitor
- Real-time CPU, memory, disk usage
- Process list with sorting
- Interactive process viewer (F9)

### 🎨 Themes (10 Built-in)

| Theme | Style |
|-------|-------|
| **Elegant Dark** | Default sleek dark mode |
| **Elegant Light** | Clean light theme |
| **Monokai** | Classic code editor colors |
| **Dracula** | Popular dark purple theme |
| **Solarized Dark** | Precision colors for dark |
| **Solarized Light** | Precision colors for light |
| **Nord** | Arctic, bluish color palette |
| **Gruvbox Dark** | Retro groove colors |
| **One Dark** | Atom editor inspired |
| **Tokyo Night** | A clean, dark theme inspired by Tokyo city lights |

---

## ⌨️ Key Bindings

### Global
| Key | Action |
|-----|--------|
| `q` | Quit |
| `Esc` | Exit current mode |
| `[` / `]` | Switch modes |
| `F5` | Toggle console panel |
| `F8` | Settings |
| `F9` | Process viewer |
| `F12` / `` ` `` | Shell popup |

### File Manager
| Key | Action |
|-----|--------|
| `↑/↓/←/→` | Navigate |
| `Enter` | Open/Enter |
| `v` | View file |
| `/` | Search |
| `F2` | Rename |
| `F3` | Add pane |
| `F4` | Remove pane |
| `F7` | New folder |
| `F8` | New file |
| `c/x/p` | Copy/Cut/Paste |
| `b/B` | Add/View bookmarks |
| `s` | Cycle sort |

### Console Panel
| Key | Action |
|-----|--------|
| `Tab` | Switch focus to/from console |
| `Esc` | Return focus to file manager |
| All keys | Passed to shell (bash/zsh) |

---

## ⚙️ Configuration

### Config File Location
```
~/.config/senterm/config.toml
```

### Example Configuration
```toml
[theme]
name = "Elegant Dark"

first_run = false
show_parent_dirs = 5
max_ui_trees = 3
sort_option = "Name"

bookmarks = [
    "/home/user/Documents",
    "/home/user/Projects"
]
```

---

## 🏗️ Architecture

```
senterm/
├── src/
│   ├── main.rs              # Entry point
│   ├── app.rs               # Application state
│   ├── ui.rs                # UI rendering
│   ├── config.rs            # Configuration
│   ├── navigation.rs        # Miller Columns logic
│   ├── fs/
│   │   ├── mod.rs           # File system operations
│   │   └── watcher.rs       # Real-time file monitoring
│   ├── events/              # Event handlers
│   ├── viewer/
│   │   ├── mod.rs           # File viewer
│   │   ├── image.rs         # Terminal image preview
│   │   └── highlight.rs     # Syntax highlighting
│   ├── system/              # System monitor
│   └── process/             # Process viewer
├── tests/
│   └── integration_tests.rs # Test suite
└── Cargo.toml
```

---

## 📦 Dependencies

| Crate | Purpose |
|-------|---------|
| `ratatui` | Terminal UI framework |
| `crossterm` | Cross-platform terminal |
| `tokio` | Async runtime |
| `notify` | File system watching |
| `portable-pty` | PTY for shell |
| `syntect` | Syntax highlighting |
| `image` | Image processing for preview |
| `sysinfo` | System information |

---

## 🗺️ Roadmap

### Completed ✅
- [x] Miller Columns file manager
- [x] Integrated shell panel
- [x] Real-time file watching
- [x] Image preview in terminal (ASCII/Unicode)
- [x] Multi-pane support (up to 3 panes)
- [x] 10 built-in themes
- [x] Process viewer
- [x] System monitor
- [x] File viewer (text, markdown, office documents)

### Planned 🚧
- [ ] Sixel/Kitty graphics protocol
- [ ] Tab completion in file manager
- [ ] More file format support

---

## 🤝 Contributing

Contributions welcome! Please read our contributing guidelines.

```bash
# Run all tests
cargo test

# Run with logging
RUST_LOG=debug cargo run

# Build release version
cargo build --release

# Check code quality
cargo clippy
```

---

## 📄 License

This project is licensed under the [MIT License](LICENSE).

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

---

## 🙏 Acknowledgments

- Built with [Ratatui](https://ratatui.rs/) and the Rust community
