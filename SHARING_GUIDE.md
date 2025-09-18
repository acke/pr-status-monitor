# PR Status Monitor - Sharing Guide

## 🎯 Quick Distribution

Your colleagues can get up and running in 3 simple steps:

### 1. **Share the Package**
Send them the file: `dist/pr-status-monitor-1.0.0.tar.gz` (2.1MB)

### 2. **They Extract & Install**
```bash
tar -xzf pr-status-monitor-1.0.0.tar.gz
cd pr-status-monitor-1.0.0
./install.sh
```

### 3. **They Configure & Use**
```bash
# Set up GitHub token
my-prs config --wizard

# Test CLI
my-prs my-prs

# Launch menu bar app
open /Applications/prstatus.app
```

## 📋 What They Get

### ✅ **CLI Tool** (`my-prs`)
- **Location**: `/usr/local/bin/my-prs`
- **Features**: List PRs, check status, monitor builds, scan local repos
- **Usage**: `my-prs --help` for all commands

### ✅ **Menu Bar App** (`prstatus.app`)
- **Location**: `/Applications/prstatus.app`
- **Features**: Real-time PR monitoring, visual status indicators, click to open PRs
- **Auto-detects**: CLI tool location automatically

## 🔧 **Prerequisites for Colleagues**

### **Required**:
- **macOS** (ARM64 or Intel)
- **GitHub Personal Access Token** with `repo` permissions
- **Terminal access** for initial setup

### **Optional but Recommended**:
- **Local git repositories** in `~/repos/` (configurable)
- **Snyk organization access** (default filter, configurable)

## 🚀 **Key Features They'll Love**

### **CLI Tool**:
```bash
my-prs my-prs           # Show your PRs with workflow status
my-prs list             # List all your PRs
my-prs scan             # Scan organization repositories
my-prs local            # Scan local git repositories
my-prs monitor          # Real-time monitoring mode
my-prs status 123       # Check specific PR status
```

### **Menu Bar App**:
- **🔴 Red**: Failing builds (needs attention)
- **🟡 Yellow**: Running builds (in progress)
- **🟢 Green**: Passing builds (all good)
- **📭 Gray**: No PRs or loading

## 📞 **Support for Colleagues**

### **Common Issues & Solutions**:

1. **"GitHub token is required"**
   ```bash
   my-prs config --wizard
   ```

2. **"CLI Tool Not Found" in menu bar app**
   - Open app → ⚙️ Settings → 🔍 Auto-detect CLI Path

3. **Menu bar app not visible**
   - Check System Preferences → Security & Privacy
   - Look in menu bar overflow (⋯ menu)

4. **No PRs showing**
   - Check GitHub token permissions
   - Verify organization access (default: `snyk`)
   - Run `my-prs config --show` to check settings

### **Configuration Options**:
```bash
my-prs config --wizard    # Interactive setup
my-prs config --show      # View current settings
```

**Configurable**:
- GitHub token & username
- Organization filter (default: `snyk`)
- Local repository path (default: `~/repos`)
- PR state filter (open/closed/all)
- Refresh intervals

## 📤 **Distribution Files**

Your package includes:
```
pr-status-monitor-1.0.0/
├── bin/my-prs              # CLI executable
├── app/prstatus.app        # Menu bar app
├── install.sh              # Automated installer
├── README.md               # Full documentation
├── SETUP.md                # Quick setup guide
└── VERSION                 # Version info
```

## 🎉 **Success Metrics**

After installation, colleagues should be able to:
- ✅ See their PR status in terminal: `my-prs my-prs`
- ✅ Monitor builds in menu bar: Red/Yellow/Green indicators
- ✅ Click PRs to open in browser
- ✅ Get real-time notifications of build status changes

---

**Ready to share!** Just send them the `pr-status-monitor-1.0.0.tar.gz` file and this guide. 🚀
