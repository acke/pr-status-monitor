#!/bin/bash

# PR Status Monitor - Installation Script
# This script installs both the CLI tool and menu bar app

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
CLI_NAME="my-prs"
APP_NAME="prstatus"
INSTALL_DIR="/usr/local/bin"
APPLICATIONS_DIR="/Applications"

echo -e "${BLUE}🚀 PR Status Monitor Installation${NC}"
echo "=================================="
echo

# Check if running on macOS
if [[ "$OSTYPE" != "darwin"* ]]; then
    echo -e "${RED}❌ This installer is for macOS only${NC}"
    exit 1
fi

# Check for required files
if [[ ! -f "bin/$CLI_NAME" ]]; then
    echo -e "${RED}❌ CLI binary not found at bin/$CLI_NAME${NC}"
    echo "This distribution package seems to be corrupted or incomplete."
    exit 1
fi

if [[ ! -d "app/$APP_NAME.app" ]]; then
    echo -e "${RED}❌ Menu bar app not found at app/$APP_NAME.app${NC}"
    echo "This distribution package seems to be corrupted or incomplete."
    exit 1
fi

echo -e "${YELLOW}📋 Installation Plan:${NC}"
echo "• CLI tool: $INSTALL_DIR/$CLI_NAME"
echo "• Menu bar app: $APPLICATIONS_DIR/$APP_NAME.app"
echo

# Ask for confirmation
read -p "Continue with installation? (y/N): " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo -e "${YELLOW}⚠️  Installation cancelled${NC}"
    exit 0
fi

echo -e "${BLUE}📦 Installing CLI tool...${NC}"

# Install CLI tool
if [[ -w "$INSTALL_DIR" ]]; then
    cp "bin/$CLI_NAME" "$INSTALL_DIR/"
    chmod +x "$INSTALL_DIR/$CLI_NAME"
else
    echo "Installing CLI tool requires sudo permissions..."
    sudo cp "bin/$CLI_NAME" "$INSTALL_DIR/"
    sudo chmod +x "$INSTALL_DIR/$CLI_NAME"
fi

echo -e "${GREEN}✅ CLI tool installed to $INSTALL_DIR/$CLI_NAME${NC}"

echo -e "${BLUE}📱 Installing menu bar app...${NC}"

# Remove existing app if it exists
if [[ -d "$APPLICATIONS_DIR/$APP_NAME.app" ]]; then
    echo "Removing existing app..."
    rm -rf "$APPLICATIONS_DIR/$APP_NAME.app"
fi

# Install menu bar app
cp -R "app/$APP_NAME.app" "$APPLICATIONS_DIR/"

echo -e "${GREEN}✅ Menu bar app installed to $APPLICATIONS_DIR/$APP_NAME.app${NC}"

echo
echo -e "${GREEN}🎉 Installation Complete!${NC}"
echo
echo -e "${YELLOW}📝 Next Steps:${NC}"
echo "1. Set up your GitHub token:"
echo "   ${CLI_NAME} config --wizard"
echo
echo "2. Test the CLI tool:"
echo "   ${CLI_NAME} --help"
echo "   ${CLI_NAME} my-prs"
echo
echo "3. Launch the menu bar app:"
echo "   open $APPLICATIONS_DIR/$APP_NAME.app"
echo
echo -e "${BLUE}💡 Pro Tips:${NC}"
echo "• The menu bar app will auto-detect your CLI tool location"
echo "• Configure refresh interval in the app's settings (⚙️ icon)"
echo "• Click on PRs in the menu to open them in your browser"
echo
echo -e "${GREEN}Happy PR tracking! 🚀${NC}"
