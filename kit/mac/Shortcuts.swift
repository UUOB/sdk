// The keys every app answers to, in the menu bar (docs/design.md), with
// Linear's habits: ⌘K for a palette of every action, single letters where
// no field is typing (C new, / find, ? the list).
//   ⌘K Command palette · ⌘, Settings · ⌘N New · ⌘F Find · ⌃⌘S Toggle sidebar
//   ⌘R Refresh · ⌘? Keyboard shortcuts · Esc / ⌘W close a sheet · ⌘↩ save
// An app passes what each does (nil leaves it out) and its own extras, e.g.
//   .commands { UuobCommands(settings: { model.settingsOpen = true }, new: { … }, extra: [("Today", "t", []) …]) }

import AppKit
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

/// Shows the list of shortcuts (⌘?) or the command palette (⌘K).
extension Notification.Name {
    static let uuobShowShortcuts = Notification.Name("uuob.showShortcuts")
    static let uuobShowPalette = Notification.Name("uuob.showPalette")
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
        CommandGroup(before: .sidebar) {
            Button("Command Palette…") { NotificationCenter.default.post(name: .uuobShowPalette, object: nil) }.keyboardShortcut("k")
        }
        CommandGroup(replacing: .help) {
            Button("Keyboard Shortcuts") { NotificationCenter.default.post(name: .uuobShowShortcuts, object: nil) }
                .keyboardShortcut("/", modifiers: [.command, .shift])
        }
    }

    /// Every action with its keys, for the palette.
    var actions: [(String, String, () -> Void)] {
        var out: [(String, String, () -> Void)] = []
        if let new { out.append((newTitle, "⌘N  C", new)) }
        if let find { out.append(("Find", "⌘F  /", find)) }
        out += extra.map { ($0.title, $0.label, $0.action) }
        if let settings { out.append(("Settings", "⌘,", settings)) }
        out.append(("Toggle sidebar", "⌃⌘S", { NotificationCenter.default.post(name: .uuobToggleSidebar, object: nil) }))
        if let refresh { out.append(("Refresh", "⌘R", refresh)) }
        out.append(("Keyboard shortcuts", "⌘?", { NotificationCenter.default.post(name: .uuobShowShortcuts, object: nil) }))
        // One row per name: an app's own "Settings" page and the shared ⌘, are the same thing to read.
        var seen = Set<String>()
        return out.filter { seen.insert($0.0.lowercased()).inserted }
    }

    /// Every shortcut, for the list.
    var all: [(String, String)] {
        var rows: [(String, String)] = [("Command palette", "⌘K")]
        if settings != nil { rows.append(("Settings", "⌘,")) }
        if new != nil { rows.append((newTitle, "⌘N or C")) }
        if find != nil { rows.append(("Find", "⌘F or /")) }
        rows.append(("Toggle sidebar", "⌃⌘S"))
        if refresh != nil { rows.append(("Refresh", "⌘R")) }
        rows += extra.map { ($0.title, $0.label) }
        rows += [("Save", "⌘↩"), ("Close or cancel", "Esc"), ("Keyboard shortcuts", "⌘? or ?")]
        var seen = Set<String>()
        return rows.filter { seen.insert($0.0.lowercased()).inserted }
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

/// Linear's command palette: type to narrow every action down, ↑ ↓ to pick, ↩ to run.
struct CommandPalette: View {
    let actions: [(String, String, () -> Void)]
    var onClose: () -> Void
    @State private var query = ""
    @State private var picked = 0
    @FocusState private var typing: Bool

    private var shown: [(String, String, () -> Void)] {
        let q = query.lowercased().trimmingCharacters(in: .whitespaces)
        return q.isEmpty ? actions : actions.filter { $0.0.lowercased().contains(q) }
    }

    var body: some View {
        let list = shown
        VStack(spacing: 0) {
            HStack(spacing: 10) {
                Ico("search", size: 18).foregroundStyle(Look.text2)
                TextField("Type a command…", text: $query).textFieldStyle(.plain).font(.system(size: 17)).focused($typing)
                    .onSubmit { run(list) }
                    .onChange(of: query) { picked = 0 }
            }
            .padding(.horizontal, 18).frame(height: 54)
            Rectangle().fill(Look.line).frame(height: 1)
            ScrollView {
                VStack(spacing: 2) {
                    ForEach(Array(list.enumerated()), id: \.offset) { i, a in
                        Button { picked = i; run(list) } label: {
                            HStack {
                                Text(a.0).font(.system(size: 15))
                                Spacer()
                                Text(a.1).font(.system(size: 13, weight: .medium, design: .rounded)).foregroundStyle(Look.text2)
                            }
                            .padding(.horizontal, 12).frame(height: 36)
                            .background(i == picked ? Look.active : .clear, in: RoundedRectangle(cornerRadius: 8, style: .continuous))
                            .contentShape(Rectangle())
                        }
                        .buttonStyle(.plain)
                    }
                    if list.isEmpty { Text("Nothing matches").font(.system(size: 15)).foregroundStyle(Look.text3).padding(.vertical, 18) }
                }
                .padding(8)
            }
            .frame(maxHeight: 340)
        }
        .frame(width: sheetWidth(520))
        .background(Look.elevated)
        .foregroundStyle(Look.text)
        .onAppear { typing = true }
        .onKeyPress(.downArrow) { picked = min(picked + 1, max(0, list.count - 1)); return .handled }
        .onKeyPress(.upArrow) { picked = max(picked - 1, 0); return .handled }
        .onKeyPress(.escape) { onClose(); return .handled }
    }

    private func run(_ list: [(String, String, () -> Void)]) {
        guard list.indices.contains(picked) else { return }
        let action = list[picked].2
        onClose()
        // After the palette has gone, so a sheet the action opens can show.
        DispatchQueue.main.async { action() }
    }
}

extension View {
    /// Everything the shared keys need on an app's main view: the shortcut
    /// list (⌘?), the palette (⌘K), and Linear's single letters when no field
    /// is typing: C new, / find, ? the list.
    func uuobKeys(_ commands: UuobCommands) -> some View {
        modifier(KeysHost(commands: commands))
    }
}

private struct KeysHost: ViewModifier {
    let commands: UuobCommands
    @State private var list = false
    @State private var palette = false
    @State private var monitor: Any?
    func body(content: Content) -> some View {
        content
            // The window's own keys, before any view: SwiftUI's onKeyPress only
            // reaches a view that has the keyboard focus, which a page rarely has.
            .onAppear {
                guard monitor == nil else { return }
                monitor = NSEvent.addLocalMonitorForEvents(matching: .keyDown) { e in
                    letter(e) ? nil : e
                }
            }
            .onDisappear {
                if let m = monitor { NSEvent.removeMonitor(m) }
                monitor = nil
            }
            .onReceive(NotificationCenter.default.publisher(for: .uuobShowShortcuts)) { _ in list = true }
            .onReceive(NotificationCenter.default.publisher(for: .uuobShowPalette)) { _ in palette = true }
            .sheet(isPresented: $list) { ShortcutsSheet(rows: commands.all) { list = false } }
            .sheet(isPresented: $palette) { CommandPalette(actions: commands.actions) { palette = false } }
    }

    /// C new, / find, ? the list: only with no modifier, in the main window
    /// (not a sheet), and where no text field is taking the typing.
    private func letter(_ e: NSEvent) -> Bool {
        guard e.modifierFlags.intersection([.command, .control, .option]).isEmpty,
              let w = e.window, w.sheetParent == nil, w.attachedSheet == nil, !list, !palette,
              !(w.firstResponder is NSText) else { return false }
        switch e.charactersIgnoringModifiers {
        case "c": if let n = commands.new { n(); return true }
        case "/": if let f = commands.find { f(); return true }
        case "?": list = true; return true
        default: break
        }
        return false
    }
}
