//! What needs the client: the connect screen (until the app is connected to
//! UUOB), the update button, Settings for all apps and the time field.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use relm4::{adw, gtk, gtk::glib};
use uuob_client::{Client, Event, State};

use crate::*;

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
    // A click outside closes it, as a menu does.
    crate::close_on_outside_click(&dialog);
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

