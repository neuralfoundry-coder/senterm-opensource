# Senterm Open Source

Binary distribution for [Senterm Open Source](https://github.com/neuralfoundry-coder/senterm-opensource).

A modern terminal file manager with Miller Columns navigation.

## One-Line Installation

### Universal (Auto-detect OS)

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
curl -sSfL https://raw.githubusercontent.com/neuralfoundry-coder/senterm-opensource/main/binaries/install-universal.sh | bash -s -- --version v0.1.0

# macOS
curl -sSfL https://raw.githubusercontent.com/neuralfoundry-coder/senterm-opensource/main/binaries/install.sh | bash -s -- --version v0.1.0

# Linux
curl -sSfL https://raw.githubusercontent.com/neuralfoundry-coder/senterm-opensource/main/binaries/install-linux.sh | bash -s -- --version v0.1.0
```

## Usage

After installation, run:
```bash
x              # Start in current directory
x <path>       # Start in specified path
```

## Features

- Miller Columns file navigation
- Integrated shell panel
- Multi-pane support (up to 3 panes)
- Syntax highlighting
- Image preview in terminal
- 10 built-in themes
- Process viewer
- Real-time file watching

## Uninstall

```bash
sudo rm /usr/local/bin/x
```

## Supported Platforms

- **macOS**: Universal binary (Intel x86_64 + Apple Silicon arm64)
- **Linux**: x86_64

## License

MIT License
