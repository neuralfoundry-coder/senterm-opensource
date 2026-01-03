#!/usr/bin/env bash

# ============================================================================
# Senterm Linux Installer
# Installs senterm as 'x' command for easy access from any terminal
# ============================================================================

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m'

BINARY_NAME="senterm"
COMMAND_NAME="x"
INSTALL_DIR="/usr/local/bin"

echo ""
echo -e "${CYAN}${BOLD}╔══════════════════════════════════════════════════════════════╗${NC}"
echo -e "${CYAN}${BOLD}║              Senterm Linux Installer                         ║${NC}"
echo -e "${CYAN}${BOLD}║              Installing as '${COMMAND_NAME}' command                        ║${NC}"
echo -e "${CYAN}${BOLD}╚══════════════════════════════════════════════════════════════╝${NC}"
echo ""

# Get script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Find the binary
if [[ -f "${SCRIPT_DIR}/${BINARY_NAME}" ]]; then
    SOURCE_BINARY="${SCRIPT_DIR}/${BINARY_NAME}"
elif [[ -f "${SCRIPT_DIR}/../${BINARY_NAME}" ]]; then
    SOURCE_BINARY="${SCRIPT_DIR}/../${BINARY_NAME}"
elif [[ -f "./${BINARY_NAME}" ]]; then
    SOURCE_BINARY="./${BINARY_NAME}"
else
    echo -e "${RED}✗ Error: ${BINARY_NAME} binary not found${NC}"
    echo "Please run this script from the directory containing the binary."
    exit 1
fi

echo -e "${GREEN}✓${NC} Found binary: ${SOURCE_BINARY}"

# Check if /usr/local/bin exists
if [[ ! -d "${INSTALL_DIR}" ]]; then
    echo -e "${YELLOW}→${NC} Creating ${INSTALL_DIR}..."
    sudo mkdir -p "${INSTALL_DIR}"
fi

# Install the binary
echo ""
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
    echo -e "${GREEN}✓${NC} '${COMMAND_NAME}' command is now available globally"
    echo ""
    echo -e "  Usage:"
    echo -e "    ${CYAN}${COMMAND_NAME}${NC}              Start file manager in current directory"
    echo -e "    ${CYAN}${COMMAND_NAME} <path>${NC}       Start file manager in specified path"
    echo ""
else
    echo -e "${YELLOW}⚠${NC} Installation complete, but '${COMMAND_NAME}' not found in PATH"
    echo ""
    echo "Add the following to your shell profile (~/.bashrc or ~/.zshrc):"
    echo -e "  ${CYAN}export PATH=\"\$PATH:${INSTALL_DIR}\"${NC}"
    echo ""
    echo "Then restart your terminal or run:"
    echo -e "  ${CYAN}source ~/.bashrc${NC}"
    echo ""
fi

