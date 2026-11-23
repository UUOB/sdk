//! The keys every app answers to (docs/design.md), their list, and Linear's
//! habits: Ctrl+K for a palette of every action, single letters where no
//! field is typing (C new, / find, ? the list).

use std::cell::RefCell;
use std::rc::Rc;

use relm4::adw::prelude::*;
use relm4::{adw, gtk, gtk::gio};

use crate::detail_sheet;

/// The keys every app answers to (docs/design.md), with Ctrl for ⌘:
/// Ctrl+, Settings · Ctrl+N New · Ctrl+F Find · Ctrl+Alt+S Toggle sidebar ·
/// Ctrl+R Refresh · Ctrl+? Keyboard shortcuts; plus the app's own `extra`
/// (name, accelerator like "<Control>1" or "t", what it does).
#[derive(Default)]
pub struct Shortcuts {
    pub settings: Option<Rc<dyn Fn()>>,
    pub new: Option<(String, Rc<dyn Fn()>)>,
    pub find: Option<Rc<dyn Fn()>>,
    pub toggle_sidebar: Option<Rc<dyn Fn()>>,
    pub refresh: Option<Rc<dyn Fn()>>,
    pub extra: Vec<(String, String, Rc<dyn Fn()>)>,
}

impl Shortcuts {
    /// Every shortcut as (name, keys), for the list.
    pub fn rows(&self) -> Vec<(String, String)> {
        let mut rows = vec![("Command palette".to_string(), "Ctrl+K".to_string())];
        if self.settings.is_some() {
            rows.push(("Settings".into(), "Ctrl+,".into()));
        }
        if let Some((t, _)) = &self.new {
            rows.push((t.clone(), "Ctrl+N or C".into()));
        }
        if self.find.is_some() {
            rows.push(("Find".into(), "Ctrl+F or /".into()));
        }
        if self.toggle_sidebar.is_some() {
            rows.push(("Toggle sidebar".into(), "Ctrl+Alt+S".into()));
        }
        if self.refresh.is_some() {
            rows.push(("Refresh".into(), "Ctrl+R".into()));
        }
        for (t, accel, _) in &self.extra {
            rows.push((t.clone(), accel_label(accel)));
        }
        rows.push(("Save".into(), "Ctrl+Enter".into()));
        rows.push(("Close or cancel".into(), "Esc".into()));
        rows.push(("Keyboard shortcuts".into(), "Ctrl+? or ?".into()));
        rows
    }

    /// Every action with its keys, for the palette.
    fn actions(&self) -> Vec<(String, String, Rc<dyn Fn()>)> {
        let mut out: Vec<(String, String, Rc<dyn Fn()>)> = vec![];
        if let Some((t, f)) = &self.new {
            out.push((t.clone(), "Ctrl+N  C".into(), f.clone()));
        }
        if let Some(f) = &self.find {
            out.push(("Find".into(), "Ctrl+F  /".into(), f.clone()));
        }
        for (t, accel, f) in &self.extra {
            out.push((t.clone(), accel_label(accel), f.clone()));
        }
        if let Some(f) = &self.settings {
            out.push(("Settings".into(), "Ctrl+,".into(), f.clone()));
        }
        if let Some(f) = &self.toggle_sidebar {
            out.push(("Toggle sidebar".into(), "Ctrl+Alt+S".into(), f.clone()));
        }
        if let Some(f) = &self.refresh {
            out.push(("Refresh".into(), "Ctrl+R".into(), f.clone()));
        }
        out
    }

    /// Puts them on the window: each as a window action with its accelerator.
    pub fn install(self, window: &gtk::ApplicationWindow) {
        let rows = Rc::new(RefCell::new(vec![]));
        let mut all: Vec<(String, String, Rc<dyn Fn()>)> = vec![];
        if let Some(f) = &self.settings {
            all.push(("settings".into(), "<Control>comma".into(), f.clone()));
        }
        if let Some((_, f)) = &self.new {
            all.push(("new".into(), "<Control>n".into(), f.clone()));
        }
        if let Some(f) = &self.find {
            all.push(("find".into(), "<Control>f".into(), f.clone()));
        }
        if let Some(f) = &self.toggle_sidebar {
            all.push(("toggle-sidebar".into(), "<Control><Alt>s".into(), f.clone()));
        }
        if let Some(f) = &self.refresh {
            all.push(("refresh".into(), "<Control>r".into(), f.clone()));
        }
        for (i, (_, accel, f)) in self.extra.iter().enumerate() {
            all.push((format!("extra-{i}"), accel.clone(), f.clone()));
        }
        *rows.borrow_mut() = self.rows();
        let palette_actions = Rc::new(self.actions());
        {
            let win = window.clone();
            let acts = palette_actions.clone();
            let rows = rows.clone();
            let mut with_list: Vec<(String, String, Rc<dyn Fn()>)> = (*acts).clone();
            let w2 = window.clone();
            with_list.push(("Keyboard shortcuts".into(), "Ctrl+?".into(), Rc::new(move || shortcuts_sheet(&rows.borrow()).present(Some(&w2)))));
            let with_list = Rc::new(with_list);
            all.push(("palette".into(), "<Control>k".into(), Rc::new(move || command_palette(&with_list).present(Some(&win)))));
        }
        // Linear's single letters, when no field takes the key (C new, / find, ? the list).
        {
            let letters = gtk::EventControllerKey::new();
            letters.set_propagation_phase(gtk::PropagationPhase::Bubble);
            let new = self.new.as_ref().map(|n| n.1.clone());
            let find = self.find.clone();
            let (rows, win) = (rows.clone(), window.clone());
            letters.connect_key_pressed(move |_, key, _, mods| {
                if mods.intersects(gtk::gdk::ModifierType::CONTROL_MASK | gtk::gdk::ModifierType::ALT_MASK) {
                    return gtk::glib::Propagation::Proceed;
                }
                match key {
                    gtk::gdk::Key::c => if let Some(f) = &new { f() } else { return gtk::glib::Propagation::Proceed },
                    gtk::gdk::Key::slash => if let Some(f) = &find { f() } else { return gtk::glib::Propagation::Proceed },
                    gtk::gdk::Key::question => shortcuts_sheet(&rows.borrow()).present(Some(&win)),
                    _ => return gtk::glib::Propagation::Proceed,
                }
                gtk::glib::Propagation::Stop
            });
            window.add_controller(letters);
        }
        {
            let rows = rows.clone();
            let win = window.clone();
            all.push(("shortcuts".into(), "<Control>question".into(), Rc::new(move || shortcuts_sheet(&rows.borrow()).present(Some(&win)))));
        }
        // A controller on the window itself: it works before the window has its application, and in every dialog over it.
        let keys = gtk::ShortcutController::new();
        keys.set_scope(gtk::ShortcutScope::Global);
        for (name, accel, f) in all {
            let action = gio::SimpleAction::new(&name, None);
            let g = f.clone();
            action.connect_activate(move |_, _| g());
            window.add_action(&action);
            if let Some(trigger) = gtk::ShortcutTrigger::parse_string(&accel) {
                keys.add_shortcut(gtk::Shortcut::new(
                    Some(trigger),
                    Some(gtk::CallbackAction::new(move |_, _| {
                        f();
                        gtk::glib::Propagation::Stop
                    })),
                ));
            }
        }
        window.add_controller(keys);
    }
}

/// "<Control>1" → "Ctrl+1", "t" → "T".
fn accel_label(accel: &str) -> String {
    let mut s = accel.replace("<Control>", "Ctrl+").replace("<Alt>", "Alt+").replace("<Shift>", "Shift+");
    s = s.replace("Left", "←").replace("Right", "→").replace("space", "Space").replace("Delete", "Delete");
    if let Some(last) = s.rsplit('+').next().filter(|l| l.len() == 1) {
        let up = last.to_uppercase();
        s = format!("{}{}", &s[..s.len() - last.len()], up);
    }
    s
}

/// The list of shortcuts, in the detail card's style.
pub fn shortcuts_sheet(rows: &[(String, String)]) -> adw::Dialog {
    let (d, body) = detail_sheet("Keyboard shortcuts", &[]);
    body.set_margin_top(10);
    for (i, (name, keys)) in rows.iter().enumerate() {
        let r = crate::icon_setting_row(None, name, &gtk::Label::builder().label(keys).css_classes(["key-cap"]).build());
        if i + 1 == rows.len() {
            r.add_css_class("last");
        }
        body.append(&r);
    }
    d
}

/// Linear's command palette: type to narrow every action down, ↑ ↓ to pick, Enter to run.
pub fn command_palette(actions: &[(String, String, Rc<dyn Fn()>)]) -> adw::Dialog {
    let d = adw::Dialog::builder().content_width(520).build();
    let col = gtk::Box::builder().orientation(gtk::Orientation::Vertical).build();
    let search = gtk::SearchEntry::builder().placeholder_text("Type a command…").margin_start(14).margin_end(14).margin_top(12).margin_bottom(12).build();
    col.append(&search);
    col.append(&gtk::Separator::new(gtk::Orientation::Horizontal));
    let list = gtk::ListBox::builder().selection_mode(gtk::SelectionMode::Browse).css_classes(["navigation-sidebar"]).margin_start(6).margin_end(6).margin_top(6).margin_bottom(6).build();
    let acts: Rc<Vec<(String, String, Rc<dyn Fn()>)>> = Rc::new(actions.to_vec());
    for (name, keys, _) in acts.iter() {
        let row = gtk::Box::builder().spacing(10).margin_start(6).margin_end(6).margin_top(6).margin_bottom(6).build();
        row.append(&gtk::Label::builder().label(name).xalign(0.0).hexpand(true).build());
        row.append(&gtk::Label::builder().label(keys).css_classes(["dim-label", "caption"]).build());
        list.append(&row);
    }
    let query = Rc::new(RefCell::new(String::new()));
    {
        let (q, acts) = (query.clone(), acts.clone());
        list.set_filter_func(move |row| {
            let q = q.borrow().to_lowercase();
            q.is_empty() || acts.get(row.index() as usize).is_some_and(|a| a.0.to_lowercase().contains(&q))
        });
    }
    let first_visible = {
        let list = list.clone();
        move || {
            let mut i = 0;
            while let Some(r) = list.row_at_index(i) {
                if r.is_child_visible() {
                    return Some(r);
                }
                i += 1;
            }
            None
        }
    };
    {
        let (q, list, fv) = (query.clone(), list.clone(), first_visible.clone());
        search.connect_search_changed(move |e| {
            *q.borrow_mut() = e.text().to_string();
            list.invalidate_filter();
            list.select_row(fv().as_ref());
        });
    }
    let run = {
        let (d, acts) = (d.clone(), acts.clone());
        Rc::new(move |i: i32| {
            if let Some(a) = acts.get(i as usize) {
                d.close();
                let f = a.2.clone();
                gtk::glib::idle_add_local_once(move || f());
            }
        })
    };
    // Esc leaves (the search field would take it to clear itself).
    {
        let d = d.clone();
        search.connect_stop_search(move |_| {
            d.close();
        });
    }
    {
        let r = run.clone();
        list.connect_row_activated(move |_, row| r(row.index()));
        let (r, lst, fv) = (run.clone(), list.clone(), first_visible.clone());
        search.connect_activate(move |_| {
            if let Some(row) = lst.selected_row().or_else(|| fv()) {
                r(row.index());
            }
        });
        // ↓ from the search moves into the list.
        let l2 = list.clone();
        search.connect_next_match(move |_| {
            l2.child_focus(gtk::DirectionType::Down);
        });
    }
    list.select_row(list.row_at_index(0).as_ref());
    let scroll = gtk::ScrolledWindow::builder().hscrollbar_policy(gtk::PolicyType::Never).propagate_natural_height(true).max_content_height(340).child(&list).build();
    col.append(&scroll);
    d.set_child(Some(&col));
    d.set_focus(Some(&search));
    d
}
