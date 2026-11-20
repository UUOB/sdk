// The UUOB SDK for a Mac app (client/, through UniFFI): one per
// app. The app has no backend of its own; its data lives in UUOB. The class
// it wraps (UuobClient) is written by UniFFI when the app is built.
//
//   let uuob = Uuob(app)
//   try await uuob.client.put(collection: "photos", container: month, id: nil, value: record.text)
//   for await e in uuob.events() { if e.type.string == "changed" { reload() } }

import AppKit
import Foundation

/// A JSON value, read like `photo.taken.double` or `group.items.array`.
@dynamicMemberLookup
struct J: Identifiable, Equatable {
    let value: Any?

    init(_ value: Any?) { self.value = value is NSNull ? nil : value }
    /// From JSON text.
    init(text: String?) {
        self.init(text.flatMap { try? JSONSerialization.jsonObject(with: Data($0.utf8), options: [.fragmentsAllowed]) })
    }

    subscript(dynamicMember key: String) -> J { self[key] }
    subscript(key: String) -> J { J((value as? [String: Any])?[key]) }

    var str: String? { value as? String }
    var string: String { str ?? "" }
    var int: Int { (value as? NSNumber)?.intValue ?? 0 }
    var int64: Int64 { (value as? NSNumber)?.int64Value ?? 0 }
    var double: Double { (value as? NSNumber)?.doubleValue ?? 0 }
    var bool: Bool { (value as? NSNumber)?.boolValue ?? false }
    var isNull: Bool { value == nil }
    var array: [J] { ((value as? [Any]) ?? []).map(J.init) }
    var id: String { self["id"].string }
    /// As JSON text, for the SDK.
    var text: String {
        guard let v = value else { return "null" }
        guard let d = try? JSONSerialization.data(withJSONObject: v, options: [.fragmentsAllowed]) else { return "null" }
        return String(decoding: d, as: UTF8.self)
    }

    static func == (a: J, b: J) -> Bool { a.text == b.text }
}

/// Who the app is, for UUOB and its connect screen. `noun`: what it keeps
/// ("your photos"); `symbol`: an SF Symbol for its tile; `updateFeed`,
/// `updateKey`: where its own updates are published and their key.
struct AppInfo {
    let id: String
    let name: String
    let noun: String
    let symbol: String
    let color: String
    let version: String
    var updateFeed = ""
    var updateKey = ""
}

final class Uuob: @unchecked Sendable {
    let app: AppInfo
    let client: UuobClient
    private let lock = NSLock()
    private var listeners: [UUID: AsyncStream<J>.Continuation] = [:]

    init(_ app: AppInfo) {
        self.app = app
        let support = FileManager.default.urls(for: .applicationSupportDirectory, in: .userDomainMask)[0]
        let dir = ProcessInfo.processInfo.environment["UUOB_APP_DATA_DIR"] ?? support.appendingPathComponent("app.uuob.\(app.id)").path
        client = UuobClient(id: app.id, name: app.name, androidPackage: nil, dataDir: dir)
        // One reader of the SDK's events; every listener gets each of them.
        let client = self.client
        Task.detached { [weak self] in
            while !Task.isCancelled {
                let e = J(text: await client.nextEvent())
                guard let self else { return }
                self.lock.lock()
                let all = Array(self.listeners.values)
                self.lock.unlock()
                for c in all { c.yield(e) }
            }
        }
    }

    /// What changes: {"type":"changed","container":…}, {"type":"state",…}, {"type":"asking",…}, {"type":"settings",…}.
    func events() -> AsyncStream<J> {
        AsyncStream { continuation in
            let key = UUID()
            lock.lock()
            listeners[key] = continuation
            lock.unlock()
            continuation.onTermination = { [weak self] _ in
                guard let self else { return }
                self.lock.lock()
                self.listeners[key] = nil
                self.lock.unlock()
            }
        }
    }

    /// Brings UUOB to the front at `uuob://<what>` (connect, share).
    @MainActor func open(_ what: String) {
        if let url = URL(string: "uuob://\(what)") { NSWorkspace.shared.open(url) }
    }
}

extension Error {
    /// A failed call's message, for the user.
    var userMessage: String {
        if let e = self as? UuobError, case .Failed(_, let detail) = e { return detail }
        return localizedDescription
    }
    /// `unreachable`, `cancelled`, … ("" for other failures).
    var code: String {
        if let e = self as? UuobError, case .Failed(let code, _) = e { return code }
        return ""
    }
}
