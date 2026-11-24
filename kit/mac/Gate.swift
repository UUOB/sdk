// What every UUOB app's Mac window shares (ui/Gate.svelte and friends): the
// connect screen until the app is connected to UUOB, the "waiting for UUOB"
// note, UUOB's logo, the calendar colours and the update button.

import AppKit
import SwiftUI

/// UUOB's logo (Mesh), drawn as ui/Logo.svelte draws it.
struct UuobLogo: View {
    var size: CGFloat = 68
    var body: some View {
        Canvas { ctx, s in
            let k = s.width / 1024 * 1.08
            ctx.translateBy(x: s.width / 2, y: s.height / 2)
            ctx.scaleBy(x: k, y: k)
            ctx.translateBy(x: -512, y: -512)
            func line(_ x1: CGFloat, _ y1: CGFloat, _ x2: CGFloat, _ y2: CGFloat, _ w: CGFloat) {
                var p = Path(); p.move(to: CGPoint(x: x1, y: y1)); p.addLine(to: CGPoint(x: x2, y: y2))
                ctx.stroke(p, with: .color(.white), style: StrokeStyle(lineWidth: w, lineCap: .round))
            }
            func dot(_ x: CGFloat, _ y: CGFloat, _ r: CGFloat) {
                ctx.fill(Path(ellipseIn: CGRect(x: x - r, y: y - r, width: 2 * r, height: 2 * r)), with: .color(.white))
            }
            func box(_ x: CGFloat, _ y: CGFloat, _ w: CGFloat, _ h: CGFloat, _ r: CGFloat) {
                ctx.stroke(Path(roundedRect: CGRect(x: x, y: y, width: w, height: h), cornerRadius: r), with: .color(.white), lineWidth: 29.9)
            }
            line(430.5, 429.3, 349.1, 579.2, 22); dot(389.8, 504.3, 34)
            line(593.5, 429.3, 672.2, 574.3, 22); dot(632.8, 501.8, 34)
            line(435.5, 724.4, 582.9, 724.4, 22); dot(509.2, 724.4, 34)
            box(411.2, 192.4, 201.6, 134.4, 19.6); line(374.8, 360.4, 649.2, 360.4, 29.9)
            box(208.7, 612.4, 123.2, 224, 28); dot(270.3, 788.8, 15.4)
            box(647.3, 623.6, 212.8, 86.8, 19.6); box(647.3, 738.4, 212.8, 86.8, 19.6)
            dot(812.5, 667, 12.6); dot(812.5, 781.8, 12.6)
        }
        .frame(width: size, height: size)
        .background(.black)
        .clipShape(RoundedRectangle(cornerRadius: size * 0.28, style: .continuous))
    }
}

/// "Update to 0.4.1": appears when a newer version is published at the app's feed.
@Observable
@MainActor
final class Updater {
    var version: String?
    var busy = false
    var error: String?
    private var checked = Date.distantPast
    let app: AppInfo

    init(_ app: AppInfo) { self.app = app }

    /// When the app opens and when the user comes back to it, at most hourly.
    func check() {
        guard !busy, !app.updateFeed.isEmpty, Date().timeIntervalSince(checked) > 3600 else { return }
        checked = Date()
        Task { version = try? await updateCheck(feed: app.updateFeed, version: app.version) }
    }

    /// Downloads, checks the signature, swaps the app and starts it again.
    func install() {
        busy = true
        error = nil
        Task {
            do {
                let saved = J(text: try await updateInstall(feed: app.updateFeed, key: app.updateKey, version: app.version, dir: NSTemporaryDirectory()))
                if let app = saved.restart.str {
                    let task = Process()
                    task.launchPath = "/usr/bin/open"
                    task.arguments = ["-n", app]
                    try task.run()
                    NSApp.terminate(nil)
                }
            } catch {
                self.error = error.userMessage
            }
            busy = false
        }
    }
}

struct UpdateButton: View {
    @Bindable var updater: Updater
    var body: some View {
        if let v = updater.version {
            Button(updater.busy ? "Updating…" : updater.error != nil ? "Try again" : "Update to \(v)") { updater.install() }
                .buttonStyle(UuobButton(.primary))
                .disabled(updater.busy)
                .help(updater.error ?? "Install the new version")
        }
    }
}

/// A code is a number and three words, like 42-amber-river-falcon.
private func codeOk(_ code: String) -> Bool {
    code.split(whereSeparator: { !($0.isLetter || $0.isNumber) }).count == 4
}

/// The app until it is connected to UUOB, then `content`.
struct Gate<Content: View>: View {
    let uuob: Uuob
    @Bindable var updater: Updater
    @ViewBuilder var content: () -> Content

    private enum Step { case checking, unreachable, connect, waiting, ready }
    @State private var step: Step = .checking
    @State private var offline = false
    @State private var asking = false
    @State private var code = ""
    @State private var useCode = false
    @State private var busy = false
    @State private var error = ""

    var body: some View {
        Group {
            switch step {
            case .ready:
                content().overlay(alignment: .bottom) { note }
            case .checking:
                Color.clear
            default:
                ScrollView { gate.frame(maxWidth: 420).padding(.horizontal, 24).padding(.vertical, 40).frame(maxWidth: .infinity) }
                    .background(Look.bg)
            }
        }
        .task {
            if uuob.client.hasToken() { step = .ready }
            updater.check()
            Task { for await e in uuob.events() { handle(e) } }
            await check()
            while !Task.isCancelled {
                try? await Task.sleep(for: .seconds(3))
                if (step == .unreachable || offline) && !busy { await check() }
            }
        }
        .onReceive(NotificationCenter.default.publisher(for: NSApplication.didBecomeActiveNotification)) { _ in
            uuob.client.resume()
            updater.check()
        }
    }

    @ViewBuilder private var note: some View {
        if offline || asking {
            // The web version's toast: a small pill.
            Text(offline ? "Waiting for UUOB. Your changes are kept and sent when it’s back." : "Answer in UUOB on one of your devices.")
                .font(.system(size: 15, weight: .medium))
                .foregroundStyle(offline ? Look.warn : Look.text)
                .padding(.horizontal, 20).padding(.vertical, 12)
                .background(Look.elevated, in: Capsule())
                .overlay(Capsule().strokeBorder(Look.lineStrong))
                .shadow(color: .black.opacity(0.25), radius: 14, y: 8)
                .padding(24)
        }
    }

    private func handle(_ e: J) {
        switch e.type.string {
        case "state":
            let s = e.state.string
            if s == "disconnected" && step == .ready { step = .connect }
            offline = s == "unreachable" && step == .ready
        case "asking":
            asking = e.asking.bool
        default: break
        }
    }

    private func check() async {
        let s = await uuob.client.check()
        if step == .waiting { return }
        if s == "unreachable" && step == .ready { offline = true; return }
        offline = false
        step = s == "connected" ? .ready : s == "disconnected" ? .connect : .unreachable
    }

    private func connect(withCode: Bool) {
        error = ""
        let from = step
        if !withCode {
            step = .waiting
            uuob.open("connect")
        }
        busy = withCode
        Task {
            do {
                try await uuob.client.connect(code: withCode ? code.trimmingCharacters(in: .whitespaces) : nil)
                step = .ready
                offline = false
                code = ""
            } catch {
                if error.code == "cancelled" {
                    step = .connect
                } else {
                    self.error = error.userMessage
                    // Connected before (or refused): only a code from UUOB connects it now.
                    if !withCode && error.code == "permission_denied" { useCode = true }
                    // A refused code leaves the user where they typed it.
                    step = error.code == "unreachable" ? .unreachable : withCode ? from : .connect
                }
            }
            busy = false
        }
    }

    private var codeForm: some View {
        HStack(spacing: 10) {
            TextField("42-amber-river-falcon", text: $code)
                .textFieldStyle(UuobField())
                .autocorrectionDisabled()
                .onSubmit { if codeOk(code) && !busy { connect(withCode: true) } }
            Button(busy ? "Connecting…" : "Connect") { connect(withCode: true) }
                .buttonStyle(UuobButton(.primary))
                .disabled(!codeOk(code) || busy)
        }
        .padding(.top, 18)
    }

    private var gate: some View {
        let app = uuob.app
        return VStack(spacing: 0) {
            HStack { Spacer(); UpdateButton(updater: updater) }
            HStack(spacing: 14) {
                RoundedRectangle(cornerRadius: 19, style: .continuous).fill(Palette.color(app.color)).frame(width: 68, height: 68)
                    .overlay(Image(systemName: app.symbol).font(.system(size: 30, weight: .semibold)).foregroundStyle(.white))
                HStack(spacing: 6) { ForEach(0..<3) { _ in Circle().fill(step == .waiting ? Look.accent : Look.text3).frame(width: 6, height: 6) } }
                UuobLogo(size: 68)
            }
            .padding(.bottom, 28)
            switch step {
            case .unreachable:
                title("\(app.name) needs UUOB")
                lead("\(app.name) has no server of its own. It keeps your \(app.noun) in UUOB, on your own devices, end-to-end encrypted.")
                lead("Install UUOB on this device and open it.")
                looking("Looking for UUOB…")
                Rectangle().fill(Look.line).frame(height: 1).padding(.top, 32).padding(.bottom, 28)
                Text("Or use UUOB on another device").font(.system(size: 17, weight: .semibold)).foregroundStyle(Look.text)
                lead("In UUOB there, open Apps, choose Connect with a code, and type the code here.")
                codeForm
                errorText
            case .waiting:
                title("Allow it in UUOB")
                lead("UUOB is asking whether \(app.name) may keep its \(app.noun) in your network.")
                looking("Waiting for your answer…")
                Button("Cancel") { uuob.client.cancelConnect() }.buttonStyle(UuobButton(large: true)).environment(\.fillButtons, true)
            default:
                title("Connect \(app.name)")
                lead("\(app.name) has no server of its own. Your \(app.noun) live in UUOB, on your own devices, end-to-end encrypted.")
                HStack(spacing: 8) {
                    Circle().fill(Look.ok).frame(width: 8, height: 8)
                    Text("UUOB is running on \(uuob.client.device() ?? "this device")").font(.system(size: 15)).foregroundStyle(Look.text2)
                }
                .padding(.top, 22).padding(.bottom, 14)
                Button("Connect to UUOB") { connect(withCode: false) }
                    .buttonStyle(UuobButton(.primary, large: true)).environment(\.fillButtons, true)
                    .accessibilityLabel("Connect to UUOB")
                errorText
                if useCode {
                    codeForm
                    Text("In UUOB, open Apps and choose Connect with a code.").font(.system(size: 13)).foregroundStyle(Look.text2).padding(.top, 10)
                } else {
                    Button("Use a code instead") { useCode = true }.buttonStyle(UuobButton(.ghost)).padding(.top, 10)
                }
            }
        }
        .multilineTextAlignment(.center)
    }

    private func title(_ t: String) -> some View { Text(t).font(.system(size: 34, weight: .bold)).tracking(-0.85).foregroundStyle(Look.text) }
    private func lead(_ t: String) -> some View { Text(t).font(.system(size: 17)).foregroundStyle(Look.text2).padding(.top, 12).fixedSize(horizontal: false, vertical: true) }
    private func looking(_ t: String) -> some View {
        HStack(spacing: 8) { ProgressView().controlSize(.small); Text(t).font(.system(size: 15)).foregroundStyle(Look.text2) }.padding(.top, 22).padding(.bottom, 14)
    }
    @ViewBuilder private var errorText: some View {
        if !error.isEmpty { Text(error).font(.system(size: 15)).foregroundStyle(Look.danger).padding(.top, 12) }
    }
}
