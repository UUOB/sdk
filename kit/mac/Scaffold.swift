// The frames every app is built in, taken from Calendar's (docs/design.md):
// the sidebar that folds to a rail, the toolbar, the detail card and the
// editor, the empty state, list rows and the quiet tints. Each app puts its
// own rows inside; the frame, spacing and behaviour are the same everywhere.

import SwiftUI

// MARK: - Sidebar

/// Posted by the Toggle Sidebar command (⌃⌘S); the sidebar on screen folds or unfolds.
extension Notification.Name {
    static let uuobToggleSidebar = Notification.Name("uuob.toggleSidebar")
}

/// The sidebar and the rest of the window. Its edge drags, as Finder's:
/// - wide: 180 to 400 points (240 at first), names and sections (`sidebar`);
/// - rail: icons only, names as hints (`rail`), as wide as the window's
///   buttons so they never stick out past it; dragged narrower than the wide
///   sidebar, it snaps to this;
/// - hidden: dragged on past half the rail, nothing; the toggle then sits
///   beside the window's buttons, and the left edge drags it back out.
/// The toggle at the bottom (and ⌃⌘S) folds wide to the rail and back, and
/// brings a hidden sidebar back as it was. All of it is kept per device.
/// Below 600 points the rail shows by itself (or nothing, when hidden), and
/// the toggle opens the wide sidebar over the content.
struct SidebarScaffold<Side: View, Rail: View, Content: View, Footer: View, RailFooter: View>: View {
    @ViewBuilder var sidebar: () -> Side
    @ViewBuilder var rail: () -> Rail
    @ViewBuilder var content: () -> Content
    /// The expanded sidebar's foot, on the toggle's line and left of it, with
    /// all the width the toggle leaves: a `SidebarRow` (Settings) lines up with
    /// the rows above, a note sits under their icons.
    @ViewBuilder var footer: () -> Footer
    /// The rail's foot, above the toggle: a `RailButton`, a small mark.
    @ViewBuilder var railFooter: () -> RailFooter
    /// Folded to the rail (kept between launches, per device).
    @AppStorage("sidebar.folded") private var folded = false
    /// Dragged away to nothing; folded keeps what the toggle brings back.
    @AppStorage("sidebar.hidden") private var hidden = false
    /// The wide sidebar's width, as last dragged.
    @AppStorage("sidebar.width") private var wideWidth: Double = 240
    /// The shape while the edge is being dragged; kept only when it's let go.
    @State private var live: Fold?
    /// The expanded sidebar opened over the content in a narrow window.
    @State private var peek = false
    /// The window's buttons: where the zoom button ends, and their middle (from the top).
    @State private var buttons = CGPoint(x: 70, y: 14)

    private enum Fold: Equatable { case hidden, rail, wide(CGFloat) }
    private static var minWide: CGFloat { 180 }
    private static var maxWide: CGFloat { 400 }
    /// At least 64 points, and past the zoom button.
    private var railWidth: CGFloat { max(64, ceil(buttons.x) + 10) }

    var body: some View {
        GeometryReader { geo in
            let narrow = geo.size.width < 600
            let shape = live ?? settled(narrow: narrow, window: geo.size.width)
            let width = self.width(shape)
            ZStack(alignment: .topLeading) {
                HStack(spacing: 0) {
                    if shape != .hidden {
                        column(shape)
                        Rectangle().fill(Look.line).frame(width: 1)
                    }
                    content().frame(maxWidth: .infinity, maxHeight: .infinity)
                        .environment(\.uuobCompact, narrow)
                }
                // Hidden: the toggle beside the window's buttons.
                if shape == .hidden && !peek {
                    toggleButton(expanded: false)
                        .padding(.leading, buttons.x + 4).padding(.top, max(0, buttons.y - 18))
                }
                // The edge to drag, one view whatever the shape, so a drag that
                // snaps to the rail or to nothing goes on and can come back.
                Color.clear.frame(width: 8).frame(maxHeight: .infinity)
                    .background(ResizeCursor())
                    .contentShape(Rectangle())
                    .offset(x: max(0, width - 4))
                    .gesture(DragGesture(minimumDistance: 1, coordinateSpace: .named(Self.space))
                        .onChanged { live = self.shape(at: $0.location.x, narrow: narrow, window: geo.size.width) }
                        .onEnded { _ in keep(narrow: narrow) })
                if narrow && peek {
                    Color.black.opacity(0.25).ignoresSafeArea().onTapGesture { withAnimation(.easeOut(duration: 0.2)) { peek = false } }
                    column(.wide(240)).shadow(color: .black.opacity(0.25), radius: 18)
                        .transition(.move(edge: .leading))
                        // A row that goes somewhere closes it (`uuobClosePeek`).
                        .environment(\.uuobClosePeek) { withAnimation(.easeOut(duration: 0.2)) { peek = false } }
                }
            }
            .coordinateSpace(name: Self.space)
            .background(WindowButtons { buttons = $0 })
            .onChange(of: narrow) { if !narrow { peek = false } }
            .onReceive(NotificationCenter.default.publisher(for: .uuobToggleSidebar)) { _ in toggle(narrow: narrow) }
        }
    }

    private static var space: String { "uuob.sidebar" }

    /// What's kept: hidden, the rail, or the wide sidebar (never wider than
    /// leaves the content 360 points); only the rail or nothing below 600.
    private func settled(narrow: Bool, window: CGFloat) -> Fold {
        if hidden { return .hidden }
        if narrow || folded { return .rail }
        return .wide(clampWide(CGFloat(wideWidth), window: window))
    }

    private func clampWide(_ w: CGFloat, window: CGFloat) -> CGFloat {
        min(max(w, Self.minWide), max(Self.minWide, min(Self.maxWide, window - 360)))
    }

    /// The pointer's distance from the window's left edge, as a shape: past
    /// half the rail it's nothing, short of halfway to the wide sidebar the rail.
    private func shape(at x: CGFloat, narrow: Bool, window: CGFloat) -> Fold {
        if x < railWidth / 2 { return .hidden }
        if narrow || x < (railWidth + Self.minWide) / 2 { return .rail }
        return .wide(clampWide(x, window: window))
    }

    private func width(_ shape: Fold) -> CGFloat {
        switch shape {
        case .hidden: 0
        case .rail: railWidth
        case .wide(let w): w
        }
    }

    /// Lets go of a drag: the shape is kept (a narrow window's rail leaves
    /// the wide window's choice as it was).
    private func keep(narrow: Bool) {
        guard let shape = live else { return }
        switch shape {
        case .hidden: hidden = true
        case .rail: hidden = false; if !narrow { folded = true }
        case .wide(let w): hidden = false; folded = false; wideWidth = Double(w)
        }
        live = nil
    }

    private func toggle(narrow: Bool) {
        withAnimation(.easeOut(duration: 0.2)) {
            if narrow { peek.toggle() } else if hidden { hidden = false } else { folded.toggle() }
        }
    }

    private func toggleButton(expanded: Bool) -> some View {
        IconButton("sidebar", size: 18, help: expanded ? "Hide sidebar (⌃⌘S)" : "Show sidebar (⌃⌘S)") {
            NotificationCenter.default.post(name: .uuobToggleSidebar, object: nil)
        }
    }

    private func column(_ shape: Fold) -> some View {
        let expanded = shape != .rail
        return VStack(spacing: 0) {
            if expanded { sidebar().frame(maxHeight: .infinity, alignment: .top) } else { rail().frame(maxHeight: .infinity, alignment: .top) }
            if expanded {
                HStack(spacing: 8) {
                    footer().frame(maxWidth: .infinity, alignment: .leading)
                    toggleButton(expanded: true)
                }
                // At the right even when an app has no footer.
                .frame(maxWidth: .infinity, alignment: .trailing)
                .padding(.horizontal, 12).padding(.bottom, 12)
            } else {
                VStack(spacing: 6) {
                    railFooter()
                    toggleButton(expanded: false)
                }
                .frame(maxWidth: .infinity).padding(.bottom, 12)
            }
        }
        .frame(width: width(shape))
        .frame(maxHeight: .infinity)
        .background(Look.rail)
    }
}

/// The left-right arrows over the sidebar's edge (it takes no clicks itself).
private struct ResizeCursor: NSViewRepresentable {
    final class Area: NSView {
        override func resetCursorRects() { addCursorRect(bounds, cursor: .resizeLeftRight) }
        override func hitTest(_ point: NSPoint) -> NSView? { nil }
    }
    func makeNSView(context: Context) -> NSView { Area() }
    func updateNSView(_ nsView: NSView, context: Context) { nsView.window?.invalidateCursorRects(for: nsView) }
}

/// Reports where the window's buttons (close, minimise, zoom) are: the zoom
/// button's right edge from the window's left, and their middle from its top.
private struct WindowButtons: NSViewRepresentable {
    let report: (CGPoint) -> Void
    final class Probe: NSView {
        var report: (CGPoint) -> Void = { _ in }
        override func viewDidMoveToWindow() {
            super.viewDidMoveToWindow()
            guard let window, let zoom = window.standardWindowButton(.zoomButton), let bar = zoom.superview else { return }
            let r = bar.convert(zoom.frame, to: nil)
            let top = window.contentView?.bounds.height ?? window.frame.height
            let found = CGPoint(x: r.maxX, y: top - r.midY)
            DispatchQueue.main.async { [report] in report(found) }
        }
        override func hitTest(_ point: NSPoint) -> NSView? { nil }
    }
    func makeNSView(context: Context) -> Probe { let p = Probe(); p.report = report; return p }
    func updateNSView(_ nsView: Probe, context: Context) { nsView.report = report }
}

extension SidebarScaffold where Footer == EmptyView, RailFooter == EmptyView {
    init(@ViewBuilder sidebar: @escaping () -> Side, @ViewBuilder rail: @escaping () -> Rail, @ViewBuilder content: @escaping () -> Content) {
        self.init(sidebar: sidebar, rail: rail, content: content, footer: { EmptyView() }, railFooter: { EmptyView() })
    }
}

/// Closes the sidebar shown over a narrow window; nothing elsewhere. A
/// sidebar row that goes to another place calls it after going there.
private struct ClosePeek: EnvironmentKey { static let defaultValue: () -> Void = {} }
/// The colour of what a page shows (a list): its +, Save, ticks and blue
/// text take it instead of the accent (`.uuobTint(_:)`).
private struct PageTint: EnvironmentKey { static let defaultValue: Color? = nil }
extension EnvironmentValues {
    var uuobClosePeek: () -> Void {
        get { self[ClosePeek.self] }
        set { self[ClosePeek.self] = newValue }
    }
    var uuobTint: Color? {
        get { self[PageTint.self] }
        set { self[PageTint.self] = newValue }
    }
}

extension View {
    /// Gives this page (or sheet) the colour of what it shows; nil is the accent.
    func uuobTint(_ color: Color?) -> some View { environment(\.uuobTint, color) }
}

/// A row of the expanded sidebar: an icon, a colour dot or a check, the name,
/// and something small on the right (a count, a mark). The chosen one is raised.
struct SidebarRow<Leading: View, Trailing: View>: View {
    let title: String
    var selected = false
    @ViewBuilder var leading: () -> Leading
    @ViewBuilder var trailing: () -> Trailing
    let action: () -> Void
    @State private var hover = false

    var body: some View {
        Button(action: action) {
            HStack(spacing: 10) {
                leading().frame(width: 22)
                Text(title).font(.system(size: 15, weight: selected ? .semibold : .medium)).lineLimit(1).truncationMode(.tail)
                Spacer(minLength: 4)
                trailing().font(.system(size: 13)).foregroundStyle(Look.text2)
            }
            .padding(.horizontal, 10).frame(height: 36)
            .background(selected ? Look.active : hover ? Look.hover : .clear, in: RoundedRectangle(cornerRadius: 10, style: .continuous))
            .contentShape(Rectangle())
        }
        .buttonStyle(.plain)
        .onHover { hover = $0 }
    }
}

extension SidebarRow where Trailing == EmptyView {
    init(title: String, selected: Bool = false, @ViewBuilder leading: @escaping () -> Leading, action: @escaping () -> Void) {
        self.init(title: title, selected: selected, leading: leading, trailing: { EmptyView() }, action: action)
    }
}

/// An icon of the rail: 40 points in a soft tile when chosen, the name as a tooltip.
struct RailButton<Mark: View>: View {
    let help: String
    var selected = false
    @ViewBuilder var mark: () -> Mark
    let action: () -> Void
    @State private var hover = false

    var body: some View {
        Button(action: action) {
            mark()
                .frame(width: 40, height: 40)
                .background(selected ? Look.active : hover ? Look.hover : .clear, in: RoundedRectangle(cornerRadius: 12, style: .continuous))
                .contentShape(Rectangle())
        }
        .buttonStyle(.plain)
        .foregroundStyle(selected ? Look.text : Look.text2)
        .uuobHint(help)
        .accessibilityLabel(help)
        .onHover { hover = $0 }
    }
}

extension RailButton where Mark == AnyView {
    /// A rail button with an icon.
    init(_ icon: String, help: String, selected: Bool = false, action: @escaping () -> Void) {
        self.init(help: help, selected: selected, mark: { AnyView(Ico(icon, size: 20)) }, action: action)
    }
}

// MARK: - Toolbar

/// The top of a page: the title (with a grey count or year after it), then
/// the page's controls, then the round + button. Never squeezed: the title
/// shrinks a little, the controls keep their size.
struct Toolbar<Controls: View>: View {
    let title: Text
    var compact = false
    var add: (() -> Void)? = nil
    var addHelp = "New"
    @ViewBuilder var controls: () -> Controls

    var body: some View {
        HStack(spacing: 14) {
            title.font(.system(size: compact ? 26 : 34)).tracking(-0.85).lineLimit(1).minimumScaleFactor(0.6).layoutPriority(1)
            Spacer(minLength: 0)
            controls().fixedSize()
            if let add { PlusButton(help: addHelp, action: add) }
        }
        .padding(.bottom, compact ? 12 : 18)
    }
}

extension Toolbar where Controls == EmptyView {
    init(title: Text, compact: Bool = false, add: (() -> Void)? = nil, addHelp: String = "New") {
        self.init(title: title, compact: compact, add: add, addHelp: addHelp, controls: { EmptyView() })
    }
}

/// A big title with a grey part after it ("November 2026", "Photos 12").
func titleText(_ main: String, _ grey: String = "") -> Text {
    Text(main).fontWeight(.bold) + Text(grey.isEmpty ? "" : " \(grey)").fontWeight(.regular).foregroundStyle(Look.text2)
}

/// The round blue + (New event, New task, Add photos).
struct PlusButton: View {
    var help = "New"
    let action: () -> Void
    @Environment(\.uuobTint) private var tint
    var body: some View {
        Button(action: action) {
            Ico("plus", size: 20, stroke: 2.2).foregroundStyle(.white).frame(width: 36, height: 36).background(tint ?? Look.accent, in: Circle())
        }
        .buttonStyle(.plain).uuobHint(help).fixedSize()
    }
}

// MARK: - Detail card and editor

/// The detail card (Calendar's event card): a title with its actions and ✕,
/// then whatever the app shows (lines, `IconRow`s), then an optional footer.
/// 460 wide, as tall as it needs up to 640, Esc closes.
struct DetailSheet<Actions: View, Content: View, Footer: View>: View {
    let title: String
    var onClose: () -> Void
    @ViewBuilder var actions: () -> Actions
    @ViewBuilder var content: () -> Content
    @ViewBuilder var footer: () -> Footer

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 0) {
                HStack(alignment: .top, spacing: 4) {
                    Text(title).font(.system(size: 22, weight: .semibold)).tracking(-0.44)
                        .frame(maxWidth: .infinity, alignment: .leading).padding(.top, 4).textSelection(.enabled)
                    actions()
                    IconButton("close", size: 19, help: "Close (Esc)", action: onClose).keyboardShortcut(.cancelAction)
                }
                content()
                footer()
            }
            .padding(22)
        }
        .frame(width: sheetWidth(460)).frame(minHeight: 160, maxHeight: 640)
        .fixedSize(horizontal: false, vertical: true)
        .background(Look.elevated)
        .foregroundStyle(Look.text)
        // Read-only: a click outside closes it, as a menu does.
        .uuobClosesOnOutsideClick(onClose)
    }
}

extension DetailSheet where Footer == EmptyView {
    init(title: String, onClose: @escaping () -> Void, @ViewBuilder actions: @escaping () -> Actions, @ViewBuilder content: @escaping () -> Content) {
        self.init(title: title, onClose: onClose, actions: actions, content: content, footer: { EmptyView() })
    }
}

/// The editor (Calendar's event editor): the title big and editable at the
/// top with ✕, then rows with icons, then Delete on the left and Save on the
/// right. Esc cancels, ↩ in the title or ⌘↩ anywhere saves.
struct EditSheet<Under: View, Content: View>: View {
    var placeholder = "Title"
    @Binding var title: String
    var isNew = false
    var canSave = true
    var saveLabel: String? = nil
    var onCancel: () -> Void
    var onSave: () -> Void
    var onDelete: (() -> Void)? = nil
    /// A line under the title (Calendar: the calendar's dot and name).
    @ViewBuilder var under: () -> Under
    @ViewBuilder var content: () -> Content

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 0) {
                HStack(alignment: .top, spacing: 4) {
                    TextField(placeholder, text: $title).textFieldStyle(.plain)
                        .font(.system(size: 22, weight: .semibold)).tracking(-0.44)
                        .onSubmit { if canSave { onSave() } }
                    IconButton("close", size: 19, help: "Cancel (Esc)", action: onCancel).keyboardShortcut(.cancelAction)
                }
                .padding(.top, 2)
                under().padding(.top, 6).padding(.bottom, 8)
                content()
                HStack(spacing: 10) {
                    if let onDelete, !isNew {
                        Button("Delete", role: .destructive, action: onDelete).buttonStyle(UuobButton(.danger, small: true))
                    }
                    Spacer()
                    if isNew { Button("Cancel", action: onCancel).buttonStyle(UuobButton(small: true)) }
                    Button(saveLabel ?? (isNew ? "Add" : "Save"), action: onSave)
                        .buttonStyle(UuobButton(.primary, small: true)).disabled(!canSave)
                        .keyboardShortcut(.return, modifiers: .command)
                }
                .padding(.top, 18)
            }
            .padding(22)
        }
        .frame(width: sheetWidth(460)).frame(maxHeight: 640)
        .fixedSize(horizontal: false, vertical: true)
        .background(Look.elevated)
        .foregroundStyle(Look.text)
        .toggleStyle(UuobSwitch())
    }
}

extension EditSheet where Under == EmptyView {
    init(placeholder: String = "Title", title: Binding<String>, isNew: Bool = false, canSave: Bool = true, saveLabel: String? = nil,
         onCancel: @escaping () -> Void, onSave: @escaping () -> Void, onDelete: (() -> Void)? = nil, @ViewBuilder content: @escaping () -> Content) {
        self.init(placeholder: placeholder, title: title, isNew: isNew, canSave: canSave, saveLabel: saveLabel,
                  onCancel: onCancel, onSave: onSave, onDelete: onDelete, under: { EmptyView() }, content: content)
    }
}

// MARK: - Empty state and list rows

/// An empty list, day or library: an icon in a soft tile, a line, and maybe
/// one button. No paragraphs of explanation.
struct EmptyState: View {
    let icon: String
    let headline: String
    var color: Color = Look.text3
    var action: (String, () -> Void)? = nil

    var body: some View {
        VStack(spacing: 10) {
            Ico(icon, size: 28).foregroundStyle(color)
                .frame(width: 60, height: 60).background(Look.surface2, in: RoundedRectangle(cornerRadius: 18, style: .continuous))
            Text(headline).font(.system(size: 17, weight: .semibold)).foregroundStyle(Look.text2)
            if let action {
                Button(action.0, action: action.1).buttonStyle(UuobButton(.primary)).padding(.top, 8)
            }
        }
        .frame(maxWidth: .infinity).padding(.vertical, 56)
    }
}

/// A row of a list (a task, a calendar, an album): something on the left
/// (a check, a dot, an icon), a title with an optional grey line under it,
/// something on the right, a hairline under it. Raised while hovered.
struct ItemRow<Leading: View, Trailing: View>: View {
    let title: String
    var detail: String = ""
    var done = false
    var last = false
    @ViewBuilder var leading: () -> Leading
    @ViewBuilder var trailing: () -> Trailing
    @State private var hover = false

    var body: some View {
        HStack(alignment: .firstTextBaseline, spacing: 12) {
            leading()
            VStack(alignment: .leading, spacing: 2) {
                Text(title).font(.system(size: 16)).strikethrough(done).foregroundStyle(done ? Look.text3 : Look.text).lineLimit(2)
                if !detail.isEmpty { Text(detail).font(.system(size: 13)).foregroundStyle(Look.text2).lineLimit(1) }
            }
            Spacer(minLength: 6)
            trailing()
        }
        .padding(.horizontal, 10).padding(.vertical, 10)
        .background(hover ? Look.hover : .clear, in: RoundedRectangle(cornerRadius: 10, style: .continuous))
        .overlay(alignment: .bottom) { if !last { Rectangle().fill(Look.line).frame(height: 1).padding(.horizontal, 10) } }
        .contentShape(Rectangle())
        .onHover { hover = $0 }
    }
}

// MARK: - Tints

/// What kind of item something is, for a quiet shade of its colour.
struct TintKind: OptionSet {
    let rawValue: Int
    static let repeating = TintKind(rawValue: 1)
    static let joinable = TintKind(rawValue: 2)
    static let allDay = TintKind(rawValue: 4)
    static let past = TintKind(rawValue: 8)
    static let declined = TintKind(rawValue: 16)
    static let tentative = TintKind(rawValue: 32)

    init(rawValue: Int) { self.rawValue = rawValue }
}

/// The shades of one colour: how much of it fills the item, how strong its
/// edge is, how faded the whole item is. Subtle on purpose.
struct Tint {
    let fill: Double
    let edge: Double
    let fade: Double
}

/// The tint for a kind: a meeting to join a little deeper, a repeating one
/// lighter with a softer edge, all-day a touch lighter, past and declined faded.
func tint(_ kind: TintKind) -> Tint {
    var fill = 0.24
    if kind.contains(.allDay) { fill = 0.22 }
    if kind.contains(.joinable) { fill += 0.08 } else if kind.contains(.repeating) { fill -= 0.09 }
    let edge = kind.contains(.repeating) ? 0.55 : 1
    let fade = kind.contains(.declined) ? 0.45 : kind.contains(.past) ? 0.6 : 1
    return Tint(fill: fill, edge: edge, fade: fade)
}

/// The fill for an item of colour `base` (a palette name) and this kind, solid over the page.
func tint(_ base: String, _ kind: TintKind) -> Color {
    Palette.mix(base, toward: Palette.bg, 1 - tint(kind).fill)
}
