//! The message list's header: the Sort and display popover (mockup 18),
//! and the wide/narrow switch. The Filters menu, its chips and the All /
//! Unread / Starred switch are in `list_filters`.

use adw::prelude::*;
use relm4::prelude::*;
use std::cell::Cell;
use std::rc::Rc;

use crate::config;
use crate::i18n::{i18n, i18n_f, i18n_noop};
use crate::ui::message_list::{MessageListInput, QuickFilter, QuickFilters};
use crate::ui::message_view::MessageViewInput;

use super::list_filters::FiltersUi;
use super::model::AppModel;
use super::msg::AppMsg;
use super::AppModelWidgets;

/// The message list header's Sort and display popover, its sort action,
/// and the filter widgets, kept to be set from elsewhere.
pub(super) struct ListHeaderWidgets {
    sort_display: SortDisplayPopover,
    sort_button: gtk::MenuButton,
    sort: gtk::gio::SimpleAction,
    pub(super) filters: FiltersUi,
}

/// The orders the list header offers, by their keys in `SortOrder`. The
/// column headings can set more (#443); none of these is ticked then.
const SORT_ORDERS: [(&str, &str); 4] = [
    (i18n_noop("Date (Newest first)"), "date_newest"),
    (i18n_noop("Date (Oldest first)"), "date_oldest"),
    (i18n_noop("Sender (A–Z)"), "sender"),
    (i18n_noop("Subject (A–Z)"), "subject"),
];

/// The list header's Sort and display popover (docs/DESIGN.md, "Sorting,
/// display and filters"; mockup 18): one column, the four orders as radios
/// under Sort by, then Show's three checks. Seven items, about 290 px: the
/// HIG's small popover, where the old one mixed twenty controls of three
/// kinds and a calendar.
struct SortDisplayPopover {
    /// Show: Group into conversations, Message preview, Avatars. Each sets
    /// its setting; `sync_show` puts the setting back on it when it changes
    /// elsewhere.
    show: [gtk::CheckButton; 3],
    /// The preview lines to come back to when Message preview is ticked
    /// again: the last count above none.
    preview_lines: Rc<Cell<u32>>,
}

/// A section's heading in a header popover, as libadwaita's menus write
/// them.
pub(super) fn popover_heading(text: &str) -> gtk::Label {
    let caption = gtk::Label::new(Some(text));
    caption.set_xalign(0.0);
    caption.add_css_class("heading");
    caption.add_css_class("popover-heading");
    caption
}

impl SortDisplayPopover {
    /// The popover stays open as rows are clicked, so an order and the
    /// look can be set in one visit.
    fn new(
        button: &gtk::MenuButton,
        sort: &gtk::gio::SimpleAction,
        sender: &relm4::Sender<AppMsg>,
    ) -> Self {
        let column = gtk::Box::new(gtk::Orientation::Vertical, 0);
        column.add_css_class("header-popover");
        column.append(&popover_heading(&i18n("Sort by")));
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
            column.append(&radio);
            radios.push((key, radio));
        }
        // The headings sort through the same action, so the radios follow
        // its state wherever the change came from.
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

        column.append(&gtk::Separator::new(gtk::Orientation::Horizontal));
        column.append(&popover_heading(&i18n("Show")));
        let preview_lines = Rc::new(Cell::new(config::load_privacy().preview_lines.max(1)));
        let check = |label: String| {
            let c = gtk::CheckButton::with_label(&label);
            column.append(&c);
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

        let popover = gtk::Popover::new();
        popover.set_child(Some(&column));
        button.set_popover(Some(&popover));
        Self {
            show: [threads, preview, avatars],
            preview_lines,
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
}

impl AppModel {
    /// Narrow the list to `filters`, and show them on the Filters menu,
    /// its chips and the switch.
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
            lh.filters.sync(filters, &self.list_count);
        }
    }

    /// Tell the list whether to lay messages out on one line (#334): with
    /// the reader beside it the list is cards, with the reader below or off
    /// it is a table (#475).
    pub(super) fn push_single_line(&self) {
        let on = self.reading_pane != config::ReadingPane::Right;
        self.message_list.emit(MessageListInput::SetSingleLine(on));
    }

    /// The Sort and display popover's Show checks, as the settings are
    /// now.
    pub(super) fn sync_show_checks(&self) {
        if let Some(lh) = self.list_header_widgets.get() {
            lh.sort_display
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
            AppMsg::ListCount(text) => {
                // The match count under the filter chips follows the
                // header's.
                if let Some(lh) = self.list_header_widgets.get() {
                    lh.filters.sync(self.quick_filters, &text);
                }
                self.list_count = text;
            }
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

    /// Demo only (TEGAMI_SHOWCASE_LIST): the screens of mockups 18 to 21.
    pub(super) fn showcase_list_filters(&mut self, which: &str, sender: &ComponentSender<Self>) {
        use crate::ui::message_list::Period;
        if !super::demo::demo_mode() {
            return;
        }
        let Some(lh) = self.list_header_widgets.get() else {
            return;
        };
        let (sort_button, filters_button) = (lh.sort_button.clone(), lh.filters.button.clone());
        if which == "sort" {
            sort_button.popup();
            return;
        }
        self.message_list.emit(MessageListInput::FocusSearch);
        match which {
            "active" => self.set_quick_filters(
                self.quick_filters
                    .with(QuickFilter::Attachment, true)
                    .with_period(Period::Last30Days),
            ),
            "period" => {
                let day = |d| chrono::NaiveDate::from_ymd_opt(2025, 10, d);
                self.set_quick_filters(self.quick_filters.with_period(Period::Custom {
                    from: day(2),
                    to: day(9),
                }));
                self.open_custom_period(sender);
            }
            _ => {
                gtk::glib::timeout_add_local_once(
                    std::time::Duration::from_millis(600),
                    move || filters_button.popup(),
                );
            }
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

        let filters = FiltersUi::new(sender.input_sender());
        let pane = self.message_list.widget().clone().upcast::<gtk::Widget>();
        for (name, child) in [
            (
                crate::ui::message_list::FILTERS_SLOT,
                filters.button.clone().upcast::<gtk::Widget>(),
            ),
            (
                crate::ui::message_list::FILTER_AREA_SLOT,
                filters.area.clone().upcast(),
            ),
        ] {
            if let Some(slot) = named_descendant(&pane, name).and_downcast::<gtk::Box>() {
                slot.append(&child);
            }
        }
        let _ = self.list_header_widgets.set(ListHeaderWidgets {
            sort_display: SortDisplayPopover::new(
                &widgets.list_filter_btn,
                &sort_action,
                sender.input_sender(),
            ),
            sort_button: widgets.list_filter_btn.clone(),
            sort: sort_action,
            filters,
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

/// The first widget under `root`, itself included, named `name`.
fn named_descendant(root: &gtk::Widget, name: &str) -> Option<gtk::Widget> {
    if root.widget_name() == name {
        return Some(root.clone());
    }
    let mut child = root.first_child();
    while let Some(c) = child {
        if let Some(found) = named_descendant(&c, name) {
            return Some(found);
        }
        child = c.next_sibling();
    }
    None
}
