//! The message list's header: the sort and filter popover, and the wide/narrow switch.

use adw::prelude::*;
use relm4::prelude::*;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

use crate::config;
use crate::i18n::{i18n, i18n_f, i18n_noop};
use crate::ui::message_list::{DateBound, MessageListInput, QuickFilter, QuickFilters};
use crate::ui::message_view::MessageViewInput;

use super::model::AppModel;
use super::msg::AppMsg;
use super::AppModelWidgets;

/// How wide the list pane has to be dragged before Automatic lays the list
/// out on one line, and how far back before it returns to cards (#334).
pub(super) const LIST_WIDE_ON: i32 = 600;

const LIST_WIDE_OFF: i32 = 560;

/// The message list header's sort and filter popover, its sort action and
/// the All / Unread / Starred switch, kept to be set from elsewhere.
pub(super) struct ListHeaderWidgets {
    filter: SortFilterPopover,
    sort: gtk::gio::SimpleAction,
    /// The All / Unread / Starred switch above the list.
    show: adw::ToggleGroup,
}

/// The orders the list header offers, by their keys in `SortOrder`. The
/// column headings can set more (#443); none of these is ticked then.
const SORT_ORDERS: [(&str, &str); 6] = [
    (i18n_noop("Date (Newest first)"), "date_newest"),
    (i18n_noop("Date (Oldest first)"), "date_oldest"),
    (i18n_noop("Sender (A–Z)"), "sender"),
    (i18n_noop("Subject (A–Z)"), "subject"),
    (i18n_noop("Unread first"), "unread"),
    (i18n_noop("Starred first"), "flagged"),
];

/// The list header's sort and filter button (#441): the sort orders in one
/// column of its popover, the quick filters in the next, the date range in
/// the last.
struct SortFilterPopover {
    button: gtk::MenuButton,
    /// Show: Group into Conversations, Message Preview, Avatars (docs/
    /// DESIGN.md, "Menus"), under the sort orders. Each sets its setting;
    /// `sync_show` puts the setting back on it when it changes elsewhere.
    show: [gtk::CheckButton; 3],
    /// The preview lines to come back to when Message Preview is ticked
    /// again: the last count above none.
    preview_lines: Rc<Cell<u32>>,
    filters: Vec<(QuickFilter, gtk::CheckButton)>,
    dates: Rc<DateRange>,
    clear: gtk::Button,
}

/// The popover's date range, its last column: From and To, each a toggle
/// naming its day, a calendar that sets the one toggled, and Clear Dates.
/// The calendar sits in the popover itself, as a popover of its own inside
/// this one would leave this one unable to close on a click outside.
struct DateRange {
    ends: [(DateBound, gtk::ToggleButton, gtk::Label); 2],
    calendar: gtk::Calendar,
    clear: gtk::Button,
    days: Rc<Cell<[Option<chrono::NaiveDate>; 2]>>,
    /// Set while the toggles or the calendar are moved to show the range in
    /// force, so the move is not taken for a pick.
    quiet: Rc<Cell<bool>>,
}

fn end_index(end: DateBound) -> usize {
    match end {
        DateBound::From => 0,
        DateBound::To => 1,
    }
}

impl DateRange {
    fn new(sender: &relm4::Sender<AppMsg>) -> Rc<Self> {
        let width = widest_day_label();
        let toggle = |end: DateBound| {
            // As wide as the longest date it can name, so the popover
            // keeps its width whichever day is picked.
            let label = gtk::Label::new(None);
            label.set_width_chars(width);
            let button = gtk::ToggleButton::new();
            button.set_child(Some(&label));
            button.set_hexpand(true);
            (end, button, label)
        };
        let ends = [toggle(DateBound::From), toggle(DateBound::To)];
        ends[1].1.set_group(Some(&ends[0].1));
        let clear = gtk::Button::with_label(&i18n("Clear dates"));
        clear.add_css_class("flat");
        clear.set_halign(gtk::Align::Start);
        let this = Rc::new(Self {
            ends,
            calendar: gtk::Calendar::new(),
            clear,
            days: Rc::default(),
            quiet: Rc::default(),
        });

        // Toggling an end shows its day on the calendar; an end left open
        // takes the day in blue.
        for (end, button, _) in &this.ends {
            let (end, weak, s) = (*end, Rc::downgrade(&this), sender.clone());
            button.connect_toggled(move |b| {
                let Some(this) = weak.upgrade() else { return };
                if this.quiet.get() || !b.is_active() {
                    return;
                }
                match this.days.get()[end_index(end)] {
                    Some(day) => this.show_on_calendar(day),
                    None => s.emit(AppMsg::SetFilterDate(end, calendar_day(&this.calendar))),
                }
            });
        }
        // The arrows and the keyboard move the day in blue, and the end
        // being set follows once they rest, rather than on every step.
        let pending: Rc<RefCell<Option<gtk::glib::SourceId>>> = Rc::default();
        {
            let (weak, s, pending) = (Rc::downgrade(&this), sender.clone(), pending.clone());
            this.calendar.connect_day_selected(move |cal| {
                let Some(this) = weak.upgrade() else { return };
                let Some(end) = this.editing() else { return };
                if this.quiet.get() {
                    return;
                }
                if let Some(id) = pending.take() {
                    id.remove();
                }
                let (cal, s, slot) = (cal.clone(), s.clone(), pending.clone());
                let id = gtk::glib::timeout_add_local_once(
                    std::time::Duration::from_millis(250),
                    move || {
                        slot.take();
                        s.emit(AppMsg::SetFilterDate(end, calendar_day(&cal)));
                    },
                );
                pending.replace(Some(id));
            });
        }
        // A click on a day sets the end toggled, From if neither is, the
        // day already in blue too, which the calendar does not report as
        // selected again. Captured ahead of the calendar's own click, and
        // read once that has moved the selection.
        let click = gtk::GestureClick::new();
        click.set_propagation_phase(gtk::PropagationPhase::Capture);
        {
            let (weak, s) = (Rc::downgrade(&this), sender.clone());
            click.connect_pressed(move |_, presses, x, y| {
                let Some(this) = weak.upgrade() else { return };
                let on_day = this
                    .calendar
                    .pick(x, y, gtk::PickFlags::DEFAULT)
                    .is_some_and(|w| w.has_css_class("day-number"));
                if presses != 1 || !on_day {
                    return;
                }
                let (weak, s, pending) = (weak.clone(), s.clone(), pending.clone());
                gtk::glib::idle_add_local_once(move || {
                    let Some(this) = weak.upgrade() else { return };
                    if let Some(id) = pending.take() {
                        id.remove();
                    }
                    let end = this.editing().unwrap_or(DateBound::From);
                    this.quiet.set(true);
                    this.ends[end_index(end)].1.set_active(true);
                    this.quiet.set(false);
                    s.emit(AppMsg::SetFilterDate(end, calendar_day(&this.calendar)));
                });
            });
        }
        this.calendar.add_controller(click);
        let s = sender.clone();
        this.clear
            .connect_clicked(move |_| s.emit(AppMsg::ClearFilterDates));
        this.show(QuickFilters::default());
        this
    }

    /// The end the calendar sets: the one toggled, if either is.
    fn editing(&self) -> Option<DateBound> {
        self.ends
            .iter()
            .find(|(_, b, _)| b.is_active())
            .map(|(end, _, _)| *end)
    }

    fn show_on_calendar(&self, day: chrono::NaiveDate) {
        use chrono::Datelike;
        if calendar_day(&self.calendar) == Some(day) {
            return;
        }
        if let Ok(at) = gtk::glib::DateTime::from_local(
            day.year(),
            day.month() as i32,
            day.day() as i32,
            12,
            0,
            0.0,
        ) {
            self.quiet.set(true);
            self.calendar.select_day(&at);
            self.quiet.set(false);
        }
    }

    /// Show the range in force: each end's day on its toggle, the end being
    /// set on the calendar, and neither toggled once both are open.
    fn show(&self, filters: QuickFilters) {
        let days = [filters.date(DateBound::From), filters.date(DateBound::To)];
        self.days.set(days);
        for ((_, button, label), day) in self.ends.iter().zip(days) {
            label.set_label(&day.map_or_else(|| i18n("Any date"), day_label));
            if day.is_none() && days.iter().all(Option::is_none) && button.is_active() {
                self.quiet.set(true);
                button.set_active(false);
                self.quiet.set(false);
            }
        }
        if let Some(day) = self.editing().and_then(|end| days[end_index(end)]) {
            self.show_on_calendar(day);
        }
        self.clear.set_sensitive(filters.has_dates());
    }
}

/// The day selected on `cal`.
fn calendar_day(cal: &gtk::Calendar) -> Option<chrono::NaiveDate> {
    let d = cal.date();
    chrono::NaiveDate::from_ymd_opt(d.year(), d.month() as u32, d.day_of_month() as u32)
}

/// The most characters a date button names: Any Date, or a two-digit day
/// of the longest month written as the list writes dates.
fn widest_day_label() -> i32 {
    (1..=12)
        .filter_map(|m| chrono::NaiveDate::from_ymd_opt(2026, m, 28))
        .map(day_label)
        .chain([i18n("Any date")])
        .map(|t| t.chars().count())
        .max()
        .unwrap_or(12) as i32
        + 1
}

/// A day as the list writes dates, at its noon so no clock change moves it.
fn day_label(day: chrono::NaiveDate) -> String {
    use chrono::TimeZone;
    day.and_hms_opt(12, 0, 0)
        .and_then(|t| chrono::Local.from_local_datetime(&t).earliest())
        .map(|t| crate::datefmt::day_month_year(t.timestamp()))
        .unwrap_or_default()
}

/// The date range for the button's tooltip.
fn range_label(filters: QuickFilters) -> Option<String> {
    let from = filters.date(DateBound::From).map(day_label);
    let to = filters.date(DateBound::To).map(day_label);
    match (from, to) {
        (Some(from), Some(to)) => Some(i18n_f("{from} to {to}", &[("from", &from), ("to", &to)])),
        (Some(date), None) => Some(i18n_f("Since {date}", &[("date", &date)])),
        (None, Some(date)) => Some(i18n_f("Until {date}", &[("date", &date)])),
        (None, None) => None,
    }
}

impl SortFilterPopover {
    /// The popover stays open as rows are clicked, so an order and several
    /// filters can be set in one visit.
    pub(super) fn new(
        button: gtk::MenuButton,
        sort: &gtk::gio::SimpleAction,
        sender: &relm4::Sender<AppMsg>,
    ) -> Self {
        let column = |title: &str| {
            let col = gtk::Box::new(gtk::Orientation::Vertical, 0);
            col.add_css_class("sort-filter-column");
            let caption = gtk::Label::new(Some(title));
            caption.set_xalign(0.0);
            caption.add_css_class("dim-label");
            caption.add_css_class("caption-heading");
            col.append(&caption);
            col
        };

        let sort_col = column(&i18n("Sort by"));
        let mut radios: Vec<(&'static str, gtk::CheckButton)> = Vec::new();
        for (label, key) in SORT_ORDERS {
            let radio = gtk::CheckButton::with_label(&i18n(label));
            if let Some((_, first)) = radios.first() {
                radio.set_group(Some(first));
            }
            let action = sort.clone();
            radio.connect_toggled(move |r| {
                let current = action.state().and_then(|v| v.str().map(String::from));
                if r.is_active() && current.as_deref() != Some(key) {
                    action.activate(Some(&key.to_variant()));
                }
            });
            sort_col.append(&radio);
            radios.push((key, radio));
        }
        // The headings and the ⋯ menu sort through the same action, so the
        // radios follow its state wherever the change came from.
        let show_sort = move |key: Option<String>| {
            let key = key.unwrap_or_default();
            match radios.iter().find(|(k, _)| *k == key) {
                Some((_, r)) => r.set_active(true),
                None => radios.iter().for_each(|(_, r)| r.set_active(false)),
            }
        };
        show_sort(sort.state().and_then(|v| v.str().map(String::from)));
        sort.connect_state_notify(move |a| {
            show_sort(a.state().and_then(|v| v.str().map(String::from)))
        });

        // Show (docs/DESIGN.md, "Menus"): the list's look, under the
        // orders, in the same column as the mockup has it.
        let show_caption = gtk::Label::new(Some(&i18n("Show")));
        show_caption.set_xalign(0.0);
        show_caption.add_css_class("dim-label");
        show_caption.add_css_class("caption-heading");
        show_caption.set_margin_top(8);
        sort_col.append(&show_caption);
        let preview_lines = Rc::new(Cell::new(config::load_privacy().preview_lines.max(1)));
        let check = |label: String| {
            let c = gtk::CheckButton::with_label(&label);
            sort_col.append(&c);
            c
        };
        let threads = check(i18n("Group into conversations"));
        let preview = check(i18n("Message preview"));
        let avatars = check(i18n("Avatars"));
        {
            let s = sender.clone();
            threads.connect_toggled(move |c| {
                s.emit(AppMsg::Pref(
                    crate::ui::preferences::PrefOutput::SetThreading(c.is_active()),
                ))
            });
            let (s, lines) = (sender.clone(), preview_lines.clone());
            preview.connect_toggled(move |c| {
                let n = if c.is_active() { lines.get() } else { 0 };
                s.emit(AppMsg::Pref(
                    crate::ui::preferences::PrefOutput::SetPreviewLines(n),
                ))
            });
            let s = sender.clone();
            avatars.connect_toggled(move |c| {
                s.emit(AppMsg::Pref(
                    crate::ui::preferences::PrefOutput::SetAvatars(c.is_active()),
                ))
            });
        }

        let filter_col = column(&i18n("Show only"));
        let mut filters = Vec::new();
        for f in QuickFilter::IN_POPOVER {
            let check = gtk::CheckButton::with_label(&f.label());
            let s = sender.clone();
            check.connect_toggled(move |c| s.emit(AppMsg::SetQuickFilter(f, c.is_active())));
            filter_col.append(&check);
            filters.push((f, check));
        }
        let clear = gtk::Button::with_label(&i18n("Clear filters"));
        clear.add_css_class("flat");
        clear.set_halign(gtk::Align::Start);
        clear.set_sensitive(false);
        let s = sender.clone();
        clear.connect_clicked(move |_| s.emit(AppMsg::ClearQuickFilters));
        filter_col.append(&clear);

        let dates_col = column(&i18n("Dates"));
        let dates = gtk::Grid::new();
        dates.add_css_class("sort-filter-dates");
        dates.set_row_spacing(6);
        dates.set_column_spacing(8);
        let range = DateRange::new(sender);
        for (row, (end, button, _)) in range.ends.iter().enumerate() {
            let label = gtk::Label::new(Some(&match end {
                DateBound::From => i18n("From"),
                DateBound::To => i18n("To"),
            }));
            label.set_xalign(0.0);
            dates.attach(&label, 0, row as i32, 1, 1);
            dates.attach(button, 1, row as i32, 1, 1);
        }
        dates.attach(&range.calendar, 0, 2, 2, 1);
        dates_col.append(&dates);
        dates_col.append(&range.clear);

        let columns = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        columns.add_css_class("sort-filter-popover");
        columns.append(&sort_col);
        columns.append(&gtk::Separator::new(gtk::Orientation::Vertical));
        columns.append(&filter_col);
        columns.append(&gtk::Separator::new(gtk::Orientation::Vertical));
        columns.append(&dates_col);
        let popover = gtk::Popover::new();
        popover.set_child(Some(&columns));
        button.set_popover(Some(&popover));
        Self {
            button,
            show: [threads, preview, avatars],
            preview_lines,
            filters,
            dates: range,
            clear,
        }
    }

    /// Tick Show's three as the settings are. Setting a check to what it
    /// already is emits nothing, so this never feeds back.
    fn sync_show(&self, threading: bool, preview_lines: u32, avatars: bool) {
        if preview_lines > 0 {
            self.preview_lines.set(preview_lines);
        }
        let [threads, preview, faces] = &self.show;
        for (check, on) in [
            (threads, threading),
            (preview, preview_lines > 0),
            (faces, avatars),
        ] {
            if check.is_active() != on {
                check.set_active(on);
            }
        }
    }

    /// Show `filters` on the checks, and tint the button while any is on
    /// so a narrowed list never passes for an empty folder.
    pub(super) fn sync(&self, filters: QuickFilters) {
        for (f, check) in &self.filters {
            if check.is_active() != filters.has(*f) {
                check.set_active(filters.has(*f));
            }
        }
        self.dates.show(filters);
        self.clear.set_sensitive(!filters.is_empty());
        if filters.is_empty() {
            self.button.remove_css_class("list-filter-active");
            self.button
                .set_tooltip_text(Some(&i18n("Sort and filter messages")));
        } else {
            self.button.add_css_class("list-filter-active");
            let names: Vec<String> = QuickFilter::ALL
                .iter()
                .filter(|f| filters.has(**f))
                .map(|f| f.label())
                .chain(range_label(filters))
                .collect();
            self.button.set_tooltip_text(Some(&i18n_f(
                "Filtered: {names}",
                &[("names", &names.join(", "))],
            )));
        }
    }
}

impl AppModel {
    /// Narrow the list to `filters`, and show them on the header's popover.
    pub(super) fn set_quick_filters(&mut self, filters: QuickFilters) {
        self.quick_filters = filters;
        let contacts = if filters.has(QuickFilter::Contacts) {
            self.saved_addresses.get()
        } else {
            None
        };
        self.message_list
            .emit(MessageListInput::SetQuickFilters { filters, contacts });
        if let Some(lh) = self.list_header_widgets.get() {
            lh.filter.sync(filters);
            let name = show_toggle(
                filters.has(QuickFilter::Unread),
                filters.has(QuickFilter::Starred),
            );
            if lh.show.active_name().as_deref() != name {
                lh.show.set_active_name(name);
            }
        }
    }

    /// Where the list's search field and bulk bar open (#496).
    pub(super) fn push_list_bars(&self) {
        self.message_list.emit(MessageListInput::SetBarsPlacement {
            search_bottom: self.list_search_bottom,
            bulk_bottom: self.list_bulk_bar_bottom,
        });
    }

    /// Tell the list whether to lay messages out on one line (#334): the
    /// Layout setting, or for Automatic, the width of the pane.
    pub(super) fn push_single_line(&self) {
        // With the reader below or off the list is a table (#475).
        let on = match self.list_layout {
            _ if self.reading_pane != config::ReadingPane::Right => true,
            config::ListLayout::Cards => false,
            config::ListLayout::SingleLine => true,
            config::ListLayout::Automatic => self.list_wide,
        };
        self.message_list.emit(MessageListInput::SetSingleLine(on));
    }

    /// The sort menu's Show checks, as the settings are now.
    pub(super) fn sync_show_checks(&self) {
        if let Some(lh) = self.list_header_widgets.get() {
            lh.filter
                .sync_show(self.threading, self.preview_lines, self.avatars);
        }
    }

    pub(super) fn push_list_look(&self) {
        self.sync_show_checks();
        self.message_list.emit(MessageListInput::SetLook {
            avatars: self.avatars,
            preview_lines: self.preview_lines,
        });
        // A folded card in a conversation shows what a list row does (#326).
        self.message_view.emit(MessageViewInput::SetFoldLook {
            preview_lines: self.preview_lines,
        });
    }

    /// The list header's title: what the list shows, as the sidebar names
    /// it (docs/DESIGN.md, "Header bars"). The title is the one place that
    /// names the view once the sidebar is folded away, at narrow widths and
    /// in the rail.
    pub(super) fn list_title(&self) -> String {
        if let Some((_, keyword)) = &self.tag_view {
            let Some(keyword) = keyword else {
                return i18n("Tags");
            };
            let name = self
                .tags
                .iter()
                .find(|t| t.keyword.eq_ignore_ascii_case(keyword))
                .map_or(keyword.as_str(), |t| t.name.as_str());
            return i18n_f("Tag: {name}", &[("name", name)]);
        }
        if let Some(id) = &self.smart_view {
            return self
                .smart_folders
                .iter()
                .find(|f| &f.id == id)
                .map(|f| f.name.clone())
                .unwrap_or_default();
        }
        if self.unified {
            return match self.unified_view {
                super::types::UnifiedView::Favorites => i18n("Favorites"),
            };
        }
        self.selected
            .as_ref()
            .and_then(|s| self.folder_by_id(s.account_id, s.folder_id))
            .map(|f| f.name.clone())
            .unwrap_or_default()
    }

    /// The messages the router sends here (see `AppModel::route`).
    pub(super) fn on_list_header(&mut self, msg: AppMsg) {
        match msg {
            AppMsg::ListPaneWidth(width) => {
                // A margin each way, so a drag that rests near the line does
                // not flick the rows back and forth.
                let wide = if self.list_wide {
                    width >= LIST_WIDE_OFF
                } else {
                    width >= LIST_WIDE_ON
                };
                if wide != self.list_wide {
                    self.list_wide = wide;
                    if self.list_layout == config::ListLayout::Automatic {
                        self.push_single_line();
                    }
                }
            }

            AppMsg::ListCount(text) => self.list_count = text,
            AppMsg::ListColumnWidths(widths) => {
                pref!(self.list_column_widths = widths);
            }
            AppMsg::ListSortChanged(key) => {
                if let Some(lh) = self.list_header_widgets.get() {
                    lh.sort.set_state(&key.to_variant());
                }
            }
            _ => unreachable!("not routed to on_list_header"),
        }
    }

    /// The list header's sort menu (moved out of the message list's own
    /// toolbar): a stateful radio action feeding the list's SetSort.
    pub(super) fn init_list_sort_menu(
        &self,
        widgets: &AppModelWidgets,
        sender: &ComponentSender<Self>,
    ) {
        use crate::ui::message_list::SortOrder;
        let sort_group = gtk::gio::SimpleActionGroup::new();
        let sort_action = gtk::gio::SimpleAction::new_stateful(
            "order",
            Some(gtk::glib::VariantTy::STRING),
            &"date_newest".to_variant(),
        );
        let list = self.message_list.sender().clone();
        sort_action.connect_activate(move |action, param| {
            if let Some(key) = param.and_then(|v| v.str()) {
                action.set_state(&key.to_variant());
                let _ = list.send(MessageListInput::SetSort(SortOrder::from_key(key)));
            }
        });
        sort_group.add_action(&sort_action);
        widgets
            .list_filter_btn
            .insert_action_group("sortmenu", Some(&sort_group));

        let _ = self.list_header_widgets.set(ListHeaderWidgets {
            filter: SortFilterPopover::new(
                widgets.list_filter_btn.clone(),
                &sort_action,
                sender.input_sender(),
            ),
            sort: sort_action,
            show: build_show_switch(&widgets.list_show_bar, sender.input_sender()),
        });
        self.sync_show_checks();
        self.reader_overflow_btn
            .set_visible(self.reader_overflow_wanted());
    }
}

/// The New Message button's right-click menu: a blank message or one from a
/// template, the two entries the sidebar's compose button offered.
pub(super) fn compose_menu(
    widget: &gtk::Widget,
    x: f64,
    y: f64,
    sender: &ComponentSender<AppModel>,
) {
    use crate::ui::context_menu::{show_context_menu, MenuEntry};
    let (blank, template) = (sender.clone(), sender.clone());
    show_context_menu(
        widget,
        x,
        y,
        vec![vec![
            MenuEntry::new(i18n("New Message"), move || blank.input(AppMsg::Compose)),
            MenuEntry::new(i18n("New message from template…"), move || {
                template.input(AppMsg::NewFromTemplate);
            }),
        ]],
    );
}

/// Build the All / Unread / Starred switch into `bar`, the strip under the
/// list's header bar (docs/DESIGN.md, "Header bars"). It turns the Unread
/// and Starred quick filters on and off; the other filters stay in the sort
/// and filter popover.
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

#[cfg(test)]
mod show_switch_tests {
    use super::show_toggle;

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
}
