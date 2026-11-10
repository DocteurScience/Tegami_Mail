//! The "Then" part of the filter editor (screen 38): one row per action a
//! rule takes (Move, Mark as read, Forward, Delete), added from a menu and
//! removed with the row's button.

use std::cell::RefCell;
use std::rc::Rc;

use adw::prelude::*;

use crate::config::{parse_forward_addresses, FilterRule};
use crate::i18n::{i18n, i18n_f};

/// The kinds of action a rule can hold, in the order the menu lists them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ActionKind {
    Forward,
    MarkRead,
    Move,
    Delete,
}

impl ActionKind {
    const ALL: [ActionKind; 4] = [
        ActionKind::Move,
        ActionKind::MarkRead,
        ActionKind::Forward,
        ActionKind::Delete,
    ];

    /// The menu entry for this kind.
    fn label(self) -> String {
        match self {
            ActionKind::Forward => i18n("Forward to…"),
            ActionKind::MarkRead => i18n("Mark as read"),
            ActionKind::Move => i18n("Move to…"),
            ActionKind::Delete => i18n("Delete"),
        }
    }
}

/// What the actions of a rule add to its line in the filters list, after the
/// destination folder: `forwards to a@x.org`, `marked as read`, `deleted`.
pub(super) fn action_summary(rule: &FilterRule) -> Vec<String> {
    let mut parts = Vec::new();
    if !rule.forward_to.is_empty() {
        parts.push(i18n_f(
            "forwards to {to}",
            &[("to", &rule.forward_to.join(", "))],
        ));
    }
    if rule.mark_read {
        parts.push(i18n("marked as read"));
    }
    if rule.delete {
        parts.push(i18n("deleted for good"));
    }
    parts
}

/// One action's row: what kind it is, and the row that holds its value.
struct ActionRow {
    kind: ActionKind,
    row: gtk::Widget,
    forward: Option<adw::EntryRow>,
    dest: Option<adw::ComboRow>,
    mark: Option<adw::SwitchRow>,
}

/// The folders the Move row offers: the chosen account's, as display names
/// and server paths side by side.
#[derive(Default)]
struct Destinations {
    names: Vec<String>,
    paths: Vec<String>,
}

/// The "Then" group of the filter editor (screen 38): one row per action,
/// named by what it does ("Move to" and its folder, "Mark as read"), and
/// "Add action" last, offering only what the rule does not do yet.
#[derive(Clone)]
pub(super) struct ActionList {
    /// Titled "Then"; holds the list.
    pub(super) group: adw::PreferencesGroup,
    list: gtk::ListBox,
    rows: Rc<RefCell<Vec<ActionRow>>>,
    dests: Rc<RefCell<Destinations>>,
    changed: Rc<dyn Fn()>,
    own: Vec<(String, String)>,
}

impl ActionList {
    /// An empty list; `changed` is called whenever a row is added, removed
    /// or edited. `own` are the accounts' (name, address) pairs, for the
    /// address suggestions of the Forward row.
    pub(super) fn new(changed: Rc<dyn Fn()>, own: Vec<(String, String)>) -> Self {
        let group = adw::PreferencesGroup::new();
        group.set_title(&i18n("Then"));
        let list_box = gtk::ListBox::new();
        list_box.add_css_class("boxed-list");
        list_box.set_selection_mode(gtk::SelectionMode::None);
        group.add(&list_box);
        let add_row = super::pages::add_row(&i18n("Add action"));
        list_box.append(&add_row);
        let list = ActionList {
            group,
            list: list_box,
            rows: Default::default(),
            dests: Default::default(),
            changed,
            own,
        };
        let adder = list.clone();
        add_row.connect_activated(move |row| adder.menu(row.upcast_ref()));
        list
    }

    /// The kinds "Add action" offers: those the rule does not take yet,
    /// Move and Delete each ruling the other out.
    fn offered(&self) -> Vec<ActionKind> {
        let have: Vec<ActionKind> = self.rows.borrow().iter().map(|r| r.kind).collect();
        ActionKind::ALL
            .into_iter()
            .filter(|k| !have.contains(k))
            .filter(|k| match k {
                ActionKind::Move => !have.contains(&ActionKind::Delete),
                ActionKind::Delete => !have.contains(&ActionKind::Move),
                _ => true,
            })
            .collect()
    }

    fn menu(&self, anchor: &gtk::Widget) {
        let entries: Vec<_> = self
            .offered()
            .into_iter()
            .map(|k| {
                let list = self.clone();
                crate::ui::context_menu::MenuEntry::new(k.label(), move || list.add(k, ""))
            })
            .collect();
        if entries.is_empty() {
            return;
        }
        let y = f64::from(anchor.height());
        let x = f64::from(anchor.width()).min(400.0) / 2.0;
        crate::ui::context_menu::show_context_menu(anchor, x, y, vec![entries]);
    }

    /// Offer the folders of the chosen account to the Move row.
    pub(super) fn set_destinations(&self, folders: &[(String, String)]) {
        {
            let mut d = self.dests.borrow_mut();
            d.names = folders.iter().map(|(_, n)| n.clone()).collect();
            d.paths = folders.iter().map(|(p, _)| p.clone()).collect();
        }
        for row in self.rows.borrow().iter() {
            if let Some(dest) = &row.dest {
                self.fill_dest(dest, "");
            }
        }
    }

    /// Put the folder list in the Move row, keeping `keep` (a path) selected.
    fn fill_dest(&self, dest: &adw::ComboRow, keep: &str) {
        let d = self.dests.borrow();
        let names: Vec<&str> = d.names.iter().map(String::as_str).collect();
        dest.set_model(Some(&gtk::StringList::new(&names)));
        if let Some(i) = d.paths.iter().position(|p| p == keep) {
            dest.set_selected(i as u32);
        }
    }

    /// Fill the list from `rule`: one row per action it takes.
    pub(super) fn load(&self, rule: &FilterRule) {
        if !rule.dest_path.is_empty() {
            self.add(ActionKind::Move, &rule.dest_path);
        }
        if rule.mark_read {
            self.add(ActionKind::MarkRead, "");
        }
        if !rule.forward_to.is_empty() {
            self.add(ActionKind::Forward, &rule.forward_to.join(", "));
        }
        if rule.delete {
            self.add(ActionKind::Delete, "");
        }
    }

    /// A small button that takes the action out of the rule.
    fn remove_button(&self) -> gtk::Button {
        let trash = gtk::Button::from_icon_name("user-trash-symbolic");
        trash.add_css_class("flat");
        trash.set_valign(gtk::Align::Center);
        trash.set_tooltip_text(Some(&i18n("Remove action")));
        trash.update_property(&[gtk::accessible::Property::Label(&i18n("Remove action"))]);
        trash
    }

    /// Add a row of `kind`, holding `value` (the addresses of a Forward,
    /// the path of a Move), before "Add action".
    fn add(&self, kind: ActionKind, value: &str) {
        let trash = self.remove_button();
        let mut item = ActionRow {
            kind,
            row: gtk::Label::new(None).upcast(),
            forward: None,
            dest: None,
            mark: None,
        };
        match kind {
            ActionKind::Forward => {
                let forward = adw::EntryRow::new();
                forward.set_title(&i18n("Forward to (addresses, separated by commas)"));
                forward.set_text(value);
                super::address_suggest::attach(&forward, self.own.clone());
                forward.add_suffix(&trash);
                let changed = self.changed.clone();
                forward.connect_changed(move |_| changed());
                item.row = forward.clone().upcast();
                item.forward = Some(forward);
            }
            ActionKind::MarkRead => {
                // A switch, as screen 38 draws it; off leaves the mail as
                // it is, and the row's button takes it out of the rule.
                let mark = adw::SwitchRow::new();
                mark.set_title(&i18n("Mark as read"));
                mark.set_active(true);
                mark.add_suffix(&trash);
                let changed = self.changed.clone();
                mark.connect_active_notify(move |_| changed());
                item.row = mark.clone().upcast();
                item.mark = Some(mark);
            }
            ActionKind::Move => {
                let dest = adw::ComboRow::new();
                dest.set_title(&i18n("Move to"));
                self.fill_dest(&dest, value);
                dest.add_suffix(&trash);
                let changed = self.changed.clone();
                dest.connect_selected_notify(move |_| changed());
                item.row = dest.clone().upcast();
                item.dest = Some(dest);
            }
            ActionKind::Delete => {
                let delete = adw::ActionRow::new();
                delete.set_title(&i18n("Delete"));
                delete.set_subtitle(&i18n("Permanently deletes the message"));
                delete.add_suffix(&trash);
                item.row = delete.upcast();
            }
        }
        let row = item.row.clone();
        let at = self.rows.borrow().len() as i32;
        self.list.insert(&row, at);
        {
            let list = self.clone();
            let row = row.clone();
            trash.connect_clicked(move |_| {
                list.list.remove(&row);
                list.rows.borrow_mut().retain(|r| r.row != row);
                (list.changed)();
            });
        }
        self.rows.borrow_mut().push(item);
        (self.changed)();
    }

    /// The actions as the rows stand, or `None` while they cannot be
    /// saved: a Move with no folder, a Forward with no address or with one
    /// that is not an address (that row is marked).
    pub(super) fn read(&self) -> Option<Actions> {
        let mut out = Actions::default();
        let mut valid = true;
        let dests = self.dests.borrow();
        for row in self.rows.borrow().iter() {
            match row.kind {
                ActionKind::Forward => {
                    let Some(forward) = &row.forward else {
                        continue;
                    };
                    match parse_forward_addresses(&forward.text()) {
                        Ok(to) if !to.is_empty() => {
                            forward.remove_css_class("error");
                            out.forward_to = to;
                        }
                        Ok(_) => valid = false,
                        Err(_) => {
                            forward.add_css_class("error");
                            valid = false;
                        }
                    }
                }
                ActionKind::MarkRead => {
                    out.mark_read = row.mark.as_ref().is_some_and(|m| m.is_active());
                }
                ActionKind::Move => {
                    let selected = row.dest.as_ref().map_or(u32::MAX, |d| d.selected());
                    match dests.paths.get(selected as usize) {
                        Some(path) => out.dest_path = path.clone(),
                        None => valid = false,
                    }
                }
                ActionKind::Delete => out.delete = true,
            }
        }
        if out.delete && !out.dest_path.is_empty() {
            valid = false;
        }
        valid.then_some(out)
    }
}

/// What the rows of an [`ActionList`] add up to.
#[derive(Default, Debug, PartialEq, Eq)]
pub(super) struct Actions {
    pub(super) forward_to: Vec<String>,
    pub(super) mark_read: bool,
    pub(super) dest_path: String,
    pub(super) delete: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{FilterField, FilterMatch};

    fn rule() -> FilterRule {
        let mut r = FilterRule::draft_from_message("a@b.c", "x@y.z", "");
        r.field = FilterField::FromAddress;
        r.matcher = FilterMatch::Equals;
        r.dest_path = String::new();
        r
    }

    #[test]
    fn summary_lists_every_action() {
        let mut r = rule();
        assert!(action_summary(&r).is_empty());
        r.forward_to = vec!["a@x.org".into(), "b@x.org".into()];
        r.mark_read = true;
        r.delete = true;
        assert_eq!(
            action_summary(&r),
            vec![
                "forwards to a@x.org, b@x.org".to_string(),
                "marked as read".to_string(),
                "deleted for good".to_string()
            ]
        );
    }
}
