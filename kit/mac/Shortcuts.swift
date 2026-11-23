// The keys every app answers to, in the menu bar (docs/design.md):
//   ⌘, Settings · ⌘N New · ⌘F Find · ⌃⌘S Toggle sidebar · ⌘R Refresh
//   ⌘? Keyboard shortcuts · Esc / ⌘W close a sheet · ⌘↩ save · ⌘⌫ delete
// An app passes what each does (nil leaves it out) and its own extras, e.g.
//   .commands { UuobCommands(settings: { model.settingsOpen = true }, new: { … }, extra: [("Today", "t", []) …]) }

import SwiftUI

/// One shortcut for the menu and the list: its name, key and modifiers, what it does.
struct Shortcut {
    let title: String
    let key: KeyEquivalent
    var modifiers: EventModifiers = .command
    var action: () -> Void

    /// "⌘N", "⌃⌘S", "←".
    var label: String {
        var s = ""
        if modifiers.contains(.control) { s += "⌃" }
        if modifiers.contains(.option) { s += "⌥" }
        if modifiers.contains(.shift) { s += "⇧" }
        if modifiers.contains(.command) { s += "⌘" }
        switch key {
        case .leftArrow: s += "←"
        case .rightArrow: s += "→"
        case .upArrow: s += "↑"
        case .downArrow: s += "↓"
        case .return: s += "↩"
        case .delete: s += "⌫"
        case .escape: s += "Esc"
        case .space: s += "Space"
        default: s += String(key.character).uppercased()
        }
        return s
    }
}

/// Shows the list of shortcuts (⌘?).
extension Notification.Name {
    static let uuobShowShortcuts = Notification.Name("uuob.showShortcuts")
}

/// The menu bar commands every app shares, plus its own (`extra`, in a menu named `extraMenu`).
struct UuobCommands: Commands {
    var settings: (() -> Void)? = nil
    var new: (() -> Void)? = nil
    var newTitle = "New"
    var find: (() -> Void)? = nil
    var refresh: (() -> Void)? = nil
    var extraMenu = "Go"
    var extra: [Shortcut] = []

    var body: some Commands {
        CommandGroup(replacing: .appSettings) {
            if let settings { Button("Settings…", action: settings).keyboardShortcut(",") }
        }
        CommandGroup(replacing: .newItem) {
            if let new { Button("\(newTitle)…", action: new).keyboardShortcut("n") }
        }
        CommandGroup(after: .textEditing) {
            if let find { Button("Find…", action: find).keyboardShortcut("f") }
        }
        CommandGroup(replacing: .sidebar) {
            Button("Toggle Sidebar") { NotificationCenter.default.post(name: .uuobToggleSidebar, object: nil) }
                .keyboardShortcut("s", modifiers: [.control, .command])
            if let refresh { Button("Refresh", action: refresh).keyboardShortcut("r") }
        }
        if !extra.isEmpty {
            CommandMenu(extraMenu) {
                ForEach(Array(extra.enumerated()), id: \.offset) { _, s in
                    Button(s.title, action: s.action).keyboardShortcut(s.key, modifiers: s.modifiers)
                }
            }
        }
        CommandGroup(replacing: .help) {
            Button("Keyboard Shortcuts") { NotificationCenter.default.post(name: .uuobShowShortcuts, object: nil) }
                .keyboardShortcut("/", modifiers: [.command, .shift])
        }
    }

    /// Every shortcut, for the list.
    var all: [(String, String)] {
        var rows: [(String, String)] = []
        if settings != nil { rows.append(("Settings", "⌘,")) }
        if new != nil { rows.append((newTitle, "⌘N")) }
        if find != nil { rows.append(("Find", "⌘F")) }
        rows.append(("Toggle sidebar", "⌃⌘S"))
        if refresh != nil { rows.append(("Refresh", "⌘R")) }
        rows += extra.map { ($0.title, $0.label) }
        rows += [("Save", "⌘↩"), ("Close or cancel", "Esc"), ("Keyboard shortcuts", "⌘?")]
        return rows
    }
}

/// The list of shortcuts, in the detail card's style.
struct ShortcutsSheet: View {
    let rows: [(String, String)]
    var onClose: () -> Void
    var body: some View {
        DetailSheet(title: "Keyboard shortcuts", onClose: onClose, actions: { EmptyView() }) {
            VStack(spacing: 0) {
                ForEach(Array(rows.enumerated()), id: \.offset) { i, r in
                    IconRow(nil, last: i == rows.count - 1) {
                        Text(r.0)
                        Spacer()
                        Text(r.1).font(.system(size: 14, weight: .medium, design: .rounded)).foregroundStyle(Look.text2)
                            .padding(.horizontal, 8).padding(.vertical, 3).background(Look.surface2, in: RoundedRectangle(cornerRadius: 6))
                    }
                }
            }
            .padding(.top, 10)
        }
    }
}

extension View {
    /// Shows the shortcut list when ⌘? is pressed.
    func shortcutsSheet(_ rows: [(String, String)]) -> some View {
        modifier(ShortcutsHost(rows: rows))
    }
}

private struct ShortcutsHost: ViewModifier {
    let rows: [(String, String)]
    @State private var shown = false
    func body(content: Content) -> some View {
        content
            .onReceive(NotificationCenter.default.publisher(for: .uuobShowShortcuts)) { _ in shown = true }
            .sheet(isPresented: $shown) { ShortcutsSheet(rows: rows) { shown = false } }
    }
}
