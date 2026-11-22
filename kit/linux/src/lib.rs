//! What every UUOB app's Linux window shares, as `ui/Gate.svelte` and
//! friends are for the web version: the connect screen (until the app is
//! connected to UUOB), the "waiting for UUOB" note, UUOB's logo and the
//! update button. Any app can use it; nothing in it is about one app.
//!
//! ```ignore
//! let gate = Gate::new(info, client.clone(), move || show_the_app());
//! window.set_content(Some(gate.widget()));
//! gate.set_content(&my_app_widget);
//! ```

use std::cell::{Cell, RefCell};
use std::f64::consts::PI;
use std::rc::Rc;

use relm4::adw::prelude::*;
use relm4::{adw, gtk, gtk::glib};
use uuob_client::{Client, Event, State};

/// Who the app is, for its connect screen.
#[derive(Clone)]
pub struct AppInfo {
    pub id: String,
    pub name: String,
    /// What the app keeps, as in "your photos".
    pub noun: String,
    /// A symbolic icon name for the app's tile.
    pub icon: String,
    /// One of the calendar colours (`blue`, `orange`, …).
    pub color: String,
    pub version: String,
}

/// The calendar colours every app shares (tokens.css), as RGB.
pub fn color(name: &str) -> (f64, f64, f64) {
    // The light or dark value, as the theme is now (drawing reads it each time).
    let dark = adw::StyleManager::default().is_dark();
    let hex = match (name, dark) {
        ("red", true) => 0xFF453A,
        ("red", false) => 0xFF3B30,
        ("orange", true) => 0xFF9F0A,
        ("orange", false) => 0xFF9500,
        ("yellow", true) => 0xFFD60A,
        ("yellow", false) => 0xFFCC00,
        ("green", true) => 0x30D158,
        ("green", false) => 0x34C759,
        ("teal", true) => 0x40C8E0,
        ("teal", false) => 0x30B0C7,
        ("purple", true) => 0xBF5AF2,
        ("purple", false) => 0xAF52DE,
        ("pink", true) => 0xFF375F,
        ("pink", false) => 0xFF2D55,
        (_, true) => 0x0A84FF,
        (_, false) => 0x007AFF,
    };
    (((hex >> 16) & 255) as f64 / 255.0, ((hex >> 8) & 255) as f64 / 255.0, (hex & 255) as f64 / 255.0)
}

fn css_text() -> &'static str {
    ".uuob-gate { padding: 32px 20px; }
         .uuob-gate .title-1 { margin-top: 28px; }
         .uuob-lead { margin-top: 12px; }
         .uuob-note { padding: 10px 18px; border-radius: 22px; background: alpha(@window_fg_color, 0.08); margin: 16px; }
         .uuob-note.warning { color: @warning_color; }
         .uuob-dot { min-width: 8px; min-height: 8px; border-radius: 4px; background: @success_color; }
         .uuob-code { font-size: 18px; }
         .uuob-segmented button { min-width: 74px; }"
}

/// The calendar colours, in the order things take them.
pub const COLORS: [&str; 8] = ["blue", "red", "orange", "yellow", "green", "teal", "purple", "pink"];

/// The styles the kit's widgets use; call once. Also gives Libadwaita UUOB's
/// own colours (ui/tokens.css), light and dark, and the web version's
/// building blocks as classes: `btn` (`primary`, `danger`, `ghost`, `lg`),
/// `field`, `segmented`, `card-web`, `section-label`, `large-title`.
pub fn style() {
    let display = gtk::gdk::Display::default().expect("display");
    let base = gtk::CssProvider::new();
    let mut colors = String::new();
    for name in COLORS {
        let (r, g, b) = color(name);
        let rgb = format!("rgb({},{},{})", (r * 255.0).round() as u8, (g * 255.0).round() as u8, (b * 255.0).round() as u8);
        // The palette's colour, light or dark as the theme is (see LIGHT and DARK).
        let _ = rgb;
        colors.push_str(&format!(".uuob-c-{name} {{ background: @uuob_{name}; color: white; }} .uuob-fg-{name} {{ color: @uuob_{name}; }}\n"));
    }
    base.load_from_string(&format!("{}{colors}{}", css_text(), WEB_CSS));
    gtk::style_context_add_provider_for_display(&display, &base, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);
    // The palette follows light and dark mode.
    let palette = gtk::CssProvider::new();
    gtk::style_context_add_provider_for_display(&display, &palette, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);
    let manager = adw::StyleManager::default();
    let apply = move |m: &adw::StyleManager| palette.load_from_string(if m.is_dark() { DARK } else { LIGHT });
    apply(&manager);
    manager.connect_dark_notify(apply);
}

const LIGHT: &str = "
@define-color window_bg_color #f5f5f7; @define-color window_fg_color #1d1d1f;
@define-color view_bg_color #ffffff; @define-color view_fg_color #1d1d1f;
@define-color card_bg_color #ffffff; @define-color headerbar_bg_color #f5f5f7; @define-color headerbar_fg_color #1d1d1f;
@define-color sidebar_bg_color #ffffff; @define-color sidebar_fg_color #1d1d1f;
@define-color popover_bg_color #ffffff; @define-color dialog_bg_color #ffffff;
@define-color accent_bg_color #007aff; @define-color accent_color #007aff; @define-color accent_fg_color #ffffff;
@define-color destructive_bg_color #ff3b30; @define-color destructive_color #ff3b30; @define-color success_color #34c759; @define-color error_color #ff3b30;
@define-color uuob_blue #007aff; @define-color uuob_red #ff3b30; @define-color uuob_orange #ff9500; @define-color uuob_yellow #ffcc00; @define-color uuob_green #34c759; @define-color uuob_teal #30b0c7; @define-color uuob_purple #af52de; @define-color uuob_pink #ff2d55;
@define-color uuob_surface2 #f0f0f3; @define-color uuob_line rgba(0,0,0,0.08); @define-color uuob_text2 rgba(29,29,31,0.62); @define-color uuob_text3 rgba(29,29,31,0.38);
";

const DARK: &str = "
@define-color window_bg_color #0b0b0d; @define-color window_fg_color #f5f5f7;
@define-color view_bg_color #161618; @define-color view_fg_color #f5f5f7;
@define-color card_bg_color #1c1c1f; @define-color headerbar_bg_color #0b0b0d; @define-color headerbar_fg_color #f5f5f7;
@define-color sidebar_bg_color #161618; @define-color sidebar_fg_color #f5f5f7;
@define-color popover_bg_color #26262a; @define-color dialog_bg_color #161618;
@define-color accent_bg_color #0a84ff; @define-color accent_color #0a84ff; @define-color accent_fg_color #ffffff;
@define-color destructive_bg_color #ff453a; @define-color destructive_color #ff453a; @define-color success_color #30d158; @define-color error_color #ff453a;
@define-color uuob_blue #0a84ff; @define-color uuob_red #ff453a; @define-color uuob_orange #ff9f0a; @define-color uuob_yellow #ffd60a; @define-color uuob_green #30d158; @define-color uuob_teal #40c8e0; @define-color uuob_purple #bf5af2; @define-color uuob_pink #ff375f;
@define-color uuob_surface2 #26262a; @define-color uuob_line rgba(255,255,255,0.08); @define-color uuob_text2 rgba(245,245,247,0.62); @define-color uuob_text3 rgba(245,245,247,0.38);
";

/// The web version's elements (ui/tokens.css), as GTK classes.
const WEB_CSS: &str = "
window, .background { font-size: 15px; }
.large-title { font-size: 34px; font-weight: 700; letter-spacing: -0.8px; }
.web-title { font-size: 22px; font-weight: 600; letter-spacing: -0.4px; }
.section-label { font-size: 13px; font-weight: 600; color: @uuob_text3; letter-spacing: 0.5px; }
.secondary { color: @uuob_text2; }
.caption-web { font-size: 13px; color: @uuob_text2; }
button.btn { min-height: 40px; padding: 0 18px; border-radius: 999px; background: @uuob_surface2; color: @window_fg_color; font-size: 15px; font-weight: 600; box-shadow: none; border: 0; }
button.btn:hover { background: mix(@uuob_surface2, @window_fg_color, 0.08); }
button.btn.primary { background: @accent_bg_color; color: white; }
button.btn.primary:hover { background: mix(@accent_bg_color, white, 0.12); }
button.btn.danger { background: alpha(@destructive_color, 0.12); color: @destructive_color; }
button.btn.ghost { background: none; color: @accent_color; }
button.btn.ghost:hover { background: alpha(@accent_color, 0.12); }
button.btn.lg { min-height: 50px; padding: 0 26px; font-size: 17px; }
button.btn.sm { min-height: 32px; padding: 0 14px; font-size: 14px; }
button.btn:disabled { opacity: 0.45; }
button.icon-btn { min-width: 36px; min-height: 36px; padding: 0; border-radius: 999px; background: none; color: @uuob_text2; box-shadow: none; }
button.icon-btn:hover { background: alpha(@window_fg_color, 0.06); color: @window_fg_color; }
menubutton.icon-btn > button { min-width: 36px; min-height: 36px; padding: 0; border-radius: 999px; background: none; box-shadow: none; }
menubutton.icon-btn > button:hover, menubutton.icon-btn > button:checked { background: alpha(@window_fg_color, 0.06); }
entry.field, .field { min-height: 46px; border-radius: 12px; background: @uuob_surface2; border: 1px solid @uuob_line; padding: 0 14px; font-size: 15px; box-shadow: none; outline: none; }
entry.field:focus-within, .field:focus-within { border-color: @accent_bg_color; box-shadow: 0 0 0 4px alpha(@accent_bg_color, 0.16); background: @view_bg_color; }
entry.field.field-title, entry.field.field-title:focus-within { font-size: 22px; font-weight: 650; background: none; border: 0; box-shadow: none; padding: 0 2px; min-height: 56px; }
.segmented { padding: 3px; border-radius: 10px; background: @uuob_surface2; }
.segmented > button { min-height: 0; padding: 5px 14px; border-radius: 8px; font-size: 14px; font-weight: 560; background: none; box-shadow: none; color: @uuob_text2; }
.segmented > button:checked { background: @view_bg_color; color: @window_fg_color; box-shadow: 0 1px 3px rgba(0,0,0,0.25); }
.card-web { background: @card_bg_color; border: 1px solid @uuob_line; border-radius: 18px; }
.web-rows > .web-row { min-height: 56px; padding: 14px 18px; }
.web-rows > .web-row:not(:first-child) { border-top: 1px solid @uuob_line; }
.web-row-title { font-size: 16px; font-weight: 600; }
/* Rows as the event card draws them: an icon, the content, a hairline under it; no boxes. */
.icon-row { min-height: 44px; padding: 2px 0; border-bottom: 1px solid @uuob_line; }
.icon-row.last { border-bottom: none; }
/* Controls in a row are flat, as on the card: the lines are enough. */
.icon-row .field, .icon-row entry.field, .icon-row textview.field, .icon-row menubutton.field > button, .icon-row dropdown.field > button { background: transparent; border: none; box-shadow: none; outline: none; }
.icon-row .field:focus-within { background: @uuob_surface2; }
.icon-row-title { font-size: 15px; }
.rows-label { font-size: 12px; font-weight: 600; color: @uuob_text3; letter-spacing: 0.5px; margin-top: 18px; margin-bottom: 2px; }
toast { background: @dialog_bg_color; color: @window_fg_color; border: 1px solid alpha(@window_fg_color, 0.14); border-radius: 999px; box-shadow: 0 8px 28px rgba(0,0,0,0.25); padding: 6px 8px 6px 20px; font-size: 15px; font-weight: 500; }
toast > button.circular { background: none; color: @uuob_text3; min-width: 24px; min-height: 24px; }
.fg-text { color: @window_fg_color; } .fg-text2 { color: @uuob_text2; } .fg-text3 { color: @uuob_text3; } .fg-accent { color: @accent_color; } .fg-danger { color: @destructive_color; }
dialog.web-sheet sheet, dialog.web-sheet .dialog-contents { background: @dialog_bg_color; }
dropdown.field > button, menubutton.field > button { background: none; box-shadow: none; border: 0; min-height: 44px; padding: 0; font-weight: 400; }
textview.field { padding: 10px 4px; }
textview.field > text { background: none; }
";

fn rounded(cr: &gtk::cairo::Context, x: f64, y: f64, w: f64, h: f64, r: f64) {
    cr.new_sub_path();
    cr.arc(x + w - r, y + r, r, -PI / 2.0, 0.0);
    cr.arc(x + w - r, y + h - r, r, 0.0, PI / 2.0);
    cr.arc(x + r, y + h - r, r, PI / 2.0, PI);
    cr.arc(x + r, y + r, r, PI, 1.5 * PI);
    cr.close_path();
}

/// UUOB's logo (Mesh), as ui/Logo.svelte draws it.
pub fn logo(size: i32) -> gtk::DrawingArea {
    let area = gtk::DrawingArea::builder().content_width(size).content_height(size).valign(gtk::Align::Center).build();
    area.set_draw_func(|_, cr, w, _| {
        let w = w as f64;
        rounded(cr, 0.0, 0.0, w, w, w * 0.28);
        cr.set_source_rgb(0.0, 0.0, 0.0);
        let _ = cr.fill();
        cr.save().ok();
        cr.translate(w / 2.0, w / 2.0);
        cr.scale(w / 1024.0 * 1.08, w / 1024.0 * 1.08);
        cr.translate(-512.0, -512.0);
        cr.set_source_rgb(1.0, 1.0, 1.0);
        cr.set_line_cap(gtk::cairo::LineCap::Round);
        let line = |x1: f64, y1: f64, x2: f64, y2: f64, sw: f64| {
            cr.set_line_width(sw);
            cr.move_to(x1, y1);
            cr.line_to(x2, y2);
            let _ = cr.stroke();
        };
        let dot = |x: f64, y: f64, r: f64| {
            cr.arc(x, y, r, 0.0, 2.0 * PI);
            let _ = cr.fill();
        };
        let boxed = |x: f64, y: f64, bw: f64, bh: f64, r: f64| {
            cr.set_line_width(29.9);
            rounded(cr, x, y, bw, bh, r);
            let _ = cr.stroke();
        };
        line(430.5, 429.3, 349.1, 579.2, 22.0);
        dot(389.8, 504.3, 34.0);
        line(593.5, 429.3, 672.2, 574.3, 22.0);
        dot(632.8, 501.8, 34.0);
        line(435.5, 724.4, 582.9, 724.4, 22.0);
        dot(509.2, 724.4, 34.0);
        boxed(411.2, 192.4, 201.6, 134.4, 19.6);
        line(374.8, 360.4, 649.2, 360.4, 29.9);
        boxed(208.7, 612.4, 123.2, 224.0, 28.0);
        dot(270.3, 788.8, 15.4);
        boxed(647.3, 623.6, 212.8, 86.8, 19.6);
        boxed(647.3, 738.4, 212.8, 86.8, 19.6);
        dot(812.5, 667.0, 12.6);
        dot(812.5, 781.8, 12.6);
        cr.restore().ok();
    });
    area
}

/// An app's tile: its colour and its icon.
pub fn tile(info: &AppInfo, size: i32) -> gtk::Widget {
    let (r, g, b) = color(&info.color);
    let area = gtk::DrawingArea::builder().content_width(size).content_height(size).valign(gtk::Align::Center).build();
    area.set_draw_func(move |_, cr, w, _| {
        let w = w as f64;
        rounded(cr, 0.0, 0.0, w, w, w * 0.28);
        cr.set_source_rgb(r, g, b);
        let _ = cr.fill();
    });
    let icon = gtk::Image::builder().icon_name(&info.icon).pixel_size(size / 2).css_classes(["uuob-tile-icon"]).build();
    icon.add_css_class("white");
    let overlay = gtk::Overlay::new();
    overlay.set_child(Some(&area));
    overlay.add_overlay(&icon);
    overlay.upcast()
}

/// A code is a number and three words, like 42-amber-river-falcon.
pub fn code_ok(code: &str) -> bool {
    code.split(|c: char| !c.is_ascii_alphanumeric()).filter(|s| !s.is_empty()).count() == 4
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum Step {
    Checking,
    Unreachable,
    Connect,
    Waiting,
    Ready,
}

struct Inner {
    info: AppInfo,
    client: Client,
    stack: gtk::Stack,
    page: gtk::Box,
    note: gtk::Label,
    step: Cell<Step>,
    offline: Cell<bool>,
    asking: Cell<bool>,
    busy: Cell<bool>,
    use_code: Cell<bool>,
    error: RefCell<String>,
    code: RefCell<String>,
    on_ready: Box<dyn Fn()>,
}

/// The app until it is connected to UUOB, then the app's own content.
#[derive(Clone)]
pub struct Gate(Rc<Inner>);

impl Gate {
    /// `on_ready` runs each time the app becomes connected (load the data then).
    pub fn new(info: AppInfo, client: Client, on_ready: impl Fn() + 'static) -> Gate {
        let stack = gtk::Stack::builder().transition_type(gtk::StackTransitionType::Crossfade).build();
        let page = gtk::Box::new(gtk::Orientation::Vertical, 0);
        let scroll = gtk::ScrolledWindow::builder().hscrollbar_policy(gtk::PolicyType::Never).child(&page).build();
        stack.add_named(&scroll, Some("gate"));
        stack.add_named(&gtk::Box::new(gtk::Orientation::Vertical, 0), Some("checking"));
        let note = gtk::Label::builder().wrap(true).justify(gtk::Justification::Center).halign(gtk::Align::Center).valign(gtk::Align::End).css_classes(["uuob-note"]).visible(false).build();
        let step = if client.has_token() { Step::Ready } else { Step::Checking };
        let gate = Gate(Rc::new(Inner {
            info,
            client,
            stack,
            page,
            note,
            step: Cell::new(step),
            offline: Cell::new(false),
            asking: Cell::new(false),
            busy: Cell::new(false),
            use_code: Cell::new(false),
            error: RefCell::new(String::new()),
            code: RefCell::new(String::new()),
            on_ready: Box::new(on_ready),
        }));
        gate.render();
        // Changes of state from the SDK.
        let weak = Rc::downgrade(&gate.0);
        let mut events = gate.0.client.subscribe();
        glib::spawn_future_local(async move {
            while let Ok(e) = events.recv().await {
                let Some(g) = weak.upgrade().map(Gate) else { return };
                match e {
                    Event::State(s) => {
                        if s == State::Disconnected && g.0.step.get() == Step::Ready {
                            g.set(Step::Connect);
                        }
                        g.0.offline.set(s == State::Unreachable && g.0.step.get() == Step::Ready);
                        g.render_note();
                    }
                    Event::Asking(a) => {
                        g.0.asking.set(a);
                        g.render_note();
                    }
                    _ => {}
                }
            }
        });
        // Look for UUOB now, and every 3 seconds while it's away.
        let weak = Rc::downgrade(&gate.0);
        glib::spawn_future_local(async move {
            let mut first = true;
            loop {
                let Some(g) = weak.upgrade().map(Gate) else { return };
                let away = g.0.step.get() == Step::Unreachable || g.0.offline.get();
                if (first || away) && !g.0.busy.get() {
                    g.check().await;
                }
                first = false;
                drop(g);
                glib::timeout_future_seconds(3).await;
            }
        });
        if step == Step::Ready {
            glib::idle_add_local_once({
                let g = gate.clone();
                move || (g.0.on_ready)()
            });
        }
        gate
    }

    /// The widget to put in the window.
    pub fn widget(&self) -> gtk::Widget {
        let overlay = gtk::Overlay::new();
        overlay.set_child(Some(&self.0.stack));
        overlay.add_overlay(&self.0.note);
        overlay.upcast()
    }

    /// The app itself, shown once it is connected.
    pub fn set_content(&self, content: &impl IsA<gtk::Widget>) {
        if let Some(old) = self.0.stack.child_by_name("ready") {
            self.0.stack.remove(&old);
        }
        self.0.stack.add_named(content, Some("ready"));
        self.render();
    }

    fn set(&self, step: Step) {
        let was = self.0.step.replace(step);
        self.render();
        if step == Step::Ready && was != Step::Ready {
            (self.0.on_ready)();
        }
    }

    async fn check(&self) {
        let s = self.0.client.check().await;
        if self.0.step.get() == Step::Waiting {
            return;
        }
        if s == State::Unreachable && self.0.step.get() == Step::Ready {
            self.0.offline.set(true);
            self.render_note();
            return;
        }
        self.0.offline.set(false);
        self.render_note();
        self.set(match s {
            State::Connected => Step::Ready,
            State::Disconnected => Step::Connect,
            State::Unreachable => Step::Unreachable,
        });
    }

    fn connect(&self, with_code: bool) {
        *self.0.error.borrow_mut() = String::new();
        let from = self.0.step.get();
        if !with_code {
            self.set(Step::Waiting);
        } else {
            self.0.busy.set(true);
            self.render();
        }
        let g = self.clone();
        let code = with_code.then(|| self.0.code.borrow().trim().to_string());
        glib::spawn_future_local(async move {
            let r = g.0.client.connect(code).await;
            g.0.busy.set(false);
            match r {
                Ok(()) => {
                    g.0.offline.set(false);
                    *g.0.code.borrow_mut() = String::new();
                    g.set(Step::Ready);
                }
                Err(e) if e.code() == "cancelled" => g.set(Step::Connect),
                Err(e) => {
                    *g.0.error.borrow_mut() = e.to_string();
                    // Connected before (or refused): only a code from UUOB connects it now.
                    if !with_code && e.code() == "permission_denied" {
                        g.0.use_code.set(true);
                    }
                    // A refused code leaves the user where they typed it.
                    g.set(if e.code() == "unreachable" { Step::Unreachable } else if with_code { from } else { Step::Connect });
                }
            }
        });
    }

    fn render_note(&self) {
        let i = &self.0;
        let text = if i.step.get() != Step::Ready {
            None
        } else if i.offline.get() {
            Some("Waiting for UUOB. Your changes are kept and sent when it’s back.")
        } else if i.asking.get() {
            Some("Answer in UUOB on one of your devices.")
        } else {
            None
        };
        i.note.set_label(text.unwrap_or_default());
        i.note.set_visible(text.is_some());
        if i.offline.get() {
            i.note.add_css_class("warning");
        } else {
            i.note.remove_css_class("warning");
        }
    }

    fn render(&self) {
        let i = &self.0;
        self.render_note();
        match i.step.get() {
            Step::Ready if i.stack.child_by_name("ready").is_some() => return i.stack.set_visible_child_name("ready"),
            Step::Checking | Step::Ready => return i.stack.set_visible_child_name("checking"),
            _ => i.stack.set_visible_child_name("gate"),
        }
        while let Some(c) = i.page.first_child() {
            i.page.remove(&c);
        }
        let column = gtk::Box::builder().orientation(gtk::Orientation::Vertical).halign(gtk::Align::Center).valign(gtk::Align::Center).vexpand(true).width_request(320).css_classes(["uuob-gate"]).build();
        let clamp = adw::Clamp::builder().maximum_size(420).child(&column).build();
        i.page.append(&clamp);

        let pair = gtk::Box::builder().spacing(14).halign(gtk::Align::Center).build();
        pair.append(&tile(&i.info, 68));
        let dots = gtk::Label::builder().label("• • •").css_classes(["dim-label"]).build();
        pair.append(&dots);
        pair.append(&logo(68));
        column.append(&pair);

        let name = &i.info.name;
        let title = |t: &str| gtk::Label::builder().label(t).wrap(true).justify(gtk::Justification::Center).css_classes(["large-title"]).build();
        let lead = |t: &str| gtk::Label::builder().label(t).wrap(true).justify(gtk::Justification::Center).css_classes(["uuob-lead", "dim-label"]).build();
        let wide = |label: &str, suggested: bool| {
            let b = gtk::Button::builder().label(label).css_classes(["btn", "lg"]).margin_top(14).build();
            if suggested {
                b.add_css_class("primary");
            }
            b
        };
        let looking = |t: &str| {
            let row = gtk::Box::builder().spacing(8).halign(gtk::Align::Center).margin_top(22).build();
            row.append(&gtk::Spinner::builder().spinning(true).build());
            row.append(&gtk::Label::builder().label(t).css_classes(["dim-label"]).build());
            row
        };
        let error = || {
            let e = i.error.borrow().clone();
            gtk::Label::builder().label(&e).wrap(true).visible(!e.is_empty()).css_classes(["error"]).margin_top(12).build()
        };
        let code_form = |g: &Gate| {
            let row = gtk::Box::builder().spacing(10).margin_top(18).build();
            let entry = gtk::Entry::builder().placeholder_text("42-amber-river-falcon").hexpand(true).text(i.code.borrow().as_str()).css_classes(["field", "uuob-code"]).build();
            let go = gtk::Button::builder().css_classes(["btn", "primary"]).label(if i.busy.get() { "Connecting…" } else { "Connect" }).sensitive(code_ok(&i.code.borrow()) && !i.busy.get()).build();
            entry.connect_changed({
                let g = g.clone();
                let go = go.clone();
                move |e| {
                    *g.0.code.borrow_mut() = e.text().to_string();
                    go.set_sensitive(code_ok(&e.text()) && !g.0.busy.get());
                }
            });
            entry.connect_activate({
                let g = g.clone();
                move |e| {
                    if code_ok(&e.text()) && !g.0.busy.get() {
                        g.connect(true)
                    }
                }
            });
            go.connect_clicked({
                let g = g.clone();
                move |_| g.connect(true)
            });
            row.append(&entry);
            row.append(&go);
            row
        };
        match i.step.get() {
            Step::Unreachable => {
                column.append(&title(&format!("{name} needs UUOB")));
                column.append(&lead("Install UUOB on this device and open it."));
                column.append(&looking("Looking for UUOB…"));
                column.append(&gtk::Separator::builder().margin_top(32).margin_bottom(28).build());
                column.append(&gtk::Label::builder().label("Or use UUOB on another device").css_classes(["heading"]).build());
                column.append(&code_form(self));
                column.append(&error());
            }
            Step::Waiting => {
                column.append(&title("Allow it in UUOB"));
                column.append(&looking("Waiting for your answer…"));
                let cancel = wide("Cancel", false);
                cancel.connect_clicked({
                    let g = self.clone();
                    move |_| g.0.client.cancel_connect()
                });
                column.append(&cancel);
            }
            _ => {
                column.append(&title(&format!("Connect {name}")));
                let found = gtk::Box::builder().spacing(8).halign(gtk::Align::Center).margin_top(22).build();
                found.append(&gtk::Box::builder().css_classes(["uuob-dot"]).valign(gtk::Align::Center).build());
                found.append(&gtk::Label::builder().label(format!("UUOB is running on {}", i.client.device().unwrap_or_else(|| "this device".into()))).css_classes(["dim-label"]).build());
                column.append(&found);
                let go = wide("Connect to UUOB", true);
                go.connect_clicked({
                    let g = self.clone();
                    move |_| g.connect(false)
                });
                column.append(&go);
                column.append(&error());
                if i.use_code.get() {
                    column.append(&code_form(self));
                } else {
                    let link = gtk::Button::builder().label("Use a code instead").css_classes(["btn", "ghost"]).halign(gtk::Align::Center).margin_top(18).build();
                    link.connect_clicked({
                        let g = self.clone();
                        move |_| {
                            g.0.use_code.set(true);
                            g.render();
                        }
                    });
                    column.append(&link);
                }
            }
        }
    }
}

/// The Update button for the app's header bar: hidden until a newer version
/// is published at `feed`. On Linux the app is updated with the system's
/// packages, so it says so instead of installing.
pub fn update_button(feed: &'static str, version: String) -> gtk::Button {
    let b = gtk::Button::builder().css_classes(["suggested-action", "pill"]).visible(false).build();
    let shown = b.clone();
    glib::spawn_future_local(async move {
        let v = version.clone();
        let newer = uuob_client::runtime().spawn_blocking(move || uuob_client::update::check(feed, &v)).await.ok().and_then(|r| r.ok()).flatten();
        if let Some(n) = newer {
            shown.set_label(&format!("Update to {n}"));
            shown.set_tooltip_text(Some("Update with your system’s packages."));
            shown.set_visible(true);
        }
    });
    b
}

// ---- Settings (ui/Settings.svelte) -------------------------------------------

/// Whether the user writes times as 2:30 PM (a setting for all apps, kept in UUOB).
pub fn hour12(client: &Client) -> bool {
    client.cached_settings()["hour12"] == serde_json::json!(true)
}

/// Whether the user's weeks start on Sunday (a setting for all apps).
pub fn sunday_first(client: &Client) -> bool {
    client.cached_settings()["sunday_first"] == serde_json::json!(true)
}

/// Linked buttons, one of them on, like a segmented control.
pub fn segmented(options: &[&str], selected: usize, on_select: impl Fn(usize) + 'static) -> gtk::Box {
    let row = gtk::Box::builder().css_classes(["segmented"]).spacing(2).valign(gtk::Align::Center).build();
    let on_select = Rc::new(on_select);
    let mut first: Option<gtk::ToggleButton> = None;
    for (i, label) in options.iter().enumerate() {
        let b = gtk::ToggleButton::builder().label(*label).active(i == selected).build();
        if let Some(f) = &first {
            b.set_group(Some(f));
        } else {
            first = Some(b.clone());
        }
        let f = on_select.clone();
        b.connect_toggled(move |b| {
            if b.is_active() {
                f(i)
            }
        });
        row.append(&b);
    }
    row
}

/// A row of Settings: its title on the left, a control on the right (`Trailing` on the web).
pub fn setting_row(title: &str, control: &impl IsA<gtk::Widget>) -> gtk::Widget {
    let row = gtk::Box::builder().spacing(14).css_classes(["web-row"]).build();
    row.append(&gtk::Label::builder().label(title).xalign(0.0).hexpand(true).wrap(true).css_classes(["web-row-title"]).build());
    control.set_valign(gtk::Align::Center);
    row.append(control);
    row.upcast()
}

/// A row as the event card draws them: a grey icon (or room for one), then
/// `content`. Put rows in a box with the `icon-rows` class for the hairlines.
pub fn icon_row(icon_name: Option<&str>, content: &impl IsA<gtk::Widget>) -> gtk::Box {
    let row = gtk::Box::builder().spacing(12).css_classes(["icon-row"]).build();
    // The icon by the first line (a row of notes grows downwards).
    let slot = gtk::Box::builder().width_request(20).valign(gtk::Align::Start).margin_top(12).build();
    if let Some(n) = icon_name {
        slot.append(&icon(n, 19, 1.8, text2()));
    }
    row.append(&slot);
    content.set_hexpand(true);
    row.append(content);
    row
}

/// An icon row with a title on the left and a control on the right.
pub fn icon_setting_row(icon_name: Option<&str>, title: &str, control: &impl IsA<gtk::Widget>) -> gtk::Box {
    let inner = gtk::Box::builder().spacing(12).build();
    inner.append(&gtk::Label::builder().label(title).xalign(0.0).hexpand(true).css_classes(["icon-row-title"]).build());
    control.set_valign(gtk::Align::Center);
    inner.append(control);
    icon_row(icon_name, &inner)
}

/// A small grey heading over a few rows ("Views").
pub fn rows_label(text: &str) -> gtk::Label {
    gtk::Label::builder().label(text.to_uppercase()).xalign(0.0).css_classes(["rows-label"]).build()
}

/// A row with a segmented choice.
pub fn choice_row(title: &str, options: &[&str], selected: usize, on_select: impl Fn(usize) + 'static) -> gtk::Widget {
    setting_row(title, &segmented(options, selected, on_select))
}

/// A row with just a line of text (grey), like "No address yet".
pub fn text_row(text: &str) -> gtk::Widget {
    let row = gtk::Box::builder().css_classes(["web-row"]).build();
    row.append(&gtk::Label::builder().label(text).xalign(0.0).hexpand(true).css_classes(["secondary"]).build());
    row.upcast()
}

/// A group of Settings (`.group` on the web): a small label, rows on a
/// white card with a hairline between them, a note under the card.
pub struct Group {
    pub title: String,
    pub note: String,
    pub rows: Vec<gtk::Widget>,
}

pub fn group(title: &str, note: &str, rows: Vec<gtk::Widget>) -> Group {
    Group { title: title.into(), note: note.into(), rows }
}

/// A group in the event card's style: a small heading, then rows with a
/// hairline between them; no card, no note (the rows say enough).
fn group_box(g: &Group) -> gtk::Box {
    let b = gtk::Box::builder().orientation(gtk::Orientation::Vertical).build();
    if !g.title.is_empty() {
        b.append(&rows_label(&g.title));
    }
    let rows = gtk::Box::builder().orientation(gtk::Orientation::Vertical).css_classes(["icon-rows"]).build();
    for (i, r) in g.rows.iter().enumerate() {
        if i == g.rows.len() - 1 {
            r.add_css_class("last");
        }
        rows.append(r);
    }
    b.append(&rows);
    b
}

/// Settings, the same sheet in every app (ui/Settings.svelte): the choices
/// for all apps at the top, the app's own `groups` below them, About last.
/// `on_change` runs after a setting for all apps changed (redraw the times).
pub fn settings_dialog(info: &AppInfo, client: &Client, week: bool, groups: Vec<Group>, on_change: impl Fn() + 'static) -> adw::Dialog {
    let dialog = adw::Dialog::builder().content_width(460).css_classes(["web-sheet"]).build();
    let on_change = Rc::new(on_change);
    let set = {
        let client = client.clone();
        let on_change = on_change.clone();
        move |change: serde_json::Value| {
            let client = client.clone();
            let on_change = on_change.clone();
            glib::spawn_future_local(async move {
                client.set_settings(change).await;
                on_change();
            });
        }
    };
    let col = gtk::Box::builder().orientation(gtk::Orientation::Vertical).margin_top(22).margin_bottom(22).margin_start(22).margin_end(22).build();
    let head = gtk::Box::builder().build();
    head.append(&gtk::Label::builder().label("Settings").xalign(0.0).hexpand(true).css_classes(["web-title"]).build());
    let close = icon_btn("close", 19, "Close");
    {
        let d = dialog.clone();
        close.connect_clicked(move |_| {
            d.close();
        });
    }
    head.append(&close);
    col.append(&head);
    let mut all_rows = vec![];
    let s = set.clone();
    all_rows.push(icon_setting_row(Some("clock"), "Time", &segmented(&["14:30", "2:30 PM"], hour12(client) as usize, move |i| s(serde_json::json!({ "hour12": i == 1 })))).upcast());
    if week {
        let s = set.clone();
        all_rows.push(icon_setting_row(Some("calendar"), "Week starts", &segmented(&["Monday", "Sunday"], sunday_first(client) as usize, move |i| s(serde_json::json!({ "sunday_first": i == 1 })))).upcast());
    }
    col.append(&group_box(&group("All your apps", "", all_rows)));
    for g in &groups {
        col.append(&group_box(g));
    }
    col.append(&gtk::Label::builder().label(format!("{} {}", info.name, info.version)).margin_top(22).css_classes(["caption-web", "dim-label"]).build());
    let scroll = gtk::ScrolledWindow::builder().hscrollbar_policy(gtk::PolicyType::Never).propagate_natural_height(true).max_content_height(620).child(&col).build();
    dialog.set_child(Some(&scroll));
    dialog
}

// ---- Time field (ui/TimeField.svelte) ----------------------------------------

/// A time field that writes times the way the user chose (14:30 or 2:30 PM).
/// Typing "1430", "9", "9.15" or "2pm" works too; anything else goes back to
/// the time it had. `value` is "HH:MM"; with `optional`, emptying the field
/// makes it None. `on_change` runs when the time changed (Enter or leaving).
pub fn time_field(value: Option<String>, hour12: bool, optional: bool, placeholder: &str, on_change: impl Fn(Option<String>) + 'static) -> gtk::Entry {
    use uuob_client::dates;
    let shown = move |v: &Option<String>| v.as_deref().map(|t| dates::friendly_time(t, hour12)).unwrap_or_default();
    let entry = gtk::Entry::builder().text(shown(&value)).placeholder_text(placeholder).width_chars(8).css_classes(["field"]).build();
    let value = Rc::new(RefCell::new(value));
    let commit = {
        let value = value.clone();
        let on_change = Rc::new(on_change);
        move |e: &gtk::Entry| {
            let text = e.text().to_string();
            let next = match dates::parse_time(&text) {
                Some(t) => Some(Some(t)),
                None if optional && text.trim().is_empty() => Some(None),
                None => None,
            };
            if let Some(n) = next {
                if n != *value.borrow() {
                    *value.borrow_mut() = n.clone();
                    on_change(n);
                }
            }
            e.set_text(&shown(&value.borrow()));
        }
    };
    let c = commit.clone();
    entry.connect_activate(move |e| c(e));
    let focus = gtk::EventControllerFocus::new();
    let e = entry.clone();
    focus.connect_leave(move |_| commit(&e));
    // Focused with all of it selected, so typing replaces the time.
    let e = entry.clone();
    focus.connect_enter(move |_| {
        let e = e.clone();
        glib::idle_add_local_once(move || e.select_region(0, -1));
    });
    entry.add_controller(focus);
    entry
}


// ---- Icons (ui/Icon.svelte) ----------------------------------------------------

/// The line icons of ui/Icon.svelte: a 24 grid, round ends.
pub fn icon_path(name: &str) -> &'static str {
    match name {
        "tasks" => "M9 6h11M9 12h11M9 18h11M4 6l1 1 2-2M4 12l1 1 2-2M4 18l1 1 2-2",
        "calendar" => "M4 7a2 2 0 0 1 2-2h12a2 2 0 0 1 2 2v12a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2zM4 10h16M8 3v4M16 3v4",
        "plus" => "M12 5v14M5 12h14",
        "trash" => "M4 7h16M10 11v6M14 11v6M6 7l1 12a2 2 0 0 0 2 2h6a2 2 0 0 0 2-2l1-12M9 7V4h6v3",
        "link" => "M10 14a4 4 0 0 0 5.7 0l3-3a4 4 0 0 0-5.7-5.7l-1 1M14 10a4 4 0 0 0-5.7 0l-3 3a4 4 0 0 0 5.7 5.7l1-1",
        "close" => "M6 6l12 12M18 6L6 18",
        "check" => "M5 12l4 4 10-10",
        "download" => "M12 4v11M7 10l5 5 5-5M5 20h14",
        "lock" => "M6 11h12v9H6zM8 11V8a4 4 0 0 1 8 0v3",
        "chevron" => "M9 6l6 6-6 6",
        "back" | "left" => "M15 6l-6 6 6 6",
        "right" => "M9 6l6 6-6 6",
        "edit" => "M4 20h4L19 9l-4-4L4 16zM13 7l4 4",
        "settings" => "M12 15a3 3 0 1 0 0-6 3 3 0 0 0 0 6zM19.4 15a1.7 1.7 0 0 0 .3 1.8l.1.1a2 2 0 1 1-2.8 2.8l-.1-.1a1.7 1.7 0 0 0-1.8-.3 1.7 1.7 0 0 0-1 1.5V21a2 2 0 1 1-4 0v-.1a1.7 1.7 0 0 0-1.1-1.5 1.7 1.7 0 0 0-1.8.3l-.1.1a2 2 0 1 1-2.8-2.8l.1-.1a1.7 1.7 0 0 0 .3-1.8 1.7 1.7 0 0 0-1.5-1H3a2 2 0 1 1 0-4h.1a1.7 1.7 0 0 0 1.5-1.1 1.7 1.7 0 0 0-.3-1.8l-.1-.1a2 2 0 1 1 2.8-2.8l.1.1a1.7 1.7 0 0 0 1.8.3H9a1.7 1.7 0 0 0 1-1.5V3a2 2 0 1 1 4 0v.1a1.7 1.7 0 0 0 1 1.5 1.7 1.7 0 0 0 1.8-.3l.1-.1a2 2 0 1 1 2.8 2.8l-.1.1a1.7 1.7 0 0 0-.3 1.8V9a1.7 1.7 0 0 0 1.5 1H21a2 2 0 1 1 0 4h-.1a1.7 1.7 0 0 0-1.5 1z",
        "sun" => "M12 8a4 4 0 1 1 0 8 4 4 0 0 1 0-8zM12 2v2M12 20v2M2 12h2M20 12h2M5 5l1.4 1.4M17.6 17.6 19 19M5 19l1.4-1.4M17.6 6.4 19 5",
        "list" => "M8 6h12M8 12h12M8 18h12M4 6h.01M4 12h.01M4 18h.01",
        "inbox" => "M4 13l2-8h12l2 8M4 13v6h16v-6M4 13h5l1 2h4l1-2h5",
        "menu" => "M4 7h16M4 12h16M4 17h16",
        "people" => "M9 11a3.5 3.5 0 1 0 0-7 3.5 3.5 0 0 0 0 7zM2.5 20a6.5 6.5 0 0 1 13 0M16 4.2a3.5 3.5 0 0 1 0 6.6M18 14.3a6.5 6.5 0 0 1 3.5 5.7",
        "share" => "M12 15V3M8 7l4-4 4 4M7 11H5a1 1 0 0 0-1 1v8a1 1 0 0 0 1 1h14a1 1 0 0 0 1-1v-8a1 1 0 0 0-1-1h-2",
        "photo" => "M4 6a2 2 0 0 1 2-2h12a2 2 0 0 1 2 2v12a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2zM4 16l4.5-4.5 3.5 3.5 3-3 5 5M9 9h.01",
        "play" => "M8 5l11 7-11 7z",
        "bell" => "M6 16V11a6 6 0 0 1 12 0v5l1.5 2h-15zM10 20.5a2 2 0 0 0 4 0",
        "place" => "M12 21s-7-6.2-7-11.5a7 7 0 0 1 14 0C19 14.8 12 21 12 21zM12 12a2.5 2.5 0 1 0 0-5 2.5 2.5 0 0 0 0 5z",
        "video" => "M3 7a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v10a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2zM16 10l5-3v10l-5-3",
        "repeat" => "M17 2l4 4-4 4M3 11V9a3 3 0 013-3h15M7 22l-4-4 4-4M21 13v2a3 3 0 01-3 3H3",
        "clock" => "M12 21a9 9 0 1 0 0-18 9 9 0 0 0 0 18zM12 7v5l3 2",
        "notes" => "M5 7h14M5 12h14M5 17h9",
        "copy" => "M9 9h10v10a1 1 0 0 1-1 1H10a1 1 0 0 1-1-1zM5 15V5a1 1 0 0 1 1-1h9",
        "info" => "M12 21a9 9 0 1 0 0-18 9 9 0 0 0 0 18zM12 11v5M12 8h.01",
        "at" => "M16 12a4 4 0 1 1-8 0 4 4 0 0 1 8 0zM16 12v1.5a2.5 2.5 0 0 0 5 0V12a9 9 0 1 0-3.5 7.1",
        _ => "",
    }
}

/// The colour name for icons drawn in `--text-2` (see `icon`).
pub fn text2() -> &'static str {
    "text2"
}

/// An icon of ui/Icon.svelte, `size` pixels, its line `stroke` wide on the
/// 24 grid. `color`: "currentColor" (the colour around it, as on the web),
/// a token ("text", "text2", "text3", "accent", "danger"), a calendar colour
/// ("red", "blue", …) or any CSS colour ("#ffffff"). Tokens and calendar
/// colours follow light and dark as the theme changes, since the icon is
/// drawn each time in its widget's current colour.
pub fn icon(name: &str, size: i32, stroke: f64, color: &str) -> gtk::DrawingArea {
    let area = gtk::DrawingArea::builder().content_width(size).content_height(size).valign(gtk::Align::Center).halign(gtk::Align::Center).css_classes(["uuob-icon"]).build();
    let fixed = match color {
        "currentColor" | "" => None,
        "text" | "text2" | "text3" | "accent" | "danger" => {
            area.add_css_class(&format!("fg-{color}"));
            None
        }
        c if COLORS.contains(&c) => {
            area.add_css_class(&format!("uuob-fg-{c}"));
            None
        }
        c => gtk::gdk::RGBA::parse(c).ok(),
    };
    let d = icon_path(name);
    area.set_draw_func(move |a, cr, w, h| {
        let rgba = fixed.unwrap_or_else(|| a.color());
        cr.set_source_rgba(rgba.red() as f64, rgba.green() as f64, rgba.blue() as f64, rgba.alpha() as f64);
        let k = (w.min(h) as f64) / 24.0;
        cr.translate((w as f64 - 24.0 * k) / 2.0, (h as f64 - 24.0 * k) / 2.0);
        cr.scale(k, k);
        svg_path(cr, d);
        cr.set_line_width(stroke);
        cr.set_line_cap(gtk::cairo::LineCap::Round);
        cr.set_line_join(gtk::cairo::LineJoin::Round);
        let _ = cr.stroke();
    });
    area
}

/// Draws an icon with cairo's current colour, `size` across at (x, y): for
/// pictures drawn by hand, like a calendar's events.
pub fn draw_icon(cr: &gtk::cairo::Context, name: &str, x: f64, y: f64, size: f64, stroke: f64) {
    cr.save().ok();
    cr.translate(x, y);
    cr.scale(size / 24.0, size / 24.0);
    svg_path(cr, icon_path(name));
    cr.set_line_width(stroke);
    cr.set_line_cap(gtk::cairo::LineCap::Round);
    cr.set_line_join(gtk::cairo::LineJoin::Round);
    let _ = cr.stroke();
    cr.restore().ok();
}

/// Draws SVG path data (M L H V C S Q A Z, absolute and relative) as the
/// current cairo path. Arcs are circles, as all the icons' are.
fn svg_path(cr: &gtk::cairo::Context, d: &str) {
    let b = d.as_bytes();
    let mut i = 0;
    let skip = |i: &mut usize| {
        while *i < b.len() && (b[*i] == b' ' || b[*i] == b',') {
            *i += 1;
        }
    };
    let num = |i: &mut usize| -> f64 {
        skip(i);
        let start = *i;
        if *i < b.len() && (b[*i] == b'-' || b[*i] == b'+') {
            *i += 1;
        }
        let mut dot = false;
        while *i < b.len() && (b[*i].is_ascii_digit() || (b[*i] == b'.' && !dot)) {
            if b[*i] == b'.' {
                dot = true;
            }
            *i += 1;
        }
        d[start..*i].parse().unwrap_or(0.0)
    };
    // An arc's flags are single digits ("013-3" is 0, 1 and 3).
    let flag = |i: &mut usize| -> bool {
        skip(i);
        let f = *i < b.len() && b[*i] == b'1';
        if *i < b.len() {
            *i += 1;
        }
        f
    };
    let has_num = |i: &mut usize| -> bool {
        skip(i);
        *i < b.len() && (b[*i].is_ascii_digit() || b[*i] == b'-' || b[*i] == b'.' || b[*i] == b'+')
    };
    let (mut cx, mut cy, mut sx, mut sy, mut px, mut py) = (0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    let mut cmd = b'M';
    loop {
        skip(&mut i);
        if i >= b.len() {
            break;
        }
        if b[i].is_ascii_alphabetic() {
            cmd = b[i];
            i += 1;
        } else if !has_num(&mut i) {
            i += 1;
            continue;
        }
        let rel = cmd.is_ascii_lowercase();
        let (bx, by) = if rel { (cx, cy) } else { (0.0, 0.0) };
        match cmd.to_ascii_uppercase() {
            b'M' => {
                cx = bx + num(&mut i);
                cy = by + num(&mut i);
                cr.move_to(cx, cy);
                (sx, sy) = (cx, cy);
                cmd = if rel { b'l' } else { b'L' };
            }
            b'L' => {
                cx = bx + num(&mut i);
                cy = by + num(&mut i);
                cr.line_to(cx, cy);
            }
            b'H' => {
                cx = (if rel { cx } else { 0.0 }) + num(&mut i);
                cr.line_to(cx, cy);
            }
            b'V' => {
                cy = (if rel { cy } else { 0.0 }) + num(&mut i);
                cr.line_to(cx, cy);
            }
            b'C' => {
                let (x1, y1) = (bx + num(&mut i), by + num(&mut i));
                let (x2, y2) = (bx + num(&mut i), by + num(&mut i));
                cx = bx + num(&mut i);
                cy = by + num(&mut i);
                cr.curve_to(x1, y1, x2, y2, cx, cy);
                (px, py) = (x2, y2);
            }
            b'S' => {
                let (x1, y1) = (2.0 * cx - px, 2.0 * cy - py);
                let (x2, y2) = (bx + num(&mut i), by + num(&mut i));
                cx = bx + num(&mut i);
                cy = by + num(&mut i);
                cr.curve_to(x1, y1, x2, y2, cx, cy);
                (px, py) = (x2, y2);
            }
            b'Q' => {
                let (qx, qy) = (bx + num(&mut i), by + num(&mut i));
                let (ex, ey) = (bx + num(&mut i), by + num(&mut i));
                cr.curve_to(cx + 2.0 / 3.0 * (qx - cx), cy + 2.0 / 3.0 * (qy - cy), ex + 2.0 / 3.0 * (qx - ex), ey + 2.0 / 3.0 * (qy - ey), ex, ey);
                (cx, cy, px, py) = (ex, ey, qx, qy);
            }
            b'A' => {
                let r = num(&mut i);
                let _ry = num(&mut i);
                let _rot = num(&mut i);
                let large = flag(&mut i);
                let sweep = flag(&mut i);
                let (ex, ey) = (bx + num(&mut i), by + num(&mut i));
                arc_to(cr, (cx, cy), (ex, ey), r, large, sweep);
                (cx, cy) = (ex, ey);
            }
            b'Z' => {
                cr.close_path();
                (cx, cy) = (sx, sy);
            }
            _ => {}
        }
    }
}

/// A circular arc from `a` to `b` with SVG's flags.
fn arc_to(cr: &gtk::cairo::Context, a: (f64, f64), b: (f64, f64), radius: f64, large: bool, sweep: bool) {
    let (dx, dy) = ((b.0 - a.0) / 2.0, (b.1 - a.1) / 2.0);
    let d = (dx * dx + dy * dy).sqrt();
    if d == 0.0 {
        return;
    }
    let r = radius.max(d);
    let h = (r * r - d * d).max(0.0).sqrt();
    let sign = if large == sweep { -1.0 } else { 1.0 };
    let (mx, my) = ((a.0 + b.0) / 2.0, (a.1 + b.1) / 2.0);
    let (ox, oy) = (mx + sign * h * (-dy / d), my + sign * h * (dx / d));
    let (s, e) = ((a.1 - oy).atan2(a.0 - ox), (b.1 - oy).atan2(b.0 - ox));
    // SVG's sweep 1 goes clockwise on screen, which is cairo's positive direction.
    if sweep { cr.arc(ox, oy, r, s, e) } else { cr.arc_negative(ox, oy, r, s, e) }
}

/// `.sheet` for a question: a title, a line of text and big pill buttons side
/// by side, like the web version's confirmations. Each button is
/// (label, extra classes such as "primary" or "danger", the answer it gives);
/// `cancel` adds a quiet Cancel below. Present it with `.present(Some(window))`.
pub fn confirm_sheet(title: &str, text: &str, buttons: &[(&str, &[&str], &str)], cancel: bool, on_pick: impl Fn(&str) + 'static) -> adw::Dialog {
    let d = adw::Dialog::builder().content_width(460).build();
    let col = gtk::Box::builder().orientation(gtk::Orientation::Vertical).spacing(8).margin_top(28).margin_bottom(28).margin_start(28).margin_end(28).build();
    col.append(&gtk::Label::builder().label(title).xalign(0.0).wrap(true).css_classes(["web-title"]).build());
    col.append(&gtk::Label::builder().label(text).xalign(0.0).wrap(true).css_classes(["secondary"]).build());
    let actions = gtk::Box::builder().spacing(10).homogeneous(true).margin_top(14).build();
    let on_pick = Rc::new(on_pick);
    for (label, classes, answer) in buttons {
        let mut all = vec!["lg"];
        all.extend_from_slice(classes);
        let b = btn(label, None, &all);
        let (d, f, answer) = (d.clone(), on_pick.clone(), answer.to_string());
        b.connect_clicked(move |_| {
            d.close();
            f(&answer);
        });
        actions.append(&b);
    }
    col.append(&actions);
    if cancel {
        let c = btn("Cancel", None, &["ghost"]);
        let d2 = d.clone();
        c.connect_clicked(move |_| {
            d2.close();
        });
        col.append(&c);
    }
    d.set_child(Some(&col));
    d
}

/// `.btn`: a pill with an optional icon before its text. `kind`: "", "primary", "danger", "ghost"; add "lg" for the big one.
pub fn btn(text: &str, icon_name: Option<&str>, classes: &[&str]) -> gtk::Button {
    let b = gtk::Button::builder().css_classes(["btn"]).build();
    for c in classes {
        b.add_css_class(c);
    }
    match icon_name {
        Some(i) => {
            // The icon takes the button's own text colour, as on the web.
            let color = if classes.contains(&"plain") { "text2" } else { "currentColor" };
            let row = gtk::Box::builder().spacing(8).halign(gtk::Align::Center).build();
            row.append(&icon(i, 18, 2.0, color));
            row.append(&gtk::Label::new(Some(text)));
            b.set_child(Some(&row));
        }
        None => b.set_label(text),
    }
    b
}

/// `.icon-btn`: a round 36 button with a grey icon.
pub fn icon_btn(name: &str, size: i32, tooltip: &str) -> gtk::Button {
    gtk::Button::builder().child(&icon(name, size, 1.75, "currentColor")).tooltip_text(tooltip).css_classes(["icon-btn"]).valign(gtk::Align::Center).build()
}
