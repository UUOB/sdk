// Settings, the same sheet in every app (ui/Settings.svelte), and the time
// field (ui/TimeField.svelte). The choices at the top belong to all of the
// user's apps (kept in UUOB, so a change reaches every app on every device);
// an app adds its own sections below them; About comes last.

import SwiftUI

/// The settings for all apps: times as 14:30 or 2:30 PM, weeks from Sunday.
@Observable
@MainActor
final class UuobSettings {
    var hour12 = false
    var sundayFirst = false
    private let uuob: Uuob

    init(_ uuob: Uuob) {
        self.uuob = uuob
        apply(J(text: uuob.client.cachedSettings()))
        Task { await refresh() }
        Task {
            for await e in uuob.events() {
                switch e.type.string {
                case "settings": apply(e.settings)
                case "changed": await refresh()
                case "state" where e.state.string == "connected": await refresh()
                default: break
                }
            }
        }
    }

    private func apply(_ s: J) {
        if s.hour12.value != nil { hour12 = s.hour12.bool }
        if s.sunday_first.value != nil { sundayFirst = s.sunday_first.bool }
    }

    func refresh() async { apply(J(text: await uuob.client.settings())) }

    /// Changes some of them, e.g. `["hour12": true]`; shown at once.
    func set(_ change: [String: Any]) {
        apply(J(change))
        let text = J(change).text
        Task { _ = await uuob.client.setSettings(change: text) }
    }
}

/// Settings, in the event card's style: rows with an icon, a hairline
/// between them, no boxes. The app's own rows (`extra`) come after the ones
/// for all apps; the version is a small line at the end.
struct SettingsSheet<Extra: View>: View {
    let app: AppInfo
    @Bindable var settings: UuobSettings
    var week = false
    @ViewBuilder var extra: () -> Extra
    @Environment(\.dismiss) private var dismiss

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 0) {
                HStack {
                    Text("Settings").font(.system(size: 22, weight: .semibold)).tracking(-0.44)
                    Spacer()
                    IconButton("close", size: 19, help: "Close") { dismiss() }.keyboardShortcut(.cancelAction)
                }
                .padding(.bottom, 4)
                RowsLabel("All your apps")
                IconRow("clock", last: !week) {
                    Text("Time")
                    Spacer(minLength: 8)
                    Segmented([("24", "14:30"), ("12", "2:30 PM")], selected: settings.hour12 ? "12" : "24") { settings.set(["hour12": $0 == "12"]) }
                }
                if week {
                    IconRow("calendar", last: true) {
                        Text("Week starts")
                        Spacer(minLength: 8)
                        Segmented([("mon", "Monday"), ("sun", "Sunday")], selected: settings.sundayFirst ? "sun" : "mon") { settings.set(["sunday_first": $0 == "sun"]) }
                    }
                }
                extra()
                Text("\(app.name) \(app.version)").font(.system(size: 12)).foregroundStyle(Look.text3)
                    .frame(maxWidth: .infinity).padding(.top, 22)
            }
            .padding(22)
        }
        .frame(width: 440).frame(minHeight: 240, maxHeight: 620)
        .fixedSize(horizontal: false, vertical: true)
        .background(Look.elevated)
        .foregroundStyle(Look.text)
        .buttonStyle(UuobButton())
        .toggleStyle(UuobSwitch())
    }
}

/// A time field that writes times the way the user chose (14:30 or 2:30 PM).
/// Typing "1430", "9", "9.15" or "2pm" works too; anything else goes back to
/// the time it had. With `optional`, emptying the field makes it nil.
struct TimeField: View {
    @Binding var value: String?
    var hour12: Bool
    var optional = false
    var placeholder = ""
    @State private var text = ""
    @FocusState private var focused: Bool

    private func shown() -> String { value.map { friendlyTime(time: $0, hour12: hour12) } ?? "" }

    var body: some View {
        TextField(placeholder, text: $text)
            .focused($focused)
            .onSubmit(commit)
            .onAppear { text = shown() }
            .onChange(of: value) { if !focused { text = shown() } }
            .onChange(of: hour12) { if !focused { text = shown() } }
            .onChange(of: focused) { if !focused { commit() } }
    }

    private func commit() {
        if let t = parseTime(text: text) {
            if t != value { value = t }
        } else if optional && text.trimmingCharacters(in: .whitespaces).isEmpty {
            if value != nil { value = nil }
        }
        text = shown()
    }
}
