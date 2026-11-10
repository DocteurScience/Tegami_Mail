//! The criteria the filter and smart-folder editors share (#192, #417;
//! screens 38 and 40): one boxed list holding the all-or-any row, a row per
//! criterion that says what it looks for ("Subject contains", the text in
//! the row), and "Add criterion" last. A criterion's ⋯ button changes how
//! it matches or removes it. The rare fields are added from an Advanced
//! row the editor places itself ([`Criteria::add_row`]).

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use adw::prelude::*;

use super::AccountsWindow;
use crate::config::FilterMatch;
use crate::i18n::{i18n, i18n_f};
use crate::ui::context_menu::{show_context_menu, MenuEntry};

/// A form page's scrolling body: groups stacked the way an
/// `AdwPreferencesPage` stacks them, in a box of our own.
pub(super) fn form_scaffold() -> (gtk::Box, gtk::ScrolledWindow) {
    let page_box = gtk::Box::new(gtk::Orientation::Vertical, 24);
    page_box.set_margin_top(24);
    page_box.set_margin_bottom(24);
    page_box.set_margin_start(12);
    page_box.set_margin_end(12);
    let clamp = adw::Clamp::new();
    clamp.set_maximum_size(600);
    clamp.set_tightening_threshold(400);
    clamp.set_child(Some(&page_box));
    let scrolled = gtk::ScrolledWindow::new();
    scrolled.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
    scrolled.set_vexpand(true);
    scrolled.set_child(Some(&clamp));
    (page_box, scrolled)
}

/// A field a criterion can look at, as the editor that owns it names it.
pub(super) struct FieldSpec {
    pub(super) label: String,
    /// Takes text to match (not a contact check).
    pub(super) text: bool,
    /// Always a "contains" match (the message body: that is the only
    /// search a server offers, #191).
    pub(super) contains_only: bool,
}

/// How many of an editor's fixed rows hold a criterion now.
type FixedCount = Rc<dyn Fn() -> usize>;

/// One criterion's row and what it holds.
struct Item {
    field: usize,
    matcher: Rc<Cell<FilterMatch>>,
    row: gtk::Widget,
    entry: Option<adw::EntryRow>,
}

/// The criteria of one editor.
#[derive(Clone)]
pub(super) struct Criteria {
    pub(super) group: adw::PreferencesGroup,
    list: gtk::ListBox,
    pub(super) combine: adw::ComboRow,
    specs: Rc<Vec<FieldSpec>>,
    items: Rc<RefCell<Vec<Item>>>,
    /// Rows the editor puts first in the list (a smart folder's Reading,
    /// Starred…), and how many of them hold a criterion now.
    fixed: Rc<Cell<usize>>,
    fixed_set: Rc<RefCell<Option<FixedCount>>>,
    changed: Rc<dyn Fn()>,
    activate: Rc<dyn Fn()>,
}

impl Criteria {
    /// An empty list titled `title`. `common` are the fields "Add
    /// criterion" offers; `changed` runs whenever a criterion is added,
    /// removed or edited, and `activate` on Enter in a criterion's text.
    pub(super) fn new(
        title: &str,
        description: Option<&str>,
        specs: Vec<FieldSpec>,
        common: Vec<usize>,
        changed: Rc<dyn Fn()>,
        activate: Rc<dyn Fn()>,
    ) -> Self {
        let group = adw::PreferencesGroup::new();
        group.set_title(title);
        group.set_description(description);
        let list = gtk::ListBox::new();
        list.add_css_class("boxed-list");
        list.set_selection_mode(gtk::SelectionMode::None);
        group.add(&list);
        let combine = adw::ComboRow::new();
        combine.set_title(&i18n("Criteria"));
        combine.set_model(Some(&gtk::StringList::new(&[
            i18n("All must match").as_str(),
            i18n("Any one may match").as_str(),
        ])));
        list.append(&combine);
        let add = super::pages::add_row(&i18n("Add criterion"));
        list.append(&add);
        let criteria = Criteria {
            group,
            list,
            combine,
            specs: Rc::new(specs),
            items: Default::default(),
            fixed: Rc::new(Cell::new(0)),
            fixed_set: Default::default(),
            changed,
            activate,
        };
        {
            let c = criteria.clone();
            add.connect_activated(move |row| c.field_menu(row.upcast_ref(), &common));
        }
        {
            let changed = criteria.changed.clone();
            criteria.combine.connect_selected_notify(move |_| changed());
        }
        criteria.refresh();
        criteria
    }

    /// Put `rows` at the head of the list, under the all-or-any row, for
    /// criteria the editor keeps as fixed rows; `set` counts those that
    /// hold one, and is asked again on [`Criteria::refresh`].
    pub(super) fn set_fixed(&self, rows: &[gtk::Widget], set: FixedCount) {
        for (i, row) in rows.iter().enumerate() {
            self.list.insert(row, i as i32 + 1);
        }
        self.fixed.set(rows.len());
        *self.fixed_set.borrow_mut() = Some(set);
        self.refresh();
    }

    /// An "add" row for another place (Advanced), whose menu offers `fields`.
    pub(super) fn add_row(&self, title: &str, fields: Vec<usize>) -> adw::ActionRow {
        let row = super::pages::add_row(title);
        let c = self.clone();
        row.connect_activated(move |row| c.field_menu(row.upcast_ref(), &fields));
        row
    }

    /// The menu of fields to add a criterion on, under `anchor`.
    fn field_menu(&self, anchor: &gtk::Widget, fields: &[usize]) {
        let entries = fields
            .iter()
            .filter_map(|&f| {
                let spec = self.specs.get(f)?;
                let c = self.clone();
                Some(MenuEntry::new(spec.label.clone(), move || {
                    c.insert(f, FilterMatch::Contains, "");
                    c.focus_last();
                }))
            })
            .collect();
        let y = f64::from(anchor.height());
        let x = f64::from(anchor.width()).min(400.0) / 2.0;
        show_context_menu(anchor, x, y, vec![entries]);
    }

    /// Put the cursor in the last criterion's text, if it takes one.
    fn focus_last(&self) {
        if let Some(entry) = self.items.borrow().last().and_then(|i| i.entry.clone()) {
            entry.grab_focus();
        }
    }

    /// Add a criterion on field `field`, before "Add criterion".
    pub(super) fn insert(&self, field: usize, matcher: FilterMatch, value: &str) {
        let Some(spec) = self.specs.get(field) else {
            return;
        };
        let matcher = Rc::new(Cell::new(if spec.contains_only {
            FilterMatch::Contains
        } else {
            matcher
        }));
        let more = gtk::Button::from_icon_name("view-more-symbolic");
        more.add_css_class("flat");
        more.set_valign(gtk::Align::Center);
        more.set_tooltip_text(Some(&i18n("Criterion options")));
        more.update_property(&[gtk::accessible::Property::Label(&i18n("Criterion options"))]);
        let (row, entry): (gtk::Widget, Option<adw::EntryRow>) = if spec.text {
            let entry = adw::EntryRow::new();
            entry.set_title(&title_of(spec, matcher.get()));
            entry.set_text(value);
            entry.add_suffix(&more);
            {
                let changed = self.changed.clone();
                entry.connect_changed(move |_| changed());
            }
            {
                let activate = self.activate.clone();
                entry.connect_entry_activated(move |_| activate());
            }
            (entry.clone().upcast(), Some(entry))
        } else {
            let row = adw::ActionRow::new();
            row.set_title(&spec.label);
            row.add_suffix(&more);
            (row.upcast(), None)
        };
        {
            let c = self.clone();
            let row = row.clone();
            let matcher = matcher.clone();
            more.connect_clicked(move |b| c.item_menu(b, &row, field, &matcher));
        }
        let at = 1 + self.fixed.get() + self.items.borrow().len();
        self.list.insert(&row, at as i32);
        self.items.borrow_mut().push(Item {
            field,
            matcher,
            row,
            entry,
        });
        self.refresh();
        (self.changed)();
    }

    /// A criterion's ⋯ menu: how its text matches, then Remove.
    fn item_menu(
        &self,
        anchor: &gtk::Button,
        row: &gtk::Widget,
        field: usize,
        matcher: &Rc<Cell<FilterMatch>>,
    ) {
        let Some(spec) = self.specs.get(field) else {
            return;
        };
        let mut sections = Vec::new();
        if spec.text && !spec.contains_only {
            let entries = FilterMatch::ALL
                .iter()
                .map(|&m| {
                    let current = matcher.get() == m;
                    let (c, row, matcher) = (self.clone(), row.clone(), matcher.clone());
                    MenuEntry::new(AccountsWindow::match_label(m), move || {
                        matcher.set(m);
                        if let (Some(entry), Some(spec)) =
                            (row.downcast_ref::<adw::EntryRow>(), c.specs.get(field))
                        {
                            entry.set_title(&title_of(spec, m));
                        }
                        (c.changed)();
                    })
                    .selected(current)
                })
                .collect();
            sections.push(entries);
        }
        let (c, row) = (self.clone(), row.clone());
        sections.push(vec![MenuEntry::new(i18n("Remove criterion"), move || {
            c.list.remove(&row);
            c.items.borrow_mut().retain(|i| i.row != row);
            c.refresh();
            (c.changed)();
        })
        .destructive()]);
        let y = f64::from(anchor.height());
        show_context_menu(anchor, 0.0, y, sections);
    }

    /// The all-or-any row shows once there are two criteria.
    pub(super) fn refresh(&self) {
        let fixed = self.fixed_set.borrow().as_ref().map_or(0, |count| count());
        self.combine
            .set_visible(fixed + self.items.borrow().len() > 1);
    }

    /// Whether any one criterion may match.
    pub(super) fn any(&self) -> bool {
        self.combine.selected() == 1
    }

    pub(super) fn set_any(&self, any: bool) {
        self.combine.set_selected(u32::from(any));
    }

    /// Each criterion as its field, how it matches, and its text (trimmed).
    pub(super) fn read(&self) -> Vec<(usize, FilterMatch, String)> {
        self.items
            .borrow()
            .iter()
            .map(|i| {
                let text = i
                    .entry
                    .as_ref()
                    .map(|e| e.text().trim().to_string())
                    .unwrap_or_default();
                (i.field, i.matcher.get(), text)
            })
            .collect()
    }
}

/// "Subject contains", or the field alone for one that always contains.
fn title_of(spec: &FieldSpec, matcher: FilterMatch) -> String {
    if spec.contains_only {
        i18n_f("{field} contains", &[("field", &spec.label)])
    } else {
        i18n_f(
            "{field} {match}",
            &[
                ("field", &spec.label),
                ("match", &AccountsWindow::match_label(matcher)),
            ],
        )
    }
}
