//! The Tags panel (docs/DESIGN.md, "Tags and favourite"; mockup 07): the
//! popover the reading pane's Tags button opens. A search field that also
//! creates, Favorite (the `\Flagged` flag) as the first row, one row per tag
//! with a tick and its colour, then New Tag… and Manage Tags….
//!
//! It owns the popover and what it shows while open: the filter, the ticks
//! flipped by a click. The app owns the tags and the messages: every choice
//! goes back through the callbacks, and the app's own state is what the
//! panel shows the next time it opens.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use gtk::prelude::*;

use crate::config::Tag;
use crate::i18n::{i18n, i18n_f};

/// What the panel shows when it opens.
pub(crate) struct TagsPanelState {
    pub(crate) tags: Vec<Tag>,
    /// The keywords the message carries (every selected message, when
    /// several are: only then does choosing a tag take it off).
    pub(crate) on: Vec<String>,
    pub(crate) starred: bool,
    /// The key that stars, as the shortcuts list writes it ("s"), if any.
    pub(crate) star_key: Option<String>,
}

/// What the panel's choices do. Each is called on the main thread.
pub(crate) struct TagsPanelActions {
    /// Put the tag with this keyword on, or take it off.
    pub(crate) toggle: Box<dyn Fn(String)>,
    pub(crate) toggle_star: Box<dyn Fn()>,
    /// Make a tag of this name and put it on (from the search field).
    pub(crate) create: Box<dyn Fn(String)>,
    pub(crate) new_tag: Box<dyn Fn()>,
    pub(crate) manage: Box<dyn Fn()>,
}

/// One tag row, kept to filter it and to flip its tick.
struct TagRow {
    row: gtk::ListBoxRow,
    name: String,
    keyword: String,
    tick: gtk::Image,
    on: Cell<bool>,
}

/// Open the panel under `anchor`.
pub(crate) fn show(
    anchor: &impl IsA<gtk::Widget>,
    state: TagsPanelState,
    actions: TagsPanelActions,
) {
    let actions = Rc::new(actions);
    let popover = gtk::Popover::new();
    popover.add_css_class("tags-panel");
    popover.set_has_arrow(false);
    popover.set_position(gtk::PositionType::Bottom);

    let content = gtk::Box::new(gtk::Orientation::Vertical, 6);
    content.set_size_request(280, -1);

    let search = gtk::SearchEntry::new();
    search.set_placeholder_text(Some(&i18n("Search or create…")));
    search.update_property(&[gtk::accessible::Property::Label(&i18n(
        "Search or create a tag",
    ))]);
    content.append(&search);

    let list = gtk::ListBox::new();
    list.set_selection_mode(gtk::SelectionMode::None);
    list.add_css_class("tags-panel-list");

    // Favorite: the star, on every message whatever tags exist.
    let star_on = Rc::new(Cell::new(state.starred));
    let star_parts: Rc<RefCell<Option<(gtk::Image, gtk::Image)>>> = Rc::new(RefCell::new(None));
    let star_row = gtk::ListBoxRow::new();
    {
        let line = row_box();
        let tick = tick_image(state.starred);
        line.append(&tick);
        let icon = gtk::Image::from_icon_name(if state.starred {
            "starred-symbolic"
        } else {
            "tegami-non-starred-symbolic"
        });
        icon.set_accessible_role(gtk::AccessibleRole::Presentation);
        line.append(&icon);
        let label = gtk::Label::new(Some(&i18n("Favorite")));
        label.set_hexpand(true);
        label.set_xalign(0.0);
        line.append(&label);
        if let Some(key) = &state.star_key {
            let keys = gtk::Label::new(Some(&key.to_uppercase()));
            keys.add_css_class("dim-label");
            keys.add_css_class("caption");
            line.append(&keys);
        }
        star_row.set_child(Some(&line));
        set_checked(&star_row, state.starred);
        star_parts.replace(Some((tick, icon)));
    }
    list.append(&star_row);

    // The tags, each with its tick and its colour.
    let rows: Rc<RefCell<Vec<TagRow>>> = Rc::new(RefCell::new(Vec::new()));
    for (i, tag) in state.tags.iter().enumerate() {
        let on = state
            .on
            .iter()
            .any(|k| k.eq_ignore_ascii_case(&tag.keyword));
        let row = gtk::ListBoxRow::new();
        let line = row_box();
        let tick = tick_image(on);
        line.append(&tick);
        line.append(&crate::ui::context_menu::swatch_widget(&tag.color, true));
        let label = gtk::Label::new(Some(&tag.name));
        label.set_hexpand(true);
        label.set_xalign(0.0);
        label.set_ellipsize(gtk::pango::EllipsizeMode::End);
        line.append(&label);
        // The first nine tags have a number key (KEYBOARD_SHORTCUTS.md).
        if let Some(key) = crate::ui::keymap::tag_key(i) {
            let keys = gtk::Label::new(Some(key));
            keys.add_css_class("dim-label");
            keys.add_css_class("caption");
            line.append(&keys);
        }
        row.set_child(Some(&line));
        if i == 0 {
            row.add_css_class("tags-panel-first");
        }
        set_checked(&row, on);
        list.append(&row);
        rows.borrow_mut().push(TagRow {
            row,
            name: tag.name.clone(),
            keyword: tag.keyword.clone(),
            tick,
            on: Cell::new(on),
        });
    }

    // Create "…": shown while the search names no tag.
    let create_row = gtk::ListBoxRow::new();
    let create_label = gtk::Label::new(None);
    {
        let line = row_box();
        let icon = gtk::Image::from_icon_name("list-add-symbolic");
        icon.set_accessible_role(gtk::AccessibleRole::Presentation);
        line.append(&icon);
        create_label.set_xalign(0.0);
        create_label.set_hexpand(true);
        create_label.set_ellipsize(gtk::pango::EllipsizeMode::End);
        line.append(&create_label);
        create_row.set_child(Some(&line));
        create_row.set_visible(false);
    }
    list.append(&create_row);
    content.append(&list);

    content.append(&gtk::Separator::new(gtk::Orientation::Horizontal));
    let new_tag = menu_button(&i18n("New tag…"));
    let manage = menu_button(&i18n("Manage tags…"));
    content.append(&new_tag);
    content.append(&manage);
    popover.set_child(Some(&content));

    // Choosing a tag flips its tick in place: the panel stays open, so
    // several tags can be set in one visit, as the mockup shows.
    {
        let (rows, actions) = (rows.clone(), actions.clone());
        let (star_row, create_row, search, popover) = (
            star_row.clone(),
            create_row.clone(),
            search.clone(),
            popover.clone(),
        );
        let (star_on, star_parts) = (star_on.clone(), star_parts.clone());
        list.connect_row_activated(move |_, activated| {
            if activated == &star_row {
                let on = !star_on.get();
                star_on.set(on);
                if let Some((tick, icon)) = star_parts.borrow().as_ref() {
                    tick.set_opacity(if on { 1.0 } else { 0.0 });
                    icon.set_icon_name(Some(if on {
                        "starred-symbolic"
                    } else {
                        "tegami-non-starred-symbolic"
                    }));
                }
                set_checked(&star_row, on);
                (actions.toggle_star)();
                return;
            }
            if activated == &create_row {
                let name = search.text().trim().to_string();
                if !name.is_empty() {
                    (actions.create)(name);
                }
                popover.popdown();
                return;
            }
            if let Some(r) = rows.borrow().iter().find(|r| &r.row == activated) {
                let on = !r.on.get();
                r.on.set(on);
                r.tick.set_opacity(if on { 1.0 } else { 0.0 });
                set_checked(&r.row, on);
                (actions.toggle)(r.keyword.clone());
            }
        });
    }
    // Typing filters the tags; a name no tag has offers to create it.
    {
        let (rows, create_row, create_label, star_row) = (
            rows.clone(),
            create_row.clone(),
            create_label.clone(),
            star_row.clone(),
        );
        search.connect_search_changed(move |entry| {
            let typed = entry.text().trim().to_string();
            let needle = typed.to_lowercase();
            let mut exact = false;
            for r in rows.borrow().iter() {
                let name = r.name.to_lowercase();
                r.row
                    .set_visible(needle.is_empty() || name.contains(&needle));
                exact |= name == needle;
            }
            star_row.set_visible(needle.is_empty());
            create_row.set_visible(!typed.is_empty() && !exact);
            create_label.set_label(&i18n_f("Create “{name}”", &[("name", &typed)]));
        });
    }
    // Return: the one tag left, else create what was typed.
    {
        let (rows, list, create_row) = (rows.clone(), list.clone(), create_row.clone());
        search.connect_activate(move |_| {
            let shown: Vec<gtk::ListBoxRow> = rows
                .borrow()
                .iter()
                .filter(|r| r.row.is_visible())
                .map(|r| r.row.clone())
                .collect();
            match shown.as_slice() {
                [one] => list.emit_by_name::<()>("row-activated", &[one]),
                _ if create_row.is_visible() => {
                    list.emit_by_name::<()>("row-activated", &[&create_row])
                }
                _ => {}
            }
        });
    }
    {
        let (actions, popover) = (actions.clone(), popover.clone());
        new_tag.connect_clicked(move |_| {
            popover.popdown();
            (actions.new_tag)();
        });
    }
    {
        let (actions, popover) = (actions.clone(), popover.clone());
        manage.connect_clicked(move |_| {
            popover.popdown();
            (actions.manage)();
        });
    }

    popover.set_parent(anchor.as_ref());
    popover.connect_closed(|p| {
        // Unparented once closed, so the next opening builds afresh from
        // the app's state rather than keeping this one's.
        let p = p.clone();
        gtk::glib::idle_add_local_once(move || p.unparent());
    });
    popover.popup();
    search.grab_focus();
    showcase_capture(content.upcast_ref());
}

/// Demo only: with TEGAMI_SHOWCASE_TAGS and TEGAMI_SHOWCASE set, capture
/// `widget` a second after it shows (see `app::showcase::list::stage_tags`).
pub(crate) fn showcase_capture(widget: &gtk::Widget) {
    if std::env::var_os("TEGAMI_SHOWCASE_TAGS").is_none() {
        return;
    }
    if let Ok(path) = std::env::var("TEGAMI_SHOWCASE") {
        let widget = widget.clone();
        gtk::glib::timeout_add_local_once(std::time::Duration::from_millis(1000), move || {
            crate::ui::showcase::showcase_capture(&widget, &format!("{path}.tags.png"));
        });
    }
}

/// A row's line: tick, mark, name, key, spaced as a menu's.
fn row_box() -> gtk::Box {
    let line = gtk::Box::new(gtk::Orientation::Horizontal, 10);
    line.set_margin_top(6);
    line.set_margin_bottom(6);
    line.set_margin_start(6);
    line.set_margin_end(6);
    line
}

/// The tick before a row that is on. Kept in place when off (transparent)
/// so the names line up whatever is ticked.
fn tick_image(on: bool) -> gtk::Image {
    let tick = gtk::Image::from_icon_name("object-select-symbolic");
    tick.set_opacity(if on { 1.0 } else { 0.0 });
    tick.set_accessible_role(gtk::AccessibleRole::Presentation);
    tick
}

/// Tell assistive technology whether a row is on: the tick is drawn, not
/// read out.
fn set_checked(row: &gtk::ListBoxRow, on: bool) {
    row.set_accessible_role(gtk::AccessibleRole::Checkbox);
    row.update_state(&[gtk::accessible::State::Checked(if on {
        gtk::AccessibleTristate::True
    } else {
        gtk::AccessibleTristate::False
    })]);
}

/// A full-width flat button with its label at the start, as a menu entry.
fn menu_button(label: &str) -> gtk::Button {
    let text = gtk::Label::new(Some(label));
    text.set_xalign(0.0);
    let button = gtk::Button::new();
    button.set_child(Some(&text));
    button.add_css_class("flat");
    button
}
