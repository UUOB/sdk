//! The keys every app answers to (docs/design.md), and their list.

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
        let mut rows = vec![];
        if self.settings.is_some() {
            rows.push(("Settings".into(), "Ctrl+,".into()));
        }
        if let Some((t, _)) = &self.new {
            rows.push((t.clone(), "Ctrl+N".into()));
        }
        if self.find.is_some() {
            rows.push(("Find".into(), "Ctrl+F".into()));
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
        rows.push(("Keyboard shortcuts".into(), "Ctrl+?".into()));
        rows
    }

    /// Puts them on the window: each as a window action with its accelerator.
    pub fn install(self, window: &gtk::ApplicationWindow) {
        let app = window.application();
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
        {
            let rows = rows.clone();
            let win = window.clone();
            all.push(("shortcuts".into(), "<Control>question".into(), Rc::new(move || shortcuts_sheet(&rows.borrow()).present(Some(&win)))));
        }
        for (name, accel, f) in all {
            let action = gio::SimpleAction::new(&name, None);
            action.connect_activate(move |_, _| f());
            window.add_action(&action);
            if let Some(app) = &app {
                app.set_accels_for_action(&format!("win.{name}"), &[accel.as_str()]);
            }
        }
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
