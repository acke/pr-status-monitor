//
//  prstatusApp.swift
//  prstatus
//
//  Created by Knut Funkel on 2025-09-09.
//

import Cocoa
import SwiftUI
import Foundation

// MARK: - Data Models
struct PRStatus: Codable {
    let repository: String
    let number: Int
    let title: String
    let author: String
    let status: String
    let url: String
    let updated_at: String
    let is_draft: Bool
}

struct StatusSummary: Codable {
    let total_prs: Int
    let failing_prs: [PRStatus]
    let running_prs: [PRStatus]
    let passing_prs: [PRStatus]
    let review_prs: [PRStatus]
    let overall_status: String
    let last_updated: String
}

// MARK: - Settings View
struct SettingsView: View {
    @State private var cliToolPath: String = ""
    @State private var refreshInterval: String = "120"
    @State private var showingPathPicker = false
    @State private var showingAutoDetectAlert = false
    @State private var autoDetectMessage = ""
    @State private var detectedPaths: [String] = []
    
    let onSave: (String, Int) -> Void
    let onCancel: () -> Void
    
    var body: some View {
        VStack(spacing: 20) {
            // Header
            HStack {
                Image(systemName: "gear")
                    .font(.title2)
                    .foregroundColor(.blue)
                Text("PR Status Monitor Settings")
                    .font(.title2)
                    .fontWeight(.semibold)
                Spacer()
            }
            .padding(.top, 20)
            
            Divider()
            
            // CLI Tool Path Section
            VStack(alignment: .leading, spacing: 12) {
                Text("CLI Tool Path")
                    .font(.headline)
                    .foregroundColor(.primary)
                
                HStack {
                    TextField("Path to my-prs CLI tool", text: $cliToolPath)
                        .textFieldStyle(RoundedBorderTextFieldStyle())
                        .font(.system(.body, design: .monospaced))
                    
                    Button("Browse...") {
                        showingPathPicker = true
                    }
                    .buttonStyle(.bordered)
                }
                
                Button("🔍 Auto-detect CLI Path") {
                    autoDetectCliPath()
                }
                .buttonStyle(.bordered)
                .foregroundColor(.blue)
            }
            
            // Refresh Interval Section
            VStack(alignment: .leading, spacing: 12) {
                Text("Refresh Interval")
                    .font(.headline)
                    .foregroundColor(.primary)
                
                HStack {
                    TextField("Seconds", text: $refreshInterval)
                        .textFieldStyle(RoundedBorderTextFieldStyle())
                        .frame(width: 100)
                    
                    Text("seconds")
                        .foregroundColor(.secondary)
                    
                    Spacer()
                }
                
                Text("How often to check for PR updates (minimum: 30 seconds)")
                    .font(.caption)
                    .foregroundColor(.secondary)
            }
            
            Spacer()
            
            // Action Buttons
            HStack {
                Button("Cancel") {
                    onCancel()
                }
                .buttonStyle(.bordered)
                
                Spacer()
                
                Button("Save") {
                    let interval = Int(refreshInterval) ?? 120
                    onSave(cliToolPath, max(30, interval))
                }
                .buttonStyle(.borderedProminent)
                .disabled(cliToolPath.isEmpty)
            }
            .padding(.bottom, 20)
        }
        .padding(24)
        .frame(width: 500, height: 400)
        .background(Color(NSColor.windowBackgroundColor))
        .onAppear {
            loadCurrentSettings()
        }
        .fileImporter(
            isPresented: $showingPathPicker,
            allowedContentTypes: [.executable],
            allowsMultipleSelection: false
        ) { result in
            switch result {
            case .success(let urls):
                if let url = urls.first {
                    cliToolPath = url.path
                }
            case .failure(let error):
                print("File picker error: \(error)")
            }
        }
        .alert("Auto-detect Results", isPresented: $showingAutoDetectAlert) {
            if detectedPaths.count == 1 {
                Button("Use This Path") {
                    cliToolPath = detectedPaths[0]
                }
                Button("Cancel", role: .cancel) { }
            } else if detectedPaths.count > 1 {
                ForEach(Array(detectedPaths.enumerated()), id: \.offset) { index, path in
                    Button("\(index + 1). \(path)") {
                        cliToolPath = path
                    }
                }
                Button("Cancel", role: .cancel) { }
            } else {
                Button("OK") { }
            }
        } message: {
            Text(autoDetectMessage)
        }
    }
    
    private func loadCurrentSettings() {
        cliToolPath = UserDefaults.standard.string(forKey: "CLIToolPath") ?? ""
        let interval = UserDefaults.standard.integer(forKey: "RefreshInterval")
        refreshInterval = interval > 0 ? String(interval) : "120"
    }
    
    private func autoDetectCliPath() {
        detectedPaths = detectCliToolPaths()
        
        if detectedPaths.isEmpty {
            autoDetectMessage = "Could not find my-prs CLI tool in common locations. Please specify the path manually."
        } else if detectedPaths.count == 1 {
            autoDetectMessage = "Found CLI tool at: \(detectedPaths[0])"
        } else {
            autoDetectMessage = "Found multiple CLI tools. Choose which one to use:"
        }
        
        showingAutoDetectAlert = true
    }
    
    private func detectCliToolPaths() -> [String] {
        var paths: [String] = []
        
        let searchPaths = [
            // Common development locations
            "/Users/\(NSUserName())/repos/prtracker/target/release/my-prs",
            "/Users/\(NSUserName())/repos/prtracker/target/debug/my-prs",
            "/Users/\(NSUserName())/Developer/prtracker/target/release/my-prs",
            "/Users/\(NSUserName())/Code/prtracker/target/release/my-prs",
            "/Users/\(NSUserName())/Projects/prtracker/target/release/my-prs",
            
            // System-wide installations
            "/usr/local/bin/my-prs",
            "/opt/homebrew/bin/my-prs",
            "/usr/bin/my-prs",
            
            // Cargo installation
            "/Users/\(NSUserName())/.cargo/bin/my-prs"
        ]
        
        for path in searchPaths {
            if FileManager.default.fileExists(atPath: path) {
                if FileManager.default.isExecutableFile(atPath: path) {
                    paths.append(path)
                }
            }
        }
        
        // Also search in PATH
        if let pathEnv = ProcessInfo.processInfo.environment["PATH"] {
            let pathDirs = pathEnv.components(separatedBy: ":")
            for dir in pathDirs {
                let fullPath = "\(dir)/my-prs"
                if FileManager.default.fileExists(atPath: fullPath) && 
                   FileManager.default.isExecutableFile(atPath: fullPath) &&
                   !paths.contains(fullPath) {
                    paths.append(fullPath)
                }
            }
        }
        
        return paths
    }
}

// MARK: - Settings Window Controller
class SettingsWindowController: NSWindowController {
    private var settingsView: SettingsView!
    
    init() {
        let window = NSWindow(
            contentRect: NSRect(x: 0, y: 0, width: 500, height: 400),
            styleMask: [.titled, .closable, .miniaturizable],
            backing: .buffered,
            defer: false
        )
        
        super.init(window: window)
        
        window.title = "PR Status Monitor Settings"
        window.center()
        window.setFrameAutosaveName("SettingsWindow")
        window.isReleasedWhenClosed = false
        
        setupSettingsView()
    }
    
    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }
    
    private func setupSettingsView() {
        settingsView = SettingsView(
            onSave: { [weak self] path, interval in
                self?.saveSettings(path: path, interval: interval)
            },
            onCancel: { [weak self] in
                self?.close()
            }
        )
        
        let hostingView = NSHostingView(rootView: settingsView)
        hostingView.translatesAutoresizingMaskIntoConstraints = false
        
        window?.contentView = hostingView
        
        NSLayoutConstraint.activate([
            hostingView.topAnchor.constraint(equalTo: window!.contentView!.topAnchor),
            hostingView.leadingAnchor.constraint(equalTo: window!.contentView!.leadingAnchor),
            hostingView.trailingAnchor.constraint(equalTo: window!.contentView!.trailingAnchor),
            hostingView.bottomAnchor.constraint(equalTo: window!.contentView!.bottomAnchor)
        ])
    }
    
    private func saveSettings(path: String, interval: Int) {
        if !path.isEmpty {
            UserDefaults.standard.set(path, forKey: "CLIToolPath")
        }
        UserDefaults.standard.set(interval, forKey: "RefreshInterval")
        
        // Notify the main app to restart timer
        NotificationCenter.default.post(name: NSNotification.Name("SettingsUpdated"), object: nil)
        
        close()
    }
    
    func showWindow() {
        window?.makeKeyAndOrderFront(nil)
        NSApp.activate(ignoringOtherApps: true)
    }
}

// MARK: - Menu Bar App
class PRStatusMenuBarApp: NSObject, NSApplicationDelegate {
    private var statusItem: NSStatusItem!
    private var menu: NSMenu!
    private var timer: Timer?
    private var currentStatus: StatusSummary?
    private var settingsWindowController: SettingsWindowController?
    private var lastError: String?
    
    // Configurable CLI tool path
    private var cliToolPath: String {
        return getCliToolPath()
    }
    
    func applicationDidFinishLaunching(_ notification: Notification) {
        print("🚀 PR Status Menu Bar App starting...")
        
        // Hide dock icon (menu bar only app)
        NSApp.setActivationPolicy(.accessory)
        
        // Listen for settings updates
        NotificationCenter.default.addObserver(
            self,
            selector: #selector(settingsUpdated),
            name: NSNotification.Name("SettingsUpdated"),
            object: nil
        )
        
        setupMenuBar()
        startPeriodicUpdates()
        updateStatus() // Initial update
        
        print("✅ PR Status Menu Bar App initialized")
    }
    
    private func setupMenuBar() {
        statusItem = NSStatusBar.system.statusItem(withLength: NSStatusItem.variableLength)
        
        if let button = statusItem.button {
            button.title = "📭" // Default icon
            button.action = #selector(menuBarButtonClicked)
            button.target = self
        }
        
        menu = NSMenu()
        setupMenu()
        statusItem.menu = menu
    }
    
    private func setupMenu() {
        menu.removeAllItems()
        
        // Header
        let headerItem = NSMenuItem(title: "PR Status Monitor", action: nil, keyEquivalent: "")
        headerItem.isEnabled = false
        menu.addItem(headerItem)
        menu.addItem(NSMenuItem.separator())
        
        if let status = currentStatus {
            addStatusItems(status)
        } else if let error = lastError {
            addErrorItems(error)
        } else {
            let loadingItem = NSMenuItem(title: "Loading...", action: nil, keyEquivalent: "")
            loadingItem.isEnabled = false
            menu.addItem(loadingItem)
        }
        
        menu.addItem(NSMenuItem.separator())
        
        // Refresh button
        let refreshItem = NSMenuItem(title: "🔄 Refresh Now", action: #selector(refreshStatus), keyEquivalent: "r")
        refreshItem.target = self
        menu.addItem(refreshItem)
        
        // Settings button
        let settingsItem = NSMenuItem(title: "⚙️ Settings...", action: #selector(showSettings), keyEquivalent: ",")
        settingsItem.target = self
        menu.addItem(settingsItem)
        
        // Quit button
        let quitItem = NSMenuItem(title: "Quit", action: #selector(quitApp), keyEquivalent: "q")
        quitItem.target = self
        menu.addItem(quitItem)
    }
    
    private func addErrorItems(_ error: String) {
        // Error icon and message
        let errorItem = NSMenuItem(title: "❌ Error", action: nil, keyEquivalent: "")
        errorItem.isEnabled = false
        menu.addItem(errorItem)
        
        // Specific error message
        let messageItem = NSMenuItem(title: error, action: nil, keyEquivalent: "")
        messageItem.isEnabled = false
        menu.addItem(messageItem)
        
        // Helpful instructions based on error type
        if error.contains("GitHub token") || error.contains("authentication") || error.contains("unauthorized") {
            let helpItem = NSMenuItem(title: "💡 Set GITHUB_TOKEN in Terminal and restart app", action: nil, keyEquivalent: "")
            helpItem.isEnabled = false
            menu.addItem(helpItem)
        } else if error.contains("CLI tool not found") {
            let helpItem = NSMenuItem(title: "💡 Check Settings > CLI Tool Path", action: nil, keyEquivalent: "")
            helpItem.isEnabled = false
            menu.addItem(helpItem)
        }
    }
    
    private func addStatusItems(_ status: StatusSummary) {
        // Overall status
        let statusText = "Total PRs: \(status.total_prs)"
        let statusItem = NSMenuItem(title: statusText, action: nil, keyEquivalent: "")
        statusItem.isEnabled = false
        menu.addItem(statusItem)
        
        let lastUpdated = NSMenuItem(title: "Updated: \(formatTime(status.last_updated))", action: nil, keyEquivalent: "")
        lastUpdated.isEnabled = false
        menu.addItem(lastUpdated)
        
        menu.addItem(NSMenuItem.separator())
        
        // Failing PRs
        if !status.failing_prs.isEmpty {
            let failingHeader = NSMenuItem(title: "🔴 Failing (\(status.failing_prs.count))", action: nil, keyEquivalent: "")
            failingHeader.isEnabled = false
            menu.addItem(failingHeader)
            
            addPRsGroupedByRepository(status.failing_prs)
            menu.addItem(NSMenuItem.separator())
        }
        
        // Running PRs
        if !status.running_prs.isEmpty {
            let runningHeader = NSMenuItem(title: "🔄 Running (\(status.running_prs.count))", action: nil, keyEquivalent: "")
            runningHeader.isEnabled = false
            menu.addItem(runningHeader)
            
            addPRsGroupedByRepository(status.running_prs)
            menu.addItem(NSMenuItem.separator())
        }
        
        // No CI/CD data PRs (less important)
        if !status.review_prs.isEmpty {
            let noChecksHeader = NSMenuItem(title: "📋 No CI/CD Data (\(status.review_prs.count))", action: nil, keyEquivalent: "")
            noChecksHeader.isEnabled = false
            menu.addItem(noChecksHeader)
            
            addPRsGroupedByRepository(status.review_prs)
            menu.addItem(NSMenuItem.separator())
        }
        
        // Passing PRs (includes "Changes requested" since CI is green)
        if !status.passing_prs.isEmpty {
            let passingHeader = NSMenuItem(title: "✅ Passing (\(status.passing_prs.count))", action: nil, keyEquivalent: "")
            passingHeader.isEnabled = false
            menu.addItem(passingHeader)
            
            addPRsGroupedByRepository(status.passing_prs)
        }
        
        if status.total_prs == 0 {
            let noPRsItem = NSMenuItem(title: "📭 No open PRs found", action: nil, keyEquivalent: "")
            noPRsItem.isEnabled = false
            menu.addItem(noPRsItem)
        }
    }
    
    private func addPRsGroupedByRepository(_ prs: [PRStatus]) {
        // Group PRs by repository
        let groupedPRs = Dictionary(grouping: prs) { $0.repository }
        
        // Sort repositories alphabetically
        let sortedRepos = groupedPRs.keys.sorted()
        
        for repository in sortedRepos {
            guard let prsInRepo = groupedPRs[repository] else { continue }
            
            // Add repository header
            let repoHeader = createRepositoryHeader(repository)
            menu.addItem(repoHeader)
            
            // Add PRs for this repository
            for pr in prsInRepo {
                let prItem = createPRMenuItem(pr)
                menu.addItem(prItem)
            }
        }
    }
    
    private func createPRMenuItem(_ pr: PRStatus) -> NSMenuItem {
        let title = "  #\(pr.number) \(truncateString(pr.title, maxLength: 40))"
        let menuItem = NSMenuItem(title: title, action: #selector(openPRInBrowser(_:)), keyEquivalent: "")
        menuItem.target = self
        menuItem.representedObject = pr.url
        
        // Add draft indicator
        if pr.is_draft {
            menuItem.title += " [DRAFT]"
        }
        
        return menuItem
    }
    
    private func createRepositoryHeader(_ repository: String) -> NSMenuItem {
        let repoItem = NSMenuItem(title: "  📁 \(repository)", action: nil, keyEquivalent: "")
        repoItem.isEnabled = false
        return repoItem
    }
    
    private func truncateString(_ string: String, maxLength: Int) -> String {
        if string.count <= maxLength {
            return string
        }
        return String(string.prefix(maxLength - 3)) + "..."
    }
    
    private func formatTime(_ timeString: String) -> String {
        let formatter = DateFormatter()
        formatter.dateFormat = "yyyy-MM-dd HH:mm:ss"
        formatter.timeZone = TimeZone.current // Use local timezone
        
        if let date = formatter.date(from: timeString) {
            formatter.dateFormat = "HH:mm"
            return formatter.string(from: date)
        }
        return timeString
    }
    
    private func updateMenuBarIcon(_ status: StatusSummary?) {
        guard let button = statusItem.button else { return }
        
        let icon: String
        if let status = status {
            // Priority 1: Show running icon if any PRs are running
            if !status.running_prs.isEmpty {
                icon = "🔄"  // Running icon - any PRs are running
            }
            // Priority 2: Show red dot if no PRs are running but any have failed
            else if !status.failing_prs.isEmpty {
                icon = "🔴"  // Red dot - failures need attention
            }
            // Priority 3: Show green if all PRs are passing
            else if !status.passing_prs.isEmpty {
                icon = "🟢"  // Green - all good
            }
            // Default: No PRs or no CI/CD data
            else {
                icon = "📭"  // No PRs or no CI/CD data
            }
        } else {
            // Error state - show warning icon
            icon = "⚠️"
        }
        
        DispatchQueue.main.async {
            button.title = icon
        }
    }
    
    private func startPeriodicUpdates() {
        let interval = TimeInterval(getRefreshInterval())
        timer = Timer.scheduledTimer(withTimeInterval: interval, repeats: true) { _ in
            self.updateStatus()
        }
    }
    
    @objc private func menuBarButtonClicked() {
        // Menu will be shown automatically
    }
    
    @objc private func refreshStatus() {
        updateStatus()
    }
    
    @objc private func openPRInBrowser(_ sender: NSMenuItem) {
        guard let urlString = sender.representedObject as? String,
              let url = URL(string: urlString) else { return }
        NSWorkspace.shared.open(url)
    }
    
    @objc private func showSettings() {
        if settingsWindowController == nil {
            settingsWindowController = SettingsWindowController()
        }
        settingsWindowController?.showWindow()
    }
    
    @objc private func settingsUpdated() {
        // Restart timer with new interval
        timer?.invalidate()
        startPeriodicUpdates()
        
        // Refresh immediately
        updateStatus()
    }
    
    
    @objc private func quitApp() {
        NSApplication.shared.terminate(nil)
    }
    
    private func updateStatus() {
        print("🔄 Updating PR status...")
        DispatchQueue.global(qos: .background).async {
            self.fetchPRStatus { [weak self] status in
                DispatchQueue.main.async {
                    if let status = status {
                        print("✅ Status update successful: \(status.total_prs) PRs, overall: \(status.overall_status)")
                        self?.currentStatus = status
                        self?.lastError = nil // Clear any previous errors
                        self?.updateMenuBarIcon(status)
                    } else {
                        print("❌ Status update failed")
                        self?.currentStatus = nil
                        // Set appropriate error message
                        if !FileManager.default.fileExists(atPath: self?.cliToolPath ?? "") {
                            self?.lastError = "CLI tool not found at: \(self?.cliToolPath ?? "unknown")"
                        } else {
                            self?.lastError = "GitHub token missing or invalid. Set GITHUB_TOKEN and restart app."
                        }
                        self?.updateMenuBarIcon(nil)
                    }
                    self?.setupMenu()
                }
            }
        }
    }
    
    private func fetchPRStatus(completion: @escaping (StatusSummary?) -> Void) {
        print("🔧 CLI Tool Path: \(cliToolPath)")
        print("🔧 CLI Arguments: status-check --json")
        
        // Check if CLI tool exists
        guard FileManager.default.fileExists(atPath: cliToolPath) else {
            print("❌ CLI tool not found at: \(cliToolPath)")
            completion(nil)
            return
        }
        
        let process = Process()
        process.executableURL = URL(fileURLWithPath: cliToolPath)
        process.arguments = ["status-check", "--json"]
        
        let pipe = Pipe()
        process.standardOutput = pipe
        process.standardError = pipe
        
        do {
            print("🚀 Starting CLI process...")
            try process.run()
            process.waitUntilExit()
            
            let data = pipe.fileHandleForReading.readDataToEndOfFile()
            let output = String(data: data, encoding: .utf8) ?? "No output"
            print("📤 CLI Output: \(output)")
            print("📊 CLI Exit Code: \(process.terminationStatus)")
            
            if process.terminationStatus == 0 {
                do {
                    let status = try JSONDecoder().decode(StatusSummary.self, from: data)
                    print("✅ Successfully parsed JSON response")
                    completion(status)
                } catch {
                    print("❌ Failed to decode JSON: \(error)")
                    completion(nil)
                }
            } else {
                let errorOutput = String(data: data, encoding: .utf8) ?? "Unknown error"
                print("❌ CLI tool failed: \(errorOutput)")
                
                // Check for specific error patterns
                if errorOutput.contains("GITHUB_TOKEN") || 
                   errorOutput.contains("authentication") || 
                   errorOutput.contains("unauthorized") ||
                   errorOutput.contains("401") {
                    print("🔑 Detected missing or invalid GitHub token")
                    completion(nil) // This will trigger the error state
                } else {
                    completion(nil)
                }
            }
        } catch {
            print("❌ Failed to run CLI tool: \(error)")
            completion(nil)
        }
    }
    
    // MARK: - Configuration Management
    private func getCliToolPath() -> String {
        if let savedPath = UserDefaults.standard.string(forKey: "CLIToolPath"), !savedPath.isEmpty {
            return savedPath
        }
        
        
        // Default fallback paths
        let defaultPaths = [
            "/Users/\(NSUserName())/repos/prtracker/target/release/my-prs",
            "/usr/local/bin/my-prs",
            "/opt/homebrew/bin/my-prs"
        ]
        
        for path in defaultPaths {
            if FileManager.default.fileExists(atPath: path) {
                saveCliToolPath(path)
                return path
            }
        }
        
        // Last resort - return a reasonable default
        return "/Users/\(NSUserName())/repos/prtracker/target/release/my-prs"
    }
    
    private func saveCliToolPath(_ path: String) {
        UserDefaults.standard.set(path, forKey: "CLIToolPath")
    }
    
    private func getRefreshInterval() -> Int {
        let interval = UserDefaults.standard.integer(forKey: "RefreshInterval")
        return interval > 0 ? interval : 120 // Default 2 minutes
    }
    
    private func saveRefreshInterval(_ interval: Int) {
        UserDefaults.standard.set(interval, forKey: "RefreshInterval")
    }
    
}

// MARK: - Main
@main
struct prstatusApp: App {
    @NSApplicationDelegateAdaptor(PRStatusMenuBarApp.self) var appDelegate
    
    var body: some Scene {
        Settings {
            EmptyView()
        }
    }
}
