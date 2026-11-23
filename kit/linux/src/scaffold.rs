//! The frames every app is built in, taken from Calendar's (docs/design.md):
//! the sidebar that folds to a rail, the toolbar, the detail card and the
//! editor, the empty state, list rows, the quiet tints, and the keys every
//! app answers to. Each app puts its own rows inside.

use std::cell::Cell;
use std::rc::Rc;

use relm4::adw::prelude::*;
use relm4::{adw, gtk, gtk::glib};
use serde_json::Value;

use crate::{btn, icon, icon_btn};

pub(crate) const CSS: &str = "
.sidebar-col { background: @uuob_rail; }
.sidebar-col.expanded { min-width: 240px; }
.sidebar-col.rail { min-width: 64px; }
.sidebar-peek { box-shadow: 0 0 24px rgba(0,0,0,0.3); }
button.sidebar-row { min-height: 36px; padding: 0 10px; border-radius: 10px; background: none; box-shadow: none; }
button.sidebar-row:hover { background: alpha(@window_fg_color, 0.06); }
button.sidebar-row.selected { background: alpha(@window_fg_color, 0.1); }
button.sidebar-row .sidebar-title { font-size: 15px; font-weight: 500; }
button.sidebar-row.selected .sidebar-title { font-weight: 600; }
button.rail-btn { min-width: 40px; min-height: 40px; padding: 0; border-radius: 12px; background: none; box-shadow: none; color: @uuob_text2; }
button.rail-btn:hover { background: alpha(@window_fg_color, 0.06); }
button.rail-btn.selected { background: alpha(@window_fg_color, 0.1); color: @window_fg_color; }
.toolbar-title { font-size: 34px; font-weight: 700; letter-spacing: -0.85px; }
.toolbar-title.compact { font-size: 26px; }
button.plus-btn { min-width: 36px; min-height: 36px; padding: 0; border-radius: 999px; background: @uuob_accent; color: white; }
.sheet-title { font-size: 22px; font-weight: 600; letter-spacing: -0.4px; }
entry.sheet-title-edit { font-size: 22px; font-weight: 600; background: none; border: none; box-shadow: none; outline: none; padding: 0; min-height: 40px; }
.empty-tile { min-width: 60px; min-height: 60px; border-radius: 18px; background: @uuob_surface2; }
.empty-headline { font-size: 17px; font-weight: 600; color: @uuob_text2; }
.item-row { padding: 10px; border-radius: 10px; border-bottom: 1px solid @uuob_line; }
.item-row.last { border-bottom: none; }
.item-row:hover { background: alpha(@window_fg_color, 0.04); }
.item-title { font-size: 16px; }
.item-title.done { color: @uuob_text3; text-decoration: line-through; }
.item-detail { font-size: 13px; color: @uuob_text2; }
.key-cap { font-size: 14px; font-weight: 500; color: @uuob_text2; background: @uuob_surface2; border-radius: 6px; padding: 2px 8px; }
";

// ---- Sidebar ----------------------------------------------------------------

/// The sidebar and the rest of the window. Three states:
/// - expanded: 240 wide, names and sections (`sidebar`);
/// - rail: 64 wide, icons only (`rail`); by itself below 600 wide, or folded
///   with the toggle at the bottom (remembered in the app's config folder);
/// - in a narrow window the toggle opens the expanded sidebar over the content.
pub struct SidebarScaffold {
    root: gtk::Overlay,
    toggle: Rc<dyn Fn()>,
}

impl SidebarScaffold {
    /// `app`: the app's id, for where the folded state is kept.
    pub fn new(app: &str, sidebar: &impl IsA<gtk::Widget>, rail: &impl IsA<gtk::Widget>, content: &impl IsA<gtk::Widget>) -> SidebarScaffold {
        let file = glib::user_config_dir().join(format!("app.uuob.{app}")).join("sidebar-folded");
        let folded = Rc::new(Cell::new(file.exists()));
        let narrow = Rc::new(Cell::new(false));
        let peek = Rc::new(Cell::new(false));

        let column = gtk::Box::builder().orientation(gtk::Orientation::Vertical).hexpand(false).css_classes(["sidebar-col", "expanded"]).build();
        let stack = gtk::Stack::builder().vexpand(true).hhomogeneous(false).transition_type(gtk::StackTransitionType::Crossfade).transition_duration(120).build();
        // `External`: the sidebar is exactly 240 (the rail 64), whatever its rows would like.
        let side_scroll = gtk::ScrolledWindow::builder().hscrollbar_policy(gtk::PolicyType::External).width_request(240).child(sidebar).build();
        let rail_scroll = gtk::ScrolledWindow::builder().hscrollbar_policy(gtk::PolicyType::External).width_request(64).child(rail).build();
        stack.add_named(&side_scroll, Some("expanded"));
        stack.add_named(&rail_scroll, Some("rail"));
        column.append(&stack);
        let toggle_btn = icon_btn("sidebar", 18, "Toggle sidebar (Ctrl+Alt+S)");
        toggle_btn.set_margin_bottom(12);
        column.append(&toggle_btn);

        let row = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        row.append(&column);
        row.append(&gtk::Separator::new(gtk::Orientation::Vertical));
        // The content can be narrower than it would like, so the window can always
        // shrink to where the rail takes over (it scrolls sideways rather than holding the window wide).
        content.as_ref().set_hexpand(true);
        let holder = gtk::ScrolledWindow::builder().hscrollbar_policy(gtk::PolicyType::External).vscrollbar_policy(gtk::PolicyType::Never).hexpand(true).child(content).build();
        row.append(&holder);
        let root = gtk::Overlay::builder().child(&row).build();

        // The expanded sidebar over the content, in a narrow window.
        let dim = gtk::Button::builder().css_classes(["flat"]).hexpand(true).vexpand(true).build();
        dim.set_opacity(0.0);
        let peek_col = gtk::Box::builder().orientation(gtk::Orientation::Vertical).halign(gtk::Align::Start).css_classes(["sidebar-col", "expanded", "sidebar-peek"]).build();
        root.add_overlay(&dim);
        root.add_overlay(&peek_col);
        dim.set_visible(false);
        peek_col.set_visible(false);

        let apply: Rc<dyn Fn()> = {
            let (column, stack, toggle_btn, side_scroll, peek_col, dim) = (column.clone(), stack.clone(), toggle_btn.clone(), side_scroll.clone(), peek_col.clone(), dim.clone());
            let (folded, narrow, peek) = (folded.clone(), narrow.clone(), peek.clone());
            Rc::new(move || {
                let expanded = !narrow.get() && !folded.get();
                stack.set_visible_child_name(if expanded { "expanded" } else { "rail" });
                column.set_css_classes(&["sidebar-col", if expanded { "expanded" } else { "rail" }]);
                column.set_width_request(if expanded { 240 } else { 64 });
                toggle_btn.set_halign(if expanded { gtk::Align::End } else { gtk::Align::Center });
                toggle_btn.set_margin_end(if expanded { 12 } else { 0 });
                let show_peek = narrow.get() && peek.get();
                // The one sidebar moves into the overlay while it is shown there.
                if show_peek && side_scroll.parent().is_some_and(|p| p != peek_col.clone().upcast::<gtk::Widget>()) {
                    stack.remove(&side_scroll);
                    peek_col.append(&side_scroll);
                    side_scroll.set_vexpand(true);
                } else if !show_peek && side_scroll.parent().is_some_and(|p| p == peek_col.clone().upcast::<gtk::Widget>()) {
                    peek_col.remove(&side_scroll);
                    stack.add_named(&side_scroll, Some("expanded"));
                    stack.set_visible_child_name(if expanded { "expanded" } else { "rail" });
                }
                peek_col.set_visible(show_peek);
                dim.set_visible(show_peek);
            })
        };
        let toggle: Rc<dyn Fn()> = {
            let (folded, narrow, peek, apply, file) = (folded.clone(), narrow.clone(), peek.clone(), apply.clone(), file.clone());
            Rc::new(move || {
                if narrow.get() {
                    peek.set(!peek.get());
                } else {
                    folded.set(!folded.get());
                    if folded.get() {
                        let _ = std::fs::create_dir_all(file.parent().unwrap_or(std::path::Path::new(".")));
                        let _ = std::fs::write(&file, b"1");
                    } else {
                        let _ = std::fs::remove_file(&file);
                    }
                }
                apply();
            })
        };
        {
            let t = toggle.clone();
            toggle_btn.connect_clicked(move |_| t());
            let (peek, apply) = (peek.clone(), apply.clone());
            dim.connect_clicked(move |_| {
                peek.set(false);
                apply();
            });
        }
        // Below 600 wide the rail shows by itself.
        {
            let (narrow, peek, apply) = (narrow.clone(), peek.clone(), apply.clone());
            root.add_tick_callback(move |w, _| {
                let n = w.width() > 0 && w.width() < 600;
                if n != narrow.get() {
                    narrow.set(n);
                    if !n {
                        peek.set(false);
                    }
                    apply();
                }
                glib::ControlFlow::Continue
            });
        }
        apply();
        SidebarScaffold { root, toggle }
    }

    pub fn widget(&self) -> gtk::Widget {
        self.root.clone().upcast()
    }

    /// Folds or unfolds the sidebar (the Toggle Sidebar shortcut).
    pub fn toggle(&self) {
        (self.toggle)()
    }

    /// The same, for a shortcut or a button elsewhere.
    pub fn toggler(&self) -> Rc<dyn Fn()> {
        self.toggle.clone()
    }
}

/// A row of the expanded sidebar: `leading` (an icon, a dot, a check), the
/// name, `trailing` (a count, a mark); the chosen one raised.
pub fn sidebar_row(leading: Option<&gtk::Widget>, title: &str, trailing: Option<&gtk::Widget>, selected: bool) -> gtk::Button {
    let line = gtk::Box::builder().spacing(10).build();
    let slot = gtk::Box::builder().width_request(22).valign(gtk::Align::Center).build();
    if let Some(l) = leading {
        slot.append(l);
    }
    line.append(&slot);
    line.append(&gtk::Label::builder().label(title).xalign(0.0).hexpand(true).ellipsize(gtk::pango::EllipsizeMode::End).css_classes(["sidebar-title"]).build());
    if let Some(t) = trailing {
        t.add_css_class("dim-label");
        line.append(t);
    }
    let b = gtk::Button::builder().child(&line).css_classes(["sidebar-row"]).build();
    if selected {
        b.add_css_class("selected");
    }
    b
}

/// An icon of the rail: 40 square, a soft tile when chosen, the name as a tooltip.
pub fn rail_button(mark: &impl IsA<gtk::Widget>, tooltip: &str, selected: bool) -> gtk::Button {
    let b = gtk::Button::builder().child(mark).tooltip_text(tooltip).css_classes(["rail-btn"]).halign(gtk::Align::Center).build();
    if selected {
        b.add_css_class("selected");
    }
    b
}

/// A rail button with an icon.
pub fn rail_icon(name: &str, tooltip: &str, selected: bool) -> gtk::Button {
    rail_button(&icon(name, 20, 1.75, "currentColor"), tooltip, selected)
}

// ---- Toolbar ----------------------------------------------------------------

/// The top of a page: the title (markup, so a grey part can follow it), the
/// page's controls, the round +. The title gives way first (ellipsized).
pub fn toolbar(title_markup: &str, compact: bool, controls: &[gtk::Widget], add: Option<&gtk::Button>) -> (gtk::Box, gtk::Label) {
    let bar = gtk::Box::builder().spacing(14).margin_bottom(if compact { 12 } else { 18 }).build();
    let title = gtk::Label::builder().xalign(0.0).hexpand(true).ellipsize(gtk::pango::EllipsizeMode::End).css_classes(["toolbar-title"]).build();
    if compact {
        title.add_css_class("compact");
    }
    title.set_markup(title_markup);
    bar.append(&title);
    for c in controls {
        c.set_valign(gtk::Align::Center);
        bar.append(c);
    }
    if let Some(a) = add {
        bar.append(a);
    }
    (bar, title)
}

/// The round blue + (New event, New task, Add photos).
pub fn plus_button(tooltip: &str) -> gtk::Button {
    gtk::Button::builder().child(&icon("plus", 20, 2.2, "#ffffff")).tooltip_text(tooltip).css_classes(["plus-btn"]).valign(gtk::Align::Center).build()
}

// ---- Detail card and editor ----------------------------------------------------

/// The detail card (Calendar's event card): the title with `actions` and ✕,
/// then a box for the app's lines and icon rows. 460 wide; Esc closes.
pub fn detail_sheet(title: &str, actions: &[gtk::Widget]) -> (adw::Dialog, gtk::Box) {
    let d = adw::Dialog::builder().content_width(460).build();
    let col = gtk::Box::builder().orientation(gtk::Orientation::Vertical).margin_top(22).margin_bottom(22).margin_start(22).margin_end(22).build();
    let head = gtk::Box::builder().spacing(4).build();
    head.append(&gtk::Label::builder().label(title).xalign(0.0).hexpand(true).wrap(true).css_classes(["sheet-title"]).build());
    for a in actions {
        head.append(a);
    }
    let close = icon_btn("close", 19, "Close (Esc)");
    {
        let d = d.clone();
        close.connect_clicked(move |_| {
            d.close();
        });
    }
    head.append(&close);
    col.append(&head);
    let body = gtk::Box::builder().orientation(gtk::Orientation::Vertical).build();
    col.append(&body);
    let scroll = gtk::ScrolledWindow::builder().hscrollbar_policy(gtk::PolicyType::Never).propagate_natural_height(true).max_content_height(640).child(&col).build();
    d.set_child(Some(&scroll));
    (d, body)
}

/// The editor (Calendar's event editor): the title big and editable with ✕,
/// a line under it (`under`, e.g. the calendar), a box for icon rows, then
/// Delete on the left and Save on the right. Esc cancels, Enter in the title
/// or Ctrl+Enter anywhere saves (`on_save` returns whether to close).
pub struct EditSheet {
    pub dialog: adw::Dialog,
    pub title: gtk::Entry,
    pub under: gtk::Box,
    pub body: gtk::Box,
    pub save: gtk::Button,
    pub delete: Option<gtk::Button>,
}

pub fn edit_sheet(placeholder: &str, title: &str, is_new: bool, on_save: impl Fn() -> bool + 'static, on_delete: Option<Box<dyn Fn()>>) -> EditSheet {
    let dialog = adw::Dialog::builder().content_width(460).build();
    let col = gtk::Box::builder().orientation(gtk::Orientation::Vertical).margin_top(22).margin_bottom(22).margin_start(22).margin_end(22).build();
    let head = gtk::Box::builder().spacing(4).build();
    let entry = gtk::Entry::builder().placeholder_text(placeholder).text(title).hexpand(true).css_classes(["sheet-title-edit"]).build();
    head.append(&entry);
    let close = icon_btn("close", 19, "Cancel (Esc)");
    {
        let d = dialog.clone();
        close.connect_clicked(move |_| {
            d.close();
        });
    }
    head.append(&close);
    col.append(&head);
    let under = gtk::Box::builder().spacing(6).margin_top(4).margin_bottom(8).build();
    col.append(&under);
    let body = gtk::Box::builder().orientation(gtk::Orientation::Vertical).build();
    col.append(&body);
    let actions = gtk::Box::builder().spacing(10).margin_top(18).build();
    let on_save: Rc<dyn Fn() -> bool> = Rc::new(on_save);
    let delete = match (on_delete, is_new) {
        (Some(f), false) => {
            let b = btn("Delete", None, &["danger"]);
            let d = dialog.clone();
            b.connect_clicked(move |_| {
                d.close();
                f();
            });
            actions.append(&b);
            Some(b)
        }
        _ => None,
    };
    actions.append(&gtk::Box::builder().hexpand(true).build());
    if is_new {
        let c = btn("Cancel", None, &[]);
        let d = dialog.clone();
        c.connect_clicked(move |_| {
            d.close();
        });
        actions.append(&c);
    }
    let save = btn(if is_new { "Add" } else { "Save" }, None, &["primary"]);
    let run = {
        let (d, f) = (dialog.clone(), on_save.clone());
        Rc::new(move || {
            if f() {
                d.close();
            }
        })
    };
    {
        let r = run.clone();
        save.connect_clicked(move |_| r());
        let r = run.clone();
        entry.connect_activate(move |_| r());
    }
    actions.append(&save);
    col.append(&actions);
    // Ctrl+Enter saves from anywhere in the editor.
    let keys = gtk::ShortcutController::new();
    {
        let r = run.clone();
        keys.add_shortcut(gtk::Shortcut::new(
            gtk::ShortcutTrigger::parse_string("<Control>Return"),
            Some(gtk::CallbackAction::new(move |_, _| {
                r();
                glib::Propagation::Stop
            })),
        ));
    }
    col.add_controller(keys);
    let scroll = gtk::ScrolledWindow::builder().hscrollbar_policy(gtk::PolicyType::Never).propagate_natural_height(true).max_content_height(640).child(&col).build();
    dialog.set_child(Some(&scroll));
    EditSheet { dialog, title: entry, under, body, save, delete }
}

// ---- Empty state and list rows ---------------------------------------------------

/// An empty list or library: an icon in a soft tile, a line, maybe one button.
pub fn empty_state(icon_name: &str, headline: &str, color: &str, action: Option<&gtk::Button>) -> gtk::Box {
    let b = gtk::Box::builder().orientation(gtk::Orientation::Vertical).spacing(10).halign(gtk::Align::Center).margin_top(56).margin_bottom(56).build();
    let tile = gtk::Box::builder().halign(gtk::Align::Center).css_classes(["empty-tile"]).build();
    let i = icon(icon_name, 28, 1.75, color);
    i.set_hexpand(true);
    i.set_vexpand(true);
    tile.append(&i);
    b.append(&tile);
    b.append(&gtk::Label::builder().label(headline).css_classes(["empty-headline"]).build());
    if let Some(a) = action {
        a.set_halign(gtk::Align::Center);
        a.set_margin_top(8);
        b.append(a);
    }
    b
}

/// A row of a list: `leading` (a check, a dot), a title with an optional grey
/// line, `trailing`, a hairline under it (none with `last`).
pub fn item_row(leading: Option<&gtk::Widget>, title: &str, detail: &str, done: bool, trailing: Option<&gtk::Widget>, last: bool) -> gtk::Box {
    let row = gtk::Box::builder().spacing(12).css_classes(["item-row"]).build();
    if last {
        row.add_css_class("last");
    }
    if let Some(l) = leading {
        l.set_valign(gtk::Align::Start);
        row.append(l);
    }
    let text = gtk::Box::builder().orientation(gtk::Orientation::Vertical).spacing(2).hexpand(true).build();
    let t = gtk::Label::builder().label(title).xalign(0.0).wrap(true).lines(2).ellipsize(gtk::pango::EllipsizeMode::End).css_classes(["item-title"]).build();
    if done {
        t.add_css_class("done");
    }
    text.append(&t);
    if !detail.is_empty() {
        text.append(&gtk::Label::builder().label(detail).xalign(0.0).ellipsize(gtk::pango::EllipsizeMode::End).css_classes(["item-detail"]).build());
    }
    row.append(&text);
    if let Some(tr) = trailing {
        tr.set_valign(gtk::Align::Center);
        row.append(tr);
    }
    row
}

// ---- Tints ------------------------------------------------------------------------

/// What kind of item something is, for a quiet shade of its colour.
#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct TintKind {
    pub repeating: bool,
    pub joinable: bool,
    pub all_day: bool,
    pub past: bool,
    pub declined: bool,
    pub tentative: bool,
}

impl TintKind {
    /// From the core's fields: repeats, join, past, answer, solid / row (all-day).
    pub fn of(e: &Value) -> TintKind {
        let t = |k: &str| e[k] == Value::Bool(true);
        TintKind {
            repeating: t("repeats"),
            joinable: t("join"),
            all_day: t("solid") || t("row"),
            past: t("past"),
            declined: e["answer"] == "no",
            tentative: e["answer"] == "maybe",
        }
    }
}

/// How much of the colour fills the item, how strong its edge is, how faded it is.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tint {
    pub fill: f64,
    pub edge: f64,
    pub fade: f64,
}

/// A meeting to join a little deeper, a repeating one lighter with a softer
/// edge, all-day a touch lighter, past and declined faded. Subtle on purpose.
pub fn tint(k: TintKind) -> Tint {
    let mut fill = if k.all_day { 0.22 } else { 0.24 };
    if k.joinable {
        fill += 0.08;
    } else if k.repeating {
        fill -= 0.09;
    }
    Tint { fill, edge: if k.repeating { 0.55 } else { 1.0 }, fade: if k.declined { 0.45 } else if k.past { 0.6 } else { 1.0 } }
}
