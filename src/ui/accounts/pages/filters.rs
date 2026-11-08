//! Settings → Filters (screen 37): the rules, each opening its editor,
//! "Add filter…" at the end of the list, and Apply now below it. The
//! search box shows once the list is long.

use adw::prelude::*;
use relm4::prelude::*;

use super::super::{AccountsInput, AccountsWindow};
use super::form::action_row;
use crate::i18n::i18n;

/// Past this many rules the list gets its search box (#141).
pub(in crate::ui::accounts) const SEARCH_FROM: usize = 8;

/// The page, and the widgets on it that the panel reaches later.
#[derive(Debug)]
pub(in crate::ui::accounts) struct FiltersPage {
    pub(in crate::ui::accounts) page: adw::PreferencesPage,
    pub(in crate::ui::accounts) search_group: adw::PreferencesGroup,
    pub(in crate::ui::accounts) apply_filters_spinner: gtk::Spinner,
    pub(in crate::ui::accounts) apply_filters_row: adw::ActionRow,
    pub(in crate::ui::accounts) filters_list: gtk::ListBox,
}

impl FiltersPage {
    pub(in crate::ui::accounts) fn build(sender: &ComponentSender<AccountsWindow>) -> Self {
        let page = adw::PreferencesPage::new();

        // A search box over a long list (#141): rows that do not match the
        // text are hidden.
        let search_group = adw::PreferencesGroup::new();
        search_group.set_visible(false);
        let search = gtk::SearchEntry::new();
        search.set_placeholder_text(Some(&i18n("Search filters")));
        {
            let s = sender.input_sender().clone();
            search.connect_search_changed(move |e| {
                s.emit(AccountsInput::SearchFilters(e.text().to_string()))
            });
        }
        search_group.add(&search);
        page.add(&search_group);

        let rules = adw::PreferencesGroup::new();
        rules.set_description(Some(&i18n(
            "File or tag mail as it arrives. Filters run on each Inbox as it syncs.",
        )));
        let filters_list = gtk::ListBox::new();
        filters_list.add_css_class("boxed-list");
        filters_list.set_selection_mode(gtk::SelectionMode::None);
        rules.add(&filters_list);
        page.add(&rules);

        // Rules normally meet mail as it arrives; this holds them up
        // against what is already in the inboxes (#198). A spinner turns
        // while the run is under way.
        let apply = adw::PreferencesGroup::new();
        let apply_filters_row = action_row(&i18n("Apply now"), None, "add-row");
        apply_filters_row.set_subtitle(&i18n(
            "Runs every rule over the mail already in your inboxes.",
        ));
        let apply_filters_spinner = gtk::Spinner::new();
        apply_filters_spinner.set_visible(false);
        apply_filters_row.add_suffix(&apply_filters_spinner);
        {
            let s = sender.input_sender().clone();
            apply_filters_row.connect_activated(move |_| s.emit(AccountsInput::ApplyFilters));
        }
        apply.add(&apply_filters_row);
        page.add(&apply);

        Self {
            page,
            search_group,
            apply_filters_spinner,
            apply_filters_row,
            filters_list,
        }
    }
}
