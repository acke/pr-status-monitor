#!/bin/bash

# PR Status Monitor - Distribution Package Creator
# This script creates a distributable package for colleagues

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
DIST_NAME="pr-status-monitor"
# Extract version from Cargo.toml
VERSION=$(grep '^version = ' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')
if [[ -z "$VERSION" ]]; then
    echo -e "${RED}❌ Could not extract version from Cargo.toml${NC}"
    exit 1
fi

# Always bump the PATCH version (third number) unless NO_BUMP=1 is set
if [[ -z "${NO_BUMP}" || "${NO_BUMP}" != "1" ]]; then
    IFS='.' read -r MAJOR MINOR PATCH <<< "${VERSION}"
    if [[ -z "${MAJOR}" || -z "${MINOR}" || -z "${PATCH}" ]]; then
        echo -e "${RED}❌ Invalid version format in Cargo.toml: ${VERSION}${NC}"
        exit 1
    fi
    NEW_PATCH=$((PATCH + 1))
    NEW_VERSION="${MAJOR}.${MINOR}.${NEW_PATCH}"
    # Update Cargo.toml in-place
    sed -i '' "s/^version = \"${VERSION}\"/version = \"${NEW_VERSION}\"/" Cargo.toml
    VERSION="${NEW_VERSION}"
fi
DIST_DIR="dist"
PACKAGE_NAME="${DIST_NAME}-${VERSION}"

echo -e "${BLUE}📦 Creating Distribution Package${NC}"
echo "================================="
echo -e "${BLUE}Version: ${VERSION}${NC}"
echo

# Clean and create distribution directory
if [[ -d "$DIST_DIR" ]]; then
    echo -e "${YELLOW}🧹 Cleaning existing distribution directory...${NC}"
    rm -rf "$DIST_DIR"
fi

mkdir -p "$DIST_DIR/$PACKAGE_NAME"

echo -e "${BLUE}📋 Checking prerequisites...${NC}"

# Always rebuild CLI to ensure latest code changes are included
echo -e "${YELLOW}🔧 Building CLI (version ${VERSION}) - always rebuild for latest changes...${NC}"
cargo build --release > /dev/null

# Ad-hoc code sign the CLI to avoid Gatekeeper kills on colleague machines
if command -v codesign >/dev/null 2>&1; then
    echo -e "${YELLOW}🔏 Code signing CLI (ad-hoc)...${NC}"
    codesign --force -s - --timestamp=none target/release/my-prs || {
        echo -e "${YELLOW}⚠️  Code signing failed; continuing unsigned${NC}"
    }
fi

# Always rebuild menu bar app to ensure latest code changes are included
echo -e "${YELLOW}🔧 Building menu bar app (always rebuild for latest changes)...${NC}"
cd prstatus
xcodebuild -project prstatus.xcodeproj -scheme prstatus -configuration Release -derivedDataPath build > /dev/null
cd ..

# Ad-hoc sign the built app bundle to avoid Gatekeeper kills on colleague machines
if command -v codesign >/dev/null 2>&1; then
    echo -e "${YELLOW}🔏 Code signing app (ad-hoc)...${NC}"
    codesign --force -s - --timestamp=none --deep prstatus/build/Build/Products/Release/prstatus.app || {
        echo -e "${YELLOW}⚠️  App code signing failed; continuing unsigned${NC}"
    }
fi

echo -e "${GREEN}✅ All prerequisites met${NC}"

echo -e "${BLUE}📁 Copying files to distribution package...${NC}"

# Copy CLI binary
mkdir -p "$DIST_DIR/$PACKAGE_NAME/bin"
cp "target/release/my-prs" "$DIST_DIR/$PACKAGE_NAME/bin/"

# Copy menu bar app
mkdir -p "$DIST_DIR/$PACKAGE_NAME/app"
cp -R "prstatus/build/Build/Products/Release/prstatus.app" "$DIST_DIR/$PACKAGE_NAME/app/"

# Copy installation script
cp "install.sh" "$DIST_DIR/$PACKAGE_NAME/"

# Copy documentation
cp "README.md" "$DIST_DIR/$PACKAGE_NAME/"

# Create a simple setup guide
cat > "$DIST_DIR/$PACKAGE_NAME/SETUP.md" << 'EOF'
# PR Status Monitor Setup Guide

Welcome to PR Status Monitor! This package includes both a CLI tool and a macOS menu bar app to track your GitHub PR status.

## Quick Start

1. **Run the installer:**
   ```bash
   ./install.sh
   ```

2. **Configure your GitHub token:**
   ```bash
   my-prs config --wizard
   ```
   
   You'll need a GitHub Personal Access Token with `repo` permissions.

3. **Test the CLI:**
   ```bash
   my-prs my-prs
   ```

4. **Launch the menu bar app:**
   ```bash
   open /Applications/prstatus.app
   ```

## What's Included

- **`bin/my-prs`** - CLI tool for terminal use
- **`app/prstatus.app`** - Menu bar app for continuous monitoring
- **`install.sh`** - Automated installation script
- **`README.md`** - Full documentation

## Features

### CLI Tool
- List your PRs across repositories
- Check PR status and CI/CD results
- Scan local repositories for GitHub remotes
- Monitor PR builds in real-time
- JSON output for automation

### Menu Bar App
- Real-time PR status in your menu bar
- Visual indicators (🔴 failing, 🟡 running, 🟢 passing)
- Click PRs to open in browser
- Configurable refresh intervals
- Auto-detects CLI tool location

## Configuration

The tool stores configuration in `~/.config/my-prs/config.toml`. You can:

- Set GitHub token, username, organization
- Configure default repository paths
- Set PR filters and limits
- Customize workflow display options

## Troubleshooting

### "CLI Tool Not Found" in Menu Bar App
1. Open the menu bar app
2. Click ⚙️ Settings
3. Click 🔍 Auto-detect CLI Path
4. Or manually set path to `/usr/local/bin/my-prs`

### "GitHub token is required" Error
Run the configuration wizard:
```bash
my-prs config --wizard
```

### Menu Bar App Not Appearing
1. Check System Preferences > Security & Privacy
2. Allow the app to run
3. Look for the icon in your menu bar (might be hidden in the "..." menu)

## Support

For issues or questions, check the README.md file or contact your colleague who shared this tool.

Happy PR tracking! 🚀
EOF

# Create version info
cat > "$DIST_DIR/$PACKAGE_NAME/VERSION" << EOF
PR Status Monitor v${VERSION}
Built: $(date)
Platform: macOS (ARM64/Intel)
Components:
- CLI Tool: my-prs
- Menu Bar App: prstatus.app
EOF

echo -e "${BLUE}📄 Creating package archive...${NC}"

# Create tarball
cd "$DIST_DIR"
tar -czf "${PACKAGE_NAME}.tar.gz" "$PACKAGE_NAME"
cd ..

# Create checksum
cd "$DIST_DIR"
shasum -a 256 "${PACKAGE_NAME}.tar.gz" > "${PACKAGE_NAME}.tar.gz.sha256"
cd ..

echo
echo -e "${GREEN}🎉 Distribution package created successfully!${NC}"
echo
echo -e "${YELLOW}📦 Package Details:${NC}"
echo "• Location: $DIST_DIR/${PACKAGE_NAME}.tar.gz"
echo "• Size: $(du -h "$DIST_DIR/${PACKAGE_NAME}.tar.gz" | cut -f1)"
echo "• Checksum: $DIST_DIR/${PACKAGE_NAME}.tar.gz.sha256"
echo
echo -e "${BLUE}📤 Sharing Instructions:${NC}"
echo "1. Send the .tar.gz file to your colleagues"
echo "2. They should extract it: tar -xzf ${PACKAGE_NAME}.tar.gz"
echo "3. Run the installer: cd ${PACKAGE_NAME} && ./install.sh"
echo
echo -e "${GREEN}Ready to share! 🚀${NC}"
