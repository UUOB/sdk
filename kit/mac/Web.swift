// The controls every UUOB app's Mac window is made of, the same as the web
// version's (ui/tokens.css, ui/Icon.svelte): its colours light and dark, pill
// buttons, groups of rows with a hairline edge, the green switch, the
// segmented control, fields, sheets, large titles and the line icons.
// Started from the UUOB app's own mac/Sources/Kit.swift; kept the same.

import AppKit
import SwiftUI

// MARK: - Colours

extension Color {
    /// A colour for light and one for dark.
    init(light: UInt32, dark: UInt32, alpha: Double = 1) {
        func rgb(_ v: UInt32) -> NSColor {
            NSColor(srgbRed: CGFloat((v >> 16) & 0xff) / 255, green: CGFloat((v >> 8) & 0xff) / 255, blue: CGFloat(v & 0xff) / 255, alpha: alpha)
        }
        self.init(nsColor: NSColor(name: nil) { $0.bestMatch(from: [.darkAqua, .aqua]) == .darkAqua ? rgb(dark) : rgb(light) })
    }
}

/// The tokens.
enum Look {
    static let bg = Color(light: 0xf5f5f7, dark: 0x0b0b0d)
    static let elevated = Color(light: 0xffffff, dark: 0x161618)
    static let surface = Color(light: 0xffffff, dark: 0x1c1c1f)
    static let surface2 = Color(light: 0xf0f0f3, dark: 0x26262a)
    static let rail = Color(light: 0xececf0, dark: 0x141416)
    static let low = Color(light: 0xfafafb, dark: 0x161618)
    static let text = Color(light: 0x1d1d1f, dark: 0xf5f5f7)
    static let text2 = Color(light: 0x1d1d1f, dark: 0xf5f5f7, alpha: 0.62)
    static let text3 = Color(light: 0x1d1d1f, dark: 0xf5f5f7, alpha: 0.38)
    static let line = Color(light: 0x000000, dark: 0xffffff, alpha: 0.08)
    static let lineStrong = Color(light: 0x000000, dark: 0xffffff, alpha: 0.14)
    static let hover = Color(light: 0x000000, dark: 0xffffff, alpha: 0.06)
    static let active = Color(light: 0x000000, dark: 0xffffff, alpha: 0.1)
    static let accent = Color(light: 0x007aff, dark: 0x0a84ff)
    static let accentSoft = Color(light: 0x007aff, dark: 0x0a84ff, alpha: 0.16)
    static let danger = Color(light: 0xff3b30, dark: 0xff453a)
    static let dangerSoft = Color(light: 0xff3b30, dark: 0xff453a, alpha: 0.14)
    static let ok = Color(light: 0x34c759, dark: 0x30d158)
    static let warn = Color(red: 1, green: 0.62, blue: 0.04)
    /// A person's face: green to blue.
    static let person = LinearGradient(colors: [Color(red: 0.19, green: 0.77, blue: 0.55), Color(red: 0.16, green: 0.62, blue: 0.96)], startPoint: .topLeading, endPoint: .bottomTrailing)
}

// MARK: - Icons

/// UUOB's line icons (24 point grid, 1.75 stroke), as ui/Icon.svelte drew them.
let iconPaths: [String: String] = [
    "tasks": "M9 6h11M9 12h11M9 18h11M4 6l1 1 2-2M4 12l1 1 2-2M4 18l1 1 2-2",
    "calendar": "M4 7a2 2 0 0 1 2-2h12a2 2 0 0 1 2 2v12a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2zM4 10h16M8 3v4M16 3v4",
    "plus": "M12 5v14M5 12h14",
    "trash": "M4 7h16M10 11v6M14 11v6M6 7l1 12a2 2 0 0 0 2 2h6a2 2 0 0 0 2-2l1-12M9 7V4h6v3",
    "link": "M10 14a4 4 0 0 0 5.7 0l3-3a4 4 0 0 0-5.7-5.7l-1 1M14 10a4 4 0 0 0-5.7 0l-3 3a4 4 0 0 0 5.7 5.7l1-1",
    "close": "M6 6l12 12M18 6L6 18",
    "check": "M5 12l4 4 10-10",
    "download": "M12 4v11M7 10l5 5 5-5M5 20h14",
    "copy": "M9 9h10v10a1 1 0 0 1-1 1H10a1 1 0 0 1-1-1zM5 15V5a1 1 0 0 1 1-1h9",
    "lock": "M6 11h12v9H6zM8 11V8a4 4 0 0 1 8 0v3",
    "chevron": "M9 6l6 6-6 6",
    "back": "M15 6l-6 6 6 6",
    "left": "M15 6l-6 6 6 6",
    "right": "M9 6l6 6-6 6",
    "edit": "M4 20h4L19 9l-4-4L4 16zM13 7l4 4",
    "settings": "M12 15a3 3 0 1 0 0-6 3 3 0 0 0 0 6zM19.4 15a1.7 1.7 0 0 0 .3 1.8l.1.1a2 2 0 1 1-2.8 2.8l-.1-.1a1.7 1.7 0 0 0-1.8-.3 1.7 1.7 0 0 0-1 1.5V21a2 2 0 1 1-4 0v-.1a1.7 1.7 0 0 0-1.1-1.5 1.7 1.7 0 0 0-1.8.3l-.1.1a2 2 0 1 1-2.8-2.8l.1-.1a1.7 1.7 0 0 0 .3-1.8 1.7 1.7 0 0 0-1.5-1H3a2 2 0 1 1 0-4h.1a1.7 1.7 0 0 0 1.5-1.1 1.7 1.7 0 0 0-.3-1.8l-.1-.1a2 2 0 1 1 2.8-2.8l.1.1a1.7 1.7 0 0 0 1.8.3H9a1.7 1.7 0 0 0 1-1.5V3a2 2 0 1 1 4 0v.1a1.7 1.7 0 0 0 1 1.5 1.7 1.7 0 0 0 1.8-.3l.1-.1a2 2 0 1 1 2.8 2.8l-.1.1a1.7 1.7 0 0 0-.3 1.8V9a1.7 1.7 0 0 0 1.5 1H21a2 2 0 1 1 0 4h-.1a1.7 1.7 0 0 0-1.5 1z",
    "sun": "M12 8a4 4 0 1 1 0 8 4 4 0 0 1 0-8zM12 2v2M12 20v2M2 12h2M20 12h2M5 5l1.4 1.4M17.6 17.6L19 19M5 19l1.4-1.4M17.6 6.4L19 5",
    "list": "M8 6h12M8 12h12M8 18h12M4 6h.01M4 12h.01M4 18h.01",
    "inbox": "M4 13l2-8h12l2 8M4 13v6h16v-6M4 13h5l1 2h4l1-2h5",
    "menu": "M4 7h16M4 12h16M4 17h16",
    "people": "M9 11a3.5 3.5 0 1 0 0-7 3.5 3.5 0 0 0 0 7zM2.5 20a6.5 6.5 0 0 1 13 0M16 4.2a3.5 3.5 0 0 1 0 6.6M18 14.3a6.5 6.5 0 0 1 3.5 5.7",
    "share": "M12 15V3M8 7l4-4 4 4M7 11H5a1 1 0 0 0-1 1v8a1 1 0 0 0 1 1h14a1 1 0 0 0 1-1v-8a1 1 0 0 0-1-1h-2",
    "photo": "M4 6a2 2 0 0 1 2-2h12a2 2 0 0 1 2 2v12a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2zM4 16l4.5-4.5 3.5 3.5 3-3 5 5M9 9h.01",
    "play": "M8 5l11 7-11 7z",
    "bell": "M6 16V11a6 6 0 0 1 12 0v5l1.5 2h-15zM10 20.5a2 2 0 0 0 4 0",
    "place": "M12 21s-7-6.2-7-11.5a7 7 0 0 1 14 0C19 14.8 12 21 12 21zM12 12a2.5 2.5 0 1 0 0-5 2.5 2.5 0 0 0 0 5z",
    "video": "M3 7a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v10a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2zM16 10l5-3v10l-5-3",
    "repeat": "M17 2l4 4-4 4M3 11V9a3 3 0 0 1 3-3h15M7 22l-4-4 4-4M21 13v2a3 3 0 0 1-3 3H3",
    "clock": "M12 21a9 9 0 1 0 0-18 9 9 0 0 0 0 18zM12 7v5l3 2",
    "notes": "M5 7h14M5 12h14M5 17h9",
    "info": "M12 21a9 9 0 1 0 0-18 9 9 0 0 0 0 18zM12 11v5M12 8h.01",
    "at": "M16 12a4 4 0 1 1-8 0 4 4 0 0 1 8 0zM16 12v1.5a2.5 2.5 0 0 0 5 0V12a9 9 0 1 0-3.5 7.1",
]

/// One of the icons, as a shape on the 24 point grid.
struct IconShape: Shape {
    let name: String
    func path(in rect: CGRect) -> Path {
        svgPath(iconPaths[name] ?? "").applying(CGAffineTransform(scaleX: rect.width / 24, y: rect.height / 24))
    }
}

/// An icon, drawn in the current foreground colour.
struct Ico: View {
    let name: String
    var size: CGFloat = 20
    var stroke: CGFloat = 1.75
    init(_ name: String, size: CGFloat = 20, stroke: CGFloat = 1.75) {
        self.name = name
        self.size = size
        self.stroke = stroke
    }
    var body: some View {
        IconShape(name: name)
            .stroke(style: StrokeStyle(lineWidth: stroke * size / 24, lineCap: .round, lineJoin: .round))
            .frame(width: size, height: size)
    }
}

/// A path from SVG path data: M L H V C S Q T A Z, absolute and relative
/// (what the icons use).
func svgPath(_ d: String) -> Path {
    var path = Path()
    var tokens: [String] = []
    var number = ""
    func flush() { if !number.isEmpty { tokens.append(number); number = "" } }
    for ch in d {
        if ch.isLetter && ch != "e" {
            flush(); tokens.append(String(ch))
        } else if ch == "," || ch == " " {
            flush()
        } else if ch == "-" && !number.isEmpty && number.last != "e" {
            flush(); number = "-"
        } else if ch == "." && number.contains(".") {
            flush(); number = "."
        } else {
            number.append(ch)
        }
    }
    flush()
    var i = 0
    var command = "M"
    var current = CGPoint.zero, start = CGPoint.zero, control = CGPoint.zero
    func next() -> CGFloat { defer { i += 1 }; return CGFloat(Double(tokens[i]) ?? 0) }
    func hasNumber() -> Bool { i < tokens.count && !(tokens[i].first?.isLetter ?? false) }
    while i < tokens.count {
        if tokens[i].first?.isLetter == true { command = tokens[i]; i += 1 }
        let rel = command == command.lowercased()
        let base = rel ? current : .zero
        switch command.uppercased() {
        case "M":
            current = CGPoint(x: base.x + next(), y: base.y + next())
            path.move(to: current); start = current
            command = rel ? "l" : "L"
        case "L":
            current = CGPoint(x: base.x + next(), y: base.y + next()); path.addLine(to: current)
        case "H":
            current.x = (rel ? current.x : 0) + next(); path.addLine(to: current)
        case "V":
            current.y = (rel ? current.y : 0) + next(); path.addLine(to: current)
        case "C":
            let c1 = CGPoint(x: base.x + next(), y: base.y + next())
            let c2 = CGPoint(x: base.x + next(), y: base.y + next())
            current = CGPoint(x: base.x + next(), y: base.y + next())
            path.addCurve(to: current, control1: c1, control2: c2); control = c2
        case "S":
            let c1 = CGPoint(x: 2 * current.x - control.x, y: 2 * current.y - control.y)
            let c2 = CGPoint(x: base.x + next(), y: base.y + next())
            current = CGPoint(x: base.x + next(), y: base.y + next())
            path.addCurve(to: current, control1: c1, control2: c2); control = c2
        case "Q":
            let c = CGPoint(x: base.x + next(), y: base.y + next())
            current = CGPoint(x: base.x + next(), y: base.y + next())
            path.addQuadCurve(to: current, control: c); control = c
        case "A":
            let r = next(); _ = next(); _ = next()
            let large = next() != 0, sweep = next() != 0
            let end = CGPoint(x: base.x + next(), y: base.y + next())
            arc(&path, from: current, to: end, radius: r, large: large, sweep: sweep)
            current = end
        case "Z":
            path.closeSubpath(); current = start
        default:
            i += 1
        }
        if command.uppercased() == "Z" { continue }
        if !hasNumber() { continue }
    }
    return path
}

/// A circular arc from `a` to `b` (the icons' arcs are all circles).
private func arc(_ path: inout Path, from a: CGPoint, to b: CGPoint, radius: CGFloat, large: Bool, sweep: Bool) {
    let dx = (b.x - a.x) / 2, dy = (b.y - a.y) / 2
    let d = sqrt(dx * dx + dy * dy)
    guard d > 0 else { return }
    let r = max(radius, d)
    let h = sqrt(max(0, r * r - d * d))
    // The centre is on the perpendicular through the midpoint, on the side the flags choose.
    let sign: CGFloat = large == sweep ? -1 : 1
    let mid = CGPoint(x: (a.x + b.x) / 2, y: (a.y + b.y) / 2)
    let centre = CGPoint(x: mid.x + sign * h * (-dy / d), y: mid.y + sign * h * (dx / d))
    let start = atan2(a.y - centre.y, a.x - centre.x), end = atan2(b.y - centre.y, b.x - centre.x)
    // SVG's sweep 1 is clockwise on screen; SwiftUI's `clockwise` is for a flipped y.
    path.addArc(center: centre, radius: r, startAngle: .radians(start), endAngle: .radians(end), clockwise: !sweep)
}

// MARK: - Buttons

/// `quiet`: grey text, no fill (Settings at the foot of a sidebar).
enum Kind { case plain, primary, danger, ghost, quiet }

/// `.btn`: a pill, 40 points high (`.lg`: 50), grey, blue, red or just blue text.
/// A destructive button turns red by itself.
struct UuobButton: ButtonStyle {
    var kind: Kind = .plain
    var large = false
    /// `.btn.sm`: 32 high (Today in Calendar's header).
    var small = false
    init(_ kind: Kind = .plain, large: Bool = false, small: Bool = false) {
        self.kind = kind
        self.large = large
        self.small = small
    }
    @Environment(\.isEnabled) private var enabled
    /// Buttons side by side at the bottom of a sheet share its width (`.actions`).
    @Environment(\.fillButtons) private var fill
    func makeBody(configuration: Configuration) -> some View {
        let kind = configuration.role == .destructive && self.kind == .plain ? .danger : self.kind
        let (bg, fg): (Color, Color) = switch kind {
        case .plain: (Look.surface2, Look.text)
        case .primary: (Look.accent, .white)
        case .danger: (Look.dangerSoft, Look.danger)
        case .ghost: (.clear, Look.accent)
        case .quiet: (.clear, Look.text2)
        }
        return configuration.label
            .font(.system(size: large ? 17 : small ? 14 : 15, weight: .semibold))
            .foregroundStyle(fg)
            .padding(.horizontal, large ? 26 : small ? 14 : 18)
            .frame(maxWidth: fill ? .infinity : nil, minHeight: large ? 50 : small ? 32 : 40)
            .background(bg.opacity(configuration.isPressed ? 0.8 : 1), in: Capsule())
            .scaleEffect(configuration.isPressed ? 0.97 : 1)
            .opacity(enabled ? 1 : 0.45)
            .contentShape(Capsule())
            .animation(.easeOut(duration: 0.12), value: configuration.isPressed)
    }
}

private struct FillButtons: EnvironmentKey { static let defaultValue = false }
extension EnvironmentValues {
    var fillButtons: Bool {
        get { self[FillButtons.self] }
        set { self[FillButtons.self] = newValue }
    }
}

/// `.icon-btn`: round, 36 points, grey until hovered.
struct IconButtonStyle: ButtonStyle {
    @State private var hover = false
    func makeBody(configuration: Configuration) -> some View {
        configuration.label
            .foregroundStyle(hover ? Look.text : Look.text2)
            .frame(width: 36, height: 36)
            .background(hover || configuration.isPressed ? Look.hover : .clear, in: Circle())
            .contentShape(Circle())
            .onHover { hover = $0 }
    }
}

// MARK: - Switch and segments

/// `.switch`: 51 by 31, green when on; the knob springs across.
struct UuobSwitch: ToggleStyle {
    func makeBody(configuration: Configuration) -> some View {
        HStack(spacing: 14) {
            configuration.label
            Spacer(minLength: 0)
            Capsule().fill(configuration.isOn ? Look.ok : Look.lineStrong)
                .frame(width: 51, height: 31)
                .overlay(alignment: configuration.isOn ? .trailing : .leading) {
                    Circle().fill(.white).frame(width: 27, height: 27).padding(2).shadow(color: .black.opacity(0.3), radius: 2.5, y: 2)
                }
                .animation(.spring(response: 0.3, dampingFraction: 0.65), value: configuration.isOn)
                .onTapGesture { configuration.isOn.toggle() }
                .accessibilityElement().accessibilityAddTraits(.isButton).accessibilityValue(configuration.isOn ? "On" : "Off")
        }
    }
}

/// `.segmented`: a grey track, the chosen one raised.
struct Segmented: View {
    let options: [(value: String, label: String)]
    @Binding var selection: String
    var stretch = false

    var body: some View {
        HStack(spacing: 2) {
            ForEach(options, id: \.value) { o in
                let on = o.value == selection
                Button { selection = o.value } label: {
                    Text(o.label).font(.system(size: 14, weight: .medium)).lineLimit(1)
                        .foregroundStyle(on ? Look.text : Look.text2)
                        .padding(.horizontal, 14).padding(.vertical, 5)
                        .frame(maxWidth: stretch ? .infinity : nil)
                        .background(on ? Look.elevated : .clear, in: RoundedRectangle(cornerRadius: 8, style: .continuous))
                        .shadow(color: on ? .black.opacity(0.25) : .clear, radius: 1.5, y: 1)
                        .contentShape(Rectangle())
                }
                .buttonStyle(.plain)
            }
        }
        .padding(3)
        .background(Look.surface2, in: RoundedRectangle(cornerRadius: 10, style: .continuous))
        .fixedSize(horizontal: !stretch, vertical: true)
    }
}

extension Segmented {
    /// A segmented choice that runs `pick` when changed (for values kept elsewhere).
    init(_ options: [(String, String)], selected: String, stretch: Bool = false, pick: @escaping (String) -> Void) {
        self.options = options.map { (value: $0.0, label: $0.1) }
        self._selection = Binding(get: { selected }, set: pick)
        self.stretch = stretch
    }
}

// MARK: - Pages, groups, rows

/// A page as UUOB's always were: a large title, a grey line about it, the
/// content in a column no wider than 680 points. With `back`, a link above
/// the title goes back.
struct Page<Content: View, Trailing: View>: View {
    let title: String
    var lead: String = ""
    var back: (() -> Void)? = nil
    @ViewBuilder var trailing: Trailing
    @ViewBuilder var content: Content

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 0) {
                if let back {
                    Button(action: back) { HStack(spacing: 4) { Ico("back", size: 18); Text("Back") } }
                        .buttonStyle(UuobButton(.ghost)).padding(.leading, -18).padding(.bottom, 12)
                }
                HStack(alignment: .firstTextBaseline) {
                    Text(title).font(.system(size: 34, weight: .bold)).tracking(-0.85).foregroundStyle(Look.text)
                    Spacer()
                    trailing
                }
                if !lead.isEmpty {
                    Text(lead).font(.system(size: 17)).foregroundStyle(Look.text2).padding(.top, 10).frame(maxWidth: 560, alignment: .leading)
                }
                content.padding(.top, 14)
            }
            .frame(maxWidth: 680)
            .padding(.horizontal, 32).padding(.top, 56).padding(.bottom, 48)
            .frame(maxWidth: .infinity)
        }
        .background(Look.bg)
        .buttonStyle(UuobButton())
        .toggleStyle(UuobSwitch())
        .foregroundStyle(Look.text)
        .navigationBarBackButtonHidden(true)
    }
}

extension Page where Trailing == EmptyView {
    init(title: String, lead: String = "", back: (() -> Void)? = nil, @ViewBuilder content: () -> Content) {
        self.title = title
        self.lead = lead
        self.back = back
        self.trailing = EmptyView()
        self.content = content()
    }
}

/// `.section-label`: small capitals, grey.
struct SectionLabel: View {
    let text: String
    init(_ text: String) { self.text = text }
    var body: some View {
        Text(text.uppercased()).font(.system(size: 13, weight: .semibold)).tracking(0.5).foregroundStyle(Look.text3)
    }
}

/// `.group`: rows on one card with a hairline edge, a line between rows;
/// a label above it, a note under it. (Like Form's Section, which it replaces.)
struct Card<Content: View, Header: View, Footer: View>: View {
    @ViewBuilder var content: Content
    @ViewBuilder var header: Header
    @ViewBuilder var footer: Footer

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            header.padding(.leading, 4)
            _VariadicView.Tree(Rows()) { content }
                .background(Look.surface, in: RoundedRectangle(cornerRadius: 18, style: .continuous))
                .overlay(RoundedRectangle(cornerRadius: 18, style: .continuous).strokeBorder(Look.line))
            footer.font(.system(size: 13)).foregroundStyle(Look.text2).padding(.horizontal, 4).padding(.top, 2)
        }
        .padding(.top, 28)
    }

    /// Each child a row: 14 by 18 points of room, at least 56 high, a hairline above all but the first.
    struct Rows: _VariadicView_MultiViewRoot {
        func body(children: _VariadicView.Children) -> some View {
            VStack(spacing: 0) {
                ForEach(children) { child in
                    if child.id != children.first?.id { Rectangle().fill(Look.line).frame(height: 1) }
                    child.padding(.horizontal, 18).padding(.vertical, 14).frame(maxWidth: .infinity, minHeight: 56, alignment: .leading)
                }
            }
        }
    }
}

extension Card where Header == EmptyView, Footer == EmptyView {
    init(@ViewBuilder content: () -> Content) {
        self.content = content(); self.header = EmptyView(); self.footer = EmptyView()
    }
}

extension Card where Header == SectionLabel, Footer == EmptyView {
    init(_ title: String, @ViewBuilder content: () -> Content) {
        self.content = content(); self.header = SectionLabel(title); self.footer = EmptyView()
    }
}

extension Card where Footer == EmptyView {
    init(@ViewBuilder content: () -> Content, @ViewBuilder header: () -> Header) {
        self.content = content(); self.header = header(); self.footer = EmptyView()
    }
}


// MARK: - Rows like the event card's

/// A row as the event card draws them (and so the edit window and Settings):
/// a grey icon, then the content, at least 44 high, a hairline under it
/// unless it is the last. No boxes: the lines are enough.
struct IconRow<Content: View>: View {
    let icon: String?
    var last = false
    @ViewBuilder var content: Content
    init(_ icon: String?, last: Bool = false, @ViewBuilder content: () -> Content) {
        self.icon = icon; self.last = last; self.content = content()
    }
    var body: some View {
        HStack(spacing: 12) {
            if let icon { Ico(icon, size: 19, stroke: 1.8).foregroundStyle(Look.text2).frame(width: 20) } else { Color.clear.frame(width: 20, height: 1) }
            content
        }
        .font(.system(size: 15))
        .frame(maxWidth: .infinity, minHeight: 44, alignment: .leading)
        .padding(.vertical, 2)
        .overlay(alignment: .bottom) { if !last { Rectangle().fill(Look.line).frame(height: 1) } }
        .contentShape(Rectangle())
    }
}

/// A small grey heading over a few rows ("Views").
struct RowsLabel: View {
    let text: String
    init(_ text: String) { self.text = text }
    var body: some View {
        Text(text.uppercased()).font(.system(size: 12, weight: .semibold)).tracking(0.5).foregroundStyle(Look.text3)
            .padding(.top, 18).padding(.bottom, 2)
    }
}

/// A plain line of text that opens a menu (the card's alert line): no box.
struct PlainMenu: View {
    let label: String
    let items: [(String, () -> Void)]
    var color: Color = Look.text
    var body: some View {
        Menu {
            ForEach(Array(items.enumerated()), id: \.offset) { _, it in Button(it.0, action: it.1) }
        } label: {
            HStack(spacing: 5) {
                Text(label).lineLimit(1)
                Ico("chevron", size: 13).rotationEffect(.degrees(90)).foregroundStyle(Look.text2)
            }
            .foregroundStyle(color).contentShape(Rectangle())
        }
        .menuStyle(.button).buttonStyle(.plain).menuIndicator(.hidden).fixedSize()
    }
}

/// `.tag`: "This device", small and blue in a pill.
struct Tag: View {
    let text: String
    init(_ text: String) { self.text = text }
    var body: some View {
        Text(text).font(.system(size: 12, weight: .semibold)).foregroundStyle(Look.accent)
            .padding(.horizontal, 8).padding(.vertical, 2).background(Look.accentSoft, in: Capsule())
    }
}

// MARK: - Rows, fields, confirmations

/// A row: what it is on the left, a control on the right (Form's LabeledContent, outside a Form).
struct Trailing<Label: View, Content: View>: View {
    @ViewBuilder var content: Content
    @ViewBuilder var label: Label
    var body: some View {
        HStack(spacing: 14) {
            label
            Spacer(minLength: 0)
            content
        }
    }
}

extension Trailing where Label == Text {
    init(_ title: String, @ViewBuilder content: () -> Content) {
        self.content = content()
        self.label = Text(title).font(.system(size: 16, weight: .semibold))
    }
}

/// `.field`: 46 high, rounded, on grey; a blue ring while typing.
struct UuobField: TextFieldStyle {
    @FocusState private var focused: Bool
    func _body(configuration: TextField<Self._Label>) -> some View {
        configuration
            .textFieldStyle(.plain)
            .font(.system(size: 17))
            .focused($focused)
            .padding(.horizontal, 14)
            .frame(minHeight: 46)
            .background(focused ? Look.surface : Look.surface2, in: RoundedRectangle(cornerRadius: 12, style: .continuous))
            .overlay(RoundedRectangle(cornerRadius: 12, style: .continuous).strokeBorder(focused ? Look.accent : Look.line))
            .overlay(RoundedRectangle(cornerRadius: 15, style: .continuous).stroke(focused ? Look.accentSoft : .clear, lineWidth: 4).padding(-3))
    }
}

/// A question before something that can't be undone, as one of UUOB's sheets.
struct ConfirmSheet: View {
    @Environment(\.dismiss) private var dismiss
    let title: String
    let message: String
    let action: String
    let perform: () -> Void

    var body: some View {
        SheetFrame(title: title, lead: message) {
            EmptyView()
        } buttons: {
            Button("Cancel") { dismiss() }.keyboardShortcut(.cancelAction)
            Button(action, role: .destructive) {
                perform()
                dismiss()
            }
            .buttonStyle(UuobButton(.danger, large: true)).keyboardShortcut(.defaultAction)
        }
    }
}

/// A sheet asking for one line of text (a device's name).
struct TextSheet: View {
    @Environment(\.dismiss) private var dismiss
    let title: String
    @State var text: String
    let save: (String) -> Void

    var body: some View {
        SheetFrame(title: title) {
            TextField("Name", text: $text).textFieldStyle(UuobField()).onSubmit(done)
        } buttons: {
            Button("Cancel") { dismiss() }.keyboardShortcut(.cancelAction)
            Button("Save", action: done).buttonStyle(UuobButton(.primary, large: true)).keyboardShortcut(.defaultAction)
                .disabled(text.trimmingCharacters(in: .whitespaces).isEmpty)
        }
    }

    private func done() {
        guard !text.trimmingCharacters(in: .whitespaces).isEmpty else { return }
        save(text)
        dismiss()
    }
}

extension View {
    /// Asks `title` (with `message`) in a sheet; `perform` runs on `action`.
    func confirm(_ title: String, isPresented: Binding<Bool>, message: String, action: String, perform: @escaping () -> Void) -> some View {
        sheet(isPresented: isPresented) { ConfirmSheet(title: title, message: message, action: action, perform: perform) }
    }
}

// MARK: - What the apps add

/// The calendar colours every app shares (tokens.css `--c-*`), light and dark.
enum Palette {
    static let names = ["blue", "red", "orange", "yellow", "green", "teal", "purple", "pink"]
    /// The colour's light and dark values.
    static func hex(_ name: String) -> (light: UInt32, dark: UInt32) {
        switch name {
        case "red": (0xff3b30, 0xff453a)
        case "orange": (0xff9500, 0xff9f0a)
        case "yellow": (0xffcc00, 0xffd60a)
        case "green": (0x34c759, 0x30d158)
        case "teal": (0x30b0c7, 0x40c8e0)
        case "purple": (0xaf52de, 0xbf5af2)
        case "pink": (0xff2d55, 0xff375f)
        default: (0x007aff, 0x0a84ff)
        }
    }
    static func color(_ name: String) -> Color { let h = hex(name); return Color(light: h.light, dark: h.dark) }

    /// `color-mix(in srgb, <name>, <toward> t)`, light and dark: `toward` is (light, dark) too.
    static func mix(_ name: String, toward: (UInt32, UInt32), _ t: Double) -> Color {
        let h = hex(name)
        return Color(light: blend(h.light, toward.0, t), dark: blend(h.dark, toward.1, t))
    }

    /// The page and text colours, for `mix`.
    static let bg: (UInt32, UInt32) = (0xf5f5f7, 0x0b0b0d)
    static let text: (UInt32, UInt32) = (0x1d1d1f, 0xf5f5f7)
}

/// `a` with `t` of `b` mixed in.
func blend(_ a: UInt32, _ b: UInt32, _ t: Double) -> UInt32 {
    func ch(_ v: UInt32, _ s: UInt32) -> Double { Double((v >> s) & 0xff) }
    func one(_ s: UInt32) -> UInt32 { UInt32((ch(a, s) * (1 - t) + ch(b, s) * t).rounded()) << s }
    return one(16) | one(8) | one(0)
}

/// An icon on a round colour (a smart list, a list, a calendar).
struct Circled: View {
    let icon: String
    let color: Color
    var size: CGFloat = 30
    var body: some View {
        Circle().fill(color).frame(width: size, height: size)
            .overlay(Ico(icon, size: size * 0.55, stroke: 2.2).foregroundStyle(.white))
    }
}

/// `.check`: a round tick box; `ring` (a list's colour) rings it and fills it when on.
struct Check: View {
    let on: Bool
    var ring: Color? = nil
    var size: CGFloat = 22
    var body: some View {
        ZStack {
            Circle().strokeBorder(on ? (ring ?? Look.accent) : (ring ?? Look.text3), lineWidth: 1.75)
            if on {
                Circle().fill(ring ?? Look.accent)
                Ico("check", size: size * 0.64, stroke: 3).foregroundStyle(.white)
            }
        }
        .frame(width: size, height: size)
        .animation(.easeOut(duration: 0.22), value: on)
    }
}

/// A button with just an icon (`.icon-btn`).
struct IconButton: View {
    let icon: String
    var size: CGFloat = 20
    var help = ""
    let action: () -> Void
    init(_ icon: String, size: CGFloat = 20, help: String = "", action: @escaping () -> Void) {
        self.icon = icon
        self.size = size
        self.help = help
        self.action = action
    }
    var body: some View {
        Button(action: action) { Ico(icon, size: size) }.buttonStyle(IconButtonStyle()).help(help)
    }
}

/// A pill button with an icon before its text (`.btn` with an Icon).
struct IconLabel: View {
    let icon: String
    let text: String
    init(_ icon: String, _ text: String) {
        self.icon = icon
        self.text = text
    }
    var body: some View { HStack(spacing: 8) { Ico(icon, size: 18, stroke: 2); Text(text) } }
}

/// `label.stack`: a small grey label above its field.
struct Stack<Content: View>: View {
    let label: String
    @ViewBuilder var content: Content
    init(_ label: String, @ViewBuilder content: () -> Content) {
        self.label = label
        self.content = content()
    }
    var body: some View {
        VStack(alignment: .leading, spacing: 6) {
            Text(label).font(.system(size: 13, weight: .medium)).foregroundStyle(Look.text2)
            content
        }
    }
}

/// `.large-title`.
struct LargeTitle: View {
    let text: String
    var color: Color = Look.text
    init(_ text: String, color: Color = Look.text) {
        self.text = text
        self.color = color
    }
    var body: some View { Text(text).font(.system(size: 34, weight: .bold)).tracking(-0.85).foregroundStyle(color).lineLimit(1) }
}

/// `.sheet`: a title, an optional line, the content, then the buttons side by side.
struct SheetFrame<Content: View, Buttons: View>: View {
    let title: String
    var lead: String? = nil
    var center = false
    @ViewBuilder var content: Content
    @ViewBuilder var buttons: Buttons

    var body: some View {
        VStack(alignment: center ? .center : .leading, spacing: 14) {
            if !title.isEmpty {
                Text(title).font(.system(size: 22, weight: .semibold)).tracking(-0.44).multilineTextAlignment(center ? .center : .leading)
            }
            if let lead { Text(lead).font(.system(size: 17)).foregroundStyle(Look.text2).multilineTextAlignment(center ? .center : .leading).fixedSize(horizontal: false, vertical: true) }
            content
            HStack(spacing: 10) {
                buttons
            }
            .environment(\.fillButtons, true)
            .padding(.top, 8)
        }
        .padding(28)
        .frame(width: 460)
        .background(Look.elevated)
        .foregroundStyle(Look.text)
        .buttonStyle(UuobButton(large: true))
        .toggleStyle(UuobSwitch())
        .textFieldStyle(UuobField())
    }
}

/// A sheet asking which of a few answers; each button: (label, kind, what it does).
struct ChoiceSheet: View {
    @Environment(\.dismiss) private var dismiss
    let title: String
    let message: String
    let choices: [(String, Kind, () -> Void)]
    var cancel = true

    var body: some View {
        VStack(spacing: 0) {
            SheetFrame(title: title, lead: message) {
                EmptyView()
            } buttons: {
                ForEach(Array(choices.enumerated()), id: \.offset) { _, c in
                    Button(c.0) {
                        c.2()
                        dismiss()
                    }
                    .buttonStyle(UuobButton(c.1, large: true))
                }
            }
            if cancel {
                Button("Cancel") { dismiss() }.buttonStyle(UuobButton(.ghost)).keyboardShortcut(.cancelAction)
                    .padding(.top, -14).padding(.bottom, 20)
            }
        }
        .background(Look.elevated)
    }
}

/// The empty look of a list or a day (`.empty`): a big glyph, a line, a caption.
struct Empty: View {
    let glyph: String
    let headline: String
    let caption: String
    var body: some View {
        VStack(spacing: 6) {
            Text(glyph).font(.system(size: 44))
            Text(headline).font(.system(size: 17, weight: .semibold)).foregroundStyle(Look.text2)
            Text(caption).font(.system(size: 13)).foregroundStyle(Look.text2).multilineTextAlignment(.center)
        }
        .frame(maxWidth: .infinity).padding(.top, 60)
    }
}
