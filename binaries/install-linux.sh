#!/usr/bin/env bash

# ============================================================================
# Senterm Open Source - One-Line Installer for Linux
# 
# Usage:
#   curl -sSfL https://raw.githubusercontent.com/neuralfoundry-coder/senterm-opensource/main/binaries/install-linux.sh | bash
#
# Or with specific version:
#   curl -sSfL https://raw.githubusercontent.com/neuralfoundry-coder/senterm-opensource/main/binaries/install-linux.sh | bash -s -- --version v0.1.0
#
# ============================================================================

set -e

# Configuration - Open Source repository
REPO_OWNER="neuralfoundry-coder"
REPO_NAME="senterm-opensource"
BINARY_NAME="senterm"
COMMAND_NAME="x"
INSTALL_DIR="/usr/local/bin"

# GitHub Releases URL
RELEASES_URL="https://github.com/${REPO_OWNER}/${REPO_NAME}/releases"
API_URL="https://api.github.com/repos/${REPO_OWNER}/${REPO_NAME}/releases"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m'

# Parse arguments
VERSION=""
while [[ $# -gt 0 ]]; do
    case $1 in
        --version|-v)
            VERSION="$2"
            shift 2
            ;;
        *)
            shift
            ;;
    esac
done

echo ""
echo -e "${CYAN}${BOLD}╔══════════════════════════════════════════════════════════════╗${NC}"
echo -e "${CYAN}${BOLD}║     Senterm Open Source - Installer for Linux                ║${NC}"
echo -e "${CYAN}${BOLD}╚══════════════════════════════════════════════════════════════╝${NC}"
echo ""

# Detect architecture
ARCH=$(uname -m)
OS=$(uname -s)

if [[ "$OS" != "Linux" ]]; then
    echo -e "${RED}✗ This installer is for Linux only${NC}"
    echo "For macOS, use: curl -sSfL https://raw.githubusercontent.com/${REPO_OWNER}/${REPO_NAME}/main/binaries/install.sh | bash"
    exit 1
fi

if [[ "$ARCH" != "x86_64" ]]; then
    echo -e "${RED}✗ Unsupported architecture: $ARCH${NC}"
    echo "Currently only x86_64 is supported"
    exit 1
fi

echo -e "${GREEN}✓${NC} Detected: Linux ($ARCH)"

# Get latest version from GitHub Releases if not specified
if [[ -z "$VERSION" ]]; then
    echo -e "${BLUE}→${NC} Fetching latest release..."
    
    # Get latest release tag from GitHub API
    VERSION=$(curl -sSf "${API_URL}/latest" 2>/dev/null | grep '"tag_name"' | sed -E 's/.*"tag_name": *"([^"]+)".*/\1/')
    
    if [[ -z "$VERSION" ]]; then
        echo -e "${RED}✗ Failed to get latest version${NC}"
        echo ""
        echo "This could mean:"
        echo "  - No releases have been published yet"
        echo "  - GitHub API rate limit reached"
        echo ""
        echo "Try specifying version manually:"
        echo "  curl ... | bash -s -- --version v0.1.0"
        echo ""
        echo "Check available releases at:"
        echo "  ${RELEASES_URL}"
        exit 1
    fi
fi

echo -e "${GREEN}✓${NC} Version: ${VERSION}"

# Construct download URL from GitHub Releases
ASSET_NAME="${BINARY_NAME}-linux-x86_64.tar.gz"
DOWNLOAD_URL="${RELEASES_URL}/download/${VERSION}/${ASSET_NAME}"

echo -e "${BLUE}→${NC} Downloading from: ${DOWNLOAD_URL}"

# Create temp directory
TEMP_DIR=$(mktemp -d)
trap "rm -rf $TEMP_DIR" EXIT

cd "$TEMP_DIR"

# Download
if ! curl -sSfL -o "${ASSET_NAME}" "${DOWNLOAD_URL}"; then
    echo -e "${RED}✗ Download failed${NC}"
    echo ""
    echo "Possible reasons:"
    echo "  - Version ${VERSION} may not exist"
    echo "  - Release assets may not be uploaded yet"
    echo ""
    echo "Check available releases at:"
    echo "  ${RELEASES_URL}"
    exit 1
fi

echo -e "${GREEN}✓${NC} Download complete"

# Extract
echo -e "${BLUE}→${NC} Extracting..."
tar -xzf "${ASSET_NAME}"

# Find the binary (check multiple locations)
if [[ -f "${BINARY_NAME}" ]]; then
    SOURCE_BINARY="${BINARY_NAME}"
elif [[ -f "senterm-linux-x86_64/${BINARY_NAME}" ]]; then
    SOURCE_BINARY="senterm-linux-x86_64/${BINARY_NAME}"
elif [[ -f "release/${BINARY_NAME}" ]]; then
    SOURCE_BINARY="release/${BINARY_NAME}"
else
    # Try to find it anywhere
    FOUND_BINARY=$(find . -name "${BINARY_NAME}" -type f 2>/dev/null | head -1)
    if [[ -n "$FOUND_BINARY" ]]; then
        SOURCE_BINARY="$FOUND_BINARY"
    else
        echo -e "${RED}✗ Binary not found in archive${NC}"
        echo "Extracted files:"
        find . -type f
        exit 1
    fi
fi

echo -e "${GREEN}✓${NC} Extracted successfully"

# Verify binary
if file "${SOURCE_BINARY}" | grep -q "ELF"; then
    echo -e "${GREEN}✓${NC} Binary verified (ELF executable)"
else
    echo -e "${RED}✗ Invalid binary format${NC}"
    exit 1
fi

# Check install directory
if [[ ! -d "${INSTALL_DIR}" ]]; then
    echo -e "${BLUE}→${NC} Creating ${INSTALL_DIR}..."
    sudo mkdir -p "${INSTALL_DIR}"
fi

# Install
echo -e "${BLUE}→${NC} Installing to ${INSTALL_DIR}/${COMMAND_NAME}..."

if [[ -w "${INSTALL_DIR}" ]]; then
    cp "${SOURCE_BINARY}" "${INSTALL_DIR}/${COMMAND_NAME}"
    chmod +x "${INSTALL_DIR}/${COMMAND_NAME}"
else
    echo -e "${YELLOW}→${NC} Administrator privileges required..."
    sudo cp "${SOURCE_BINARY}" "${INSTALL_DIR}/${COMMAND_NAME}"
    sudo chmod +x "${INSTALL_DIR}/${COMMAND_NAME}"
fi

# Verify installation
echo ""
if command -v "${COMMAND_NAME}" &> /dev/null; then
    echo -e "${GREEN}${BOLD}╔══════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${GREEN}${BOLD}║              Installation Complete!                          ║${NC}"
    echo -e "${GREEN}${BOLD}╚══════════════════════════════════════════════════════════════╝${NC}"
    echo ""
    echo -e "${GREEN}✓${NC} '${COMMAND_NAME}' command is now available"
    echo ""
    echo -e "  ${BOLD}Usage:${NC}"
    echo -e "    ${CYAN}${COMMAND_NAME}${NC}              Start file manager in current directory"
    echo -e "    ${CYAN}${COMMAND_NAME} <path>${NC}       Start file manager in specified path"
    echo ""
    echo -e "  ${BOLD}Features:${NC}"
    echo -e "    - Miller Columns file navigation"
    echo -e "    - Integrated shell panel"
    echo -e "    - Syntax highlighting & image preview"
    echo ""
    echo -e "  ${BOLD}Uninstall:${NC}"
    echo -e "    ${CYAN}sudo rm ${INSTALL_DIR}/${COMMAND_NAME}${NC}"
    echo ""
else
    echo -e "${YELLOW}⚠${NC} Installation complete, but '${COMMAND_NAME}' not found in PATH"
    echo ""
    echo "Add the following to your shell profile (~/.bashrc or ~/.zshrc):"
    echo -e "  ${CYAN}export PATH=\"\$PATH:${INSTALL_DIR}\"${NC}"
    echo ""
    echo "Then restart your terminal or run:"
    echo -e "  ${CYAN}source ~/.bashrc${NC}"
fi
