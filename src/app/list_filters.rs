//! The list's filters (docs/DESIGN.md, "Sorting, display and filters";
//! mockups 19 to 21): the Filters menu beside the search field, the chips
//! that name the filters in force under it, the All / Unread / Starred
//! switch, the match count, and the custom-period dialog.

use adw::prelude::*;
use relm4::prelude::*;
use std::cell::Cell;
use std::rc::Rc;

use crate::i18n::{i18n, i18n_f, ni18n_f};
use crate::ui::message_list::{Period, QuickFilter, QuickFilters};

use super::list_header::popover_heading;
use super::model::AppModel;
use super::msg::AppMsg;

/// The Filters menu and the area under the search field, kept to be set
/// as the filters change.
pub(super) struct FiltersUi {
    /// The Filters button, beside the search field.
    pub(super) button: gtk::MenuButton,
    /// Chips, Clear all, the switch and the count, under the search field.
    pub(super) area: gtk::Box,
    badge: gtk::Label,
    checks: Vec<(QuickFilter, gtk::CheckButton)>,
    periods: Vec<(Period, gtk::CheckButton)>,
    clear: gtk::Button,
    chips: gtk::FlowBox,
    clear_all: gtk::Button,
    show: adw::ToggleGroup,
    status: gtk::Label,
    /// What the widgets show, so a toggle that only restates it sends
    /// nothing back.
    shown: Rc<Cell<QuickFilters>>,
    sender: relm4::Sender<AppMsg>,
}

impl FiltersUi {
    pub(super) fn new(sender: &relm4::Sender<AppMsg>) -> Self {
        let shown: Rc<Cell<QuickFilters>> = Rc::default();

        // The menu: Messages (checks), Period (radios), Custom…, then
        // Clear filters, disabled while nothing is set (mockup 19). A
        // popover rather than a GMenu, so the checks and the radios keep
        // the popover open while several are set.
        let column = gtk::Box::new(gtk::Orientation::Vertical, 0);
        column.add_css_class("header-popover");
        column.append(&popover_heading(&i18n("Messages")));
        let mut checks = Vec::new();
        for f in QuickFilter::IN_MENU {
            let check = gtk::CheckButton::with_label(&f.label());
            let (s, shown) = (sender.clone(), shown.clone());
            check.connect_toggled(move |c| {
                if shown.get().has(f) != c.is_active() {
                    s.emit(AppMsg::SetQuickFilter(f, c.is_active()));
                }
            });
            column.append(&check);
            checks.push((f, check));
        }
        column.append(&gtk::Separator::new(gtk::Orientation::Horizontal));
        column.append(&popover_heading(&i18n("Period")));
        let mut periods: Vec<(Period, gtk::CheckButton)> = Vec::new();
        for p in Period::PRESETS {
            let radio = gtk::CheckButton::with_label(&p.label());
            if let Some((_, first)) = periods.first() {
                radio.set_group(Some(first));
            }
            let (s, shown) = (sender.clone(), shown.clone());
            radio.connect_toggled(move |r| {
                if r.is_active() && shown.get().period() != p {
                    s.emit(AppMsg::SetPeriod(p));
                }
            });
            column.append(&radio);
            periods.push((p, radio));
        }
        let popover = gtk::Popover::new();
        let custom = gtk::Button::with_label(&i18n("Custom…"));
        custom.add_css_class("flat");
        custom.add_css_class("header-popover-item");
        if let Some(label) = custom.child().and_downcast::<gtk::Label>() {
            label.set_xalign(0.0);
        }
        {
            let (s, popover) = (sender.clone(), popover.clone());
            custom.connect_clicked(move |_| {
                popover.popdown();
                s.emit(AppMsg::OpenCustomPeriod);
            });
        }
        column.append(&custom);
        column.append(&gtk::Separator::new(gtk::Orientation::Horizontal));
        let clear = gtk::Button::with_label(&i18n("Clear filters"));
        clear.add_css_class("flat");
        clear.add_css_class("header-popover-item");
        if let Some(label) = clear.child().and_downcast::<gtk::Label>() {
            label.set_xalign(0.0);
        }
        clear.set_sensitive(false);
        let s = sender.clone();
        clear.connect_clicked(move |_| s.emit(AppMsg::ClearQuickFilters));
        column.append(&clear);
        popover.set_child(Some(&column));

        // The button: the filter icon, "Filters", and how many are set in a
        // badge (mockup 20), the number in its accessible name too.
        let face = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        face.append(&gtk::Image::from_icon_name("filter-symbolic"));
        face.append(&gtk::Label::new(Some(&i18n("Filters"))));
        let badge = gtk::Label::new(None);
        badge.add_css_class("filters-count");
        badge.set_visible(false);
        face.append(&badge);
        let button = gtk::MenuButton::new();
        button.set_child(Some(&face));
        button.set_popover(Some(&popover));
        button.set_valign(gtk::Align::Center);
        button.add_css_class("flat");

        // The area under the search field (mockup 20): the chips, Clear
        // all, the switch, the count.
        let area = gtk::Box::new(gtk::Orientation::Vertical, 0);
        area.add_css_class("list-filter-area");
        let chips = gtk::FlowBox::new();
        chips.set_selection_mode(gtk::SelectionMode::None);
        chips.set_max_children_per_line(8);
        chips.set_column_spacing(6);
        chips.set_row_spacing(6);
        chips.add_css_class("filter-chips");
        chips.set_visible(false);
        area.append(&chips);
        let clear_all = gtk::Button::with_label(&i18n("Clear all"));
        clear_all.add_css_class("flat");
        clear_all.add_css_class("filter-clear-all");
        clear_all.set_halign(gtk::Align::Start);
        clear_all.set_visible(false);
        let s = sender.clone();
        clear_all.connect_clicked(move |_| s.emit(AppMsg::ClearQuickFilters));
        area.append(&clear_all);
        let bar = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        bar.add_css_class("list-show-bar");
        let show = build_show_switch(&bar, sender);
        area.append(&bar);
        let status = gtk::Label::new(None);
        status.set_xalign(0.0);
        status.add_css_class("caption");
        status.add_css_class("secondary-text");
        status.add_css_class("filter-status");
        status.set_visible(false);
        area.append(&status);

        Self {
            button,
            area,
            badge,
            checks,
            periods,
            clear,
            chips,
            clear_all,
            show,
            status,
            shown,
            sender: sender.clone(),
        }
    }

    /// Show `filters` everywhere they appear, and `count` (the header's
    /// match count) under the chips while any filter is on.
    pub(super) fn sync(&self, filters: QuickFilters, count: &str) {
        let changed = self.shown.get() != filters;
        self.shown.set(filters);
        for (f, check) in &self.checks {
            if check.is_active() != filters.has(*f) {
                check.set_active(filters.has(*f));
            }
        }
        // A custom period ticks no preset.
        for (p, radio) in &self.periods {
            let on = filters.period() == *p;
            if radio.is_active() != on {
                radio.set_active(on);
            }
        }
        let n = filters.menu_count();
        self.clear.set_sensitive(n > 0);
        self.badge.set_label(&n.to_string());
        self.badge.set_visible(n > 0);
        let name = if n == 0 {
            i18n("Filters")
        } else {
            ni18n_f(
                "Filters, {n} active",
                "Filters, {n} active",
                n as u32,
                &[("n", &n.to_string())],
            )
        };
        self.button.set_tooltip_text(Some(&name));
        self.button
            .update_property(&[gtk::accessible::Property::Label(&name)]);

        let toggle = show_toggle(
            filters.has(QuickFilter::Unread),
            filters.has(QuickFilter::Starred),
        );
        if self.show.active_name().as_deref() != toggle {
            self.show.set_active_name(toggle);
        }

        if changed {
            self.fill_chips(filters);
        }
        self.chips.set_visible(n > 0);
        self.clear_all.set_visible(n > 0);
        let matches = count.trim().parse::<u32>().ok();
        match matches.filter(|_| !filters.is_empty()) {
            Some(m) => {
                self.status.set_label(&ni18n_f(
                    "{count} message",
                    "{count} messages",
                    m,
                    &[("count", &m.to_string())],
                ));
                self.status.set_visible(true);
            }
            None => self.status.set_visible(false),
        }
    }

    /// A chip per filter the menu has on, each with a button that takes
    /// it off, named for screen readers ("Remove filter: …").
    fn fill_chips(&self, filters: QuickFilters) {
        self.chips.remove_all();
        let add = |name: String, off: AppMsg| {
            let chip = gtk::Box::new(gtk::Orientation::Horizontal, 2);
            chip.add_css_class("filter-chip");
            chip.set_halign(gtk::Align::Start);
            chip.append(&gtk::Label::new(Some(&name)));
            let remove = gtk::Button::from_icon_name("window-close-symbolic");
            remove.add_css_class("flat");
            remove.add_css_class("circular");
            let label = i18n_f("Remove filter: {name}", &[("name", &name)]);
            remove.set_tooltip_text(Some(&label));
            remove.update_property(&[gtk::accessible::Property::Label(&label)]);
            let s = self.sender.clone();
            let off = std::cell::RefCell::new(Some(off));
            remove.connect_clicked(move |_| {
                if let Some(msg) = off.borrow_mut().take() {
                    s.emit(msg);
                }
            });
            chip.append(&remove);
            self.chips.append(&chip);
        };
        for f in QuickFilter::IN_MENU {
            if filters.has(f) {
                add(f.label(), AppMsg::SetQuickFilter(f, false));
            }
        }
        if filters.period() != Period::Any {
            add(filters.period().label(), AppMsg::SetPeriod(Period::Any));
        }
    }
}

/// Build the All / Unread / Starred switch into `bar` (docs/DESIGN.md,
/// "Sorting, display and filters"). It turns the Unread and Starred quick
/// filters on and off; the others are in the Filters menu.
fn build_show_switch(bar: &gtk::Box, sender: &relm4::Sender<AppMsg>) -> adw::ToggleGroup {
    let group = adw::ToggleGroup::new();
    group.set_hexpand(true);
    for (name, label) in [
        ("all", i18n("All")),
        ("unread", i18n("Unread")),
        ("starred", i18n("Starred")),
    ] {
        group.add(adw::Toggle::builder().name(name).label(label).build());
    }
    group.set_active_name(Some("all"));
    let s = sender.clone();
    group.connect_active_name_notify(move |g| {
        // No toggle is active while both filters are on (set from the
        // keyboard or the overflow menu): that is not a choice to act on.
        let only = match g.active_name().as_deref() {
            Some("all") => None,
            Some("unread") => Some(QuickFilter::Unread),
            Some("starred") => Some(QuickFilter::Starred),
            _ => return,
        };
        s.emit(AppMsg::SetListShow(only));
    });
    bar.append(&group);
    group
}

/// The switch's toggle for the Unread and Starred filters as they are: one
/// of them alone, or neither. With both on, no single toggle tells the
/// truth, so none is active.
fn show_toggle(unread: bool, starred: bool) -> Option<&'static str> {
    match (unread, starred) {
        (false, false) => Some("all"),
        (true, false) => Some("unread"),
        (false, true) => Some("starred"),
        (true, true) => None,
    }
}

/// A day as the custom-period fields write it: the locale's short date
/// (02/11/2026 in French), which [`parse_day`] reads back.
fn field_text(day: chrono::NaiveDate) -> String {
    use chrono::Datelike;
    gtk::glib::DateTime::from_local(day.year(), day.month() as i32, day.day() as i32, 12, 0, 0.0)
        .ok()
        .and_then(|t| t.format("%x").ok())
        .map(|s| s.to_string())
        .unwrap_or_default()
}

/// The locale's short date as a chrono pattern: the order of day, month
/// and year [`field_text`] writes, worked out from a day whose parts
/// cannot be mistaken for one another (22 November 2000).
fn field_pattern() -> Option<String> {
    let probe = field_text(chrono::NaiveDate::from_ymd_opt(2000, 11, 22)?);
    let pattern = probe
        .replace("2000", "%Y")
        .replace("22", "%d")
        .replace("11", "%m")
        .replace("00", "%y");
    pattern.contains("%d").then_some(pattern)
}

/// A day typed into a custom-period field, read in the order the locale
/// writes dates, then as GLib's `g_date_set_parse` reads them; ISO dates
/// are read too. `None` for text that names no day.
fn parse_day(text: &str) -> Option<chrono::NaiveDate> {
    let text = text.trim();
    if text.is_empty() {
        return None;
    }
    if let Ok(day) = chrono::NaiveDate::parse_from_str(text, "%Y-%m-%d") {
        return Some(day);
    }
    if let Some(day) =
        field_pattern().and_then(|p| chrono::NaiveDate::parse_from_str(text, &p).ok())
    {
        return Some(day);
    }
    let mut date = gtk::glib::Date::from_dmy(1, gtk::glib::DateMonth::January, 2000).ok()?;
    date.set_parse(text).ok()?;
    if !date.valid() {
        return None;
    }
    chrono::NaiveDate::from_ymd_opt(
        i32::from(date.year()),
        date.month().into_glib() as u32,
        u32::from(date.day()),
    )
}

use gtk::glib::translate::IntoGlib;

impl AppModel {
    /// The custom-period dialog (mockup 21): From and To, each a field
    /// with a calendar behind a button, Cancel and Apply. A field left
    /// empty leaves that side open; Apply waits until each filled field
    /// names a day.
    pub(super) fn open_custom_period(&self, sender: &ComponentSender<Self>) {
        let (from0, to0) = match self.quick_filters.period() {
            Period::Custom { from, to } => (from, to),
            _ => (None, None),
        };
        let dialog = adw::Dialog::new();
        dialog.set_title(&i18n("Custom period"));
        dialog.set_content_width(520);
        let header = adw::HeaderBar::new();
        header.set_show_start_title_buttons(false);
        header.set_show_end_title_buttons(false);
        let cancel = gtk::Button::with_label(&i18n("Cancel"));
        let apply = gtk::Button::with_label(&i18n("Apply"));
        apply.add_css_class("suggested-action");
        header.pack_start(&cancel);
        header.pack_end(&apply);

        let grid = gtk::Grid::new();
        grid.set_row_spacing(10);
        grid.set_column_spacing(12);
        grid.set_margin_top(18);
        grid.set_margin_bottom(18);
        grid.set_margin_start(18);
        grid.set_margin_end(18);
        let field = |row: i32, title: String, day: Option<chrono::NaiveDate>| {
            let label = gtk::Label::new(Some(&title));
            label.set_xalign(0.0);
            label.add_css_class("secondary-text");
            grid.attach(&label, 0, row, 1, 1);
            let entry = gtk::Entry::new();
            entry.set_hexpand(true);
            entry.set_text(&day.map(field_text).unwrap_or_default());
            entry.update_relation(&[gtk::accessible::Relation::LabelledBy(&[label.upcast_ref()])]);
            grid.attach(&entry, 1, row, 1, 1);
            let calendar = gtk::Calendar::new();
            let pick = gtk::MenuButton::new();
            pick.set_icon_name("x-office-calendar-symbolic");
            let choose = i18n("Choose a date");
            pick.set_tooltip_text(Some(&choose));
            pick.update_property(&[gtk::accessible::Property::Label(&choose)]);
            let popover = gtk::Popover::new();
            popover.set_child(Some(&calendar));
            pick.set_popover(Some(&popover));
            // The calendar opens on the field's day, and a day picked on it
            // is written into the field.
            {
                let (entry, calendar) = (entry.clone(), calendar.clone());
                popover.connect_show(move |_| {
                    use chrono::Datelike;
                    if let Some(d) = parse_day(&entry.text()) {
                        if let Ok(at) = gtk::glib::DateTime::from_local(
                            d.year(),
                            d.month() as i32,
                            d.day() as i32,
                            12,
                            0,
                            0.0,
                        ) {
                            calendar.select_day(&at);
                        }
                    }
                });
            }
            {
                let (entry, popover) = (entry.clone(), popover.clone());
                calendar.connect_day_selected(move |cal| {
                    let d = cal.date();
                    if let Some(day) = chrono::NaiveDate::from_ymd_opt(
                        d.year(),
                        d.month() as u32,
                        d.day_of_month() as u32,
                    ) {
                        entry.set_text(&field_text(day));
                    }
                    popover.popdown();
                });
            }
            grid.attach(&pick, 2, row, 1, 1);
            entry
        };
        let from = field(0, i18n("From"), from0);
        let to = field(1, i18n("To"), to0);
        let help = gtk::Label::new(Some(&i18n(
            "Both dates are included. Leave a field empty to leave that side open.",
        )));
        help.set_wrap(true);
        help.set_xalign(0.0);
        help.add_css_class("caption");
        help.add_css_class("secondary-text");
        grid.attach(&help, 0, 2, 3, 1);

        // A filled field that names no day holds Apply back.
        let read = |e: &gtk::Entry| -> Result<Option<chrono::NaiveDate>, ()> {
            let text = e.text();
            if text.trim().is_empty() {
                Ok(None)
            } else {
                parse_day(&text).map(Some).ok_or(())
            }
        };
        let check = {
            let (from, to, apply) = (from.clone(), to.clone(), apply.clone());
            move || {
                let ok = read(&from).is_ok() && read(&to).is_ok();
                apply.set_sensitive(ok);
                for e in [&from, &to] {
                    if read(e).is_ok() {
                        e.remove_css_class("error");
                    } else {
                        e.add_css_class("error");
                    }
                }
            }
        };
        for e in [&from, &to] {
            let check = check.clone();
            e.connect_changed(move |_| check());
            e.set_activates_default(true);
        }
        check();

        let view = adw::ToolbarView::new();
        view.add_top_bar(&header);
        view.set_content(Some(&grid));
        dialog.set_child(Some(&view));
        dialog.set_default_widget(Some(&apply));
        {
            let dialog = dialog.clone();
            cancel.connect_clicked(move |_| {
                dialog.close();
            });
        }
        {
            let (dialog, s) = (dialog.clone(), sender.input_sender().clone());
            apply.connect_clicked(move |_| {
                let (Ok(from), Ok(to)) = (read(&from), read(&to)) else {
                    return;
                };
                s.emit(AppMsg::SetPeriod(Period::Custom { from, to }));
                dialog.close();
            });
        }
        dialog.present(Some(&self.window));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_filter_alone_has_its_toggle() {
        assert_eq!(show_toggle(false, false), Some("all"));
        assert_eq!(show_toggle(true, false), Some("unread"));
        assert_eq!(show_toggle(false, true), Some("starred"));
    }

    #[test]
    fn both_filters_on_light_no_toggle() {
        assert_eq!(show_toggle(true, true), None);
    }

    /// ISO dates are read whatever the locale, and a day written as the
    /// fields write it reads back; empty text names no day.
    #[test]
    fn custom_period_fields_read_iso_days() {
        let day = chrono::NaiveDate::from_ymd_opt(2026, 11, 2);
        assert_eq!(parse_day(" 2026-11-02 "), day);
        assert_eq!(parse_day(&field_text(day.unwrap())), day);
        assert_eq!(parse_day("not a day"), None);
        assert_eq!(parse_day(""), None);
    }
}
