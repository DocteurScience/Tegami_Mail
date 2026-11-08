//! Settings → Smart folders (#417; screen 39): the saved searches, each
//! opening its editor, and "Add smart folder…" at the end of the list.

use adw::prelude::*;

use crate::i18n::i18n;

/// The page, and the widgets on it that the panel reaches later.
#[derive(Debug)]
pub(in crate::ui::accounts) struct SmartPage {
    pub(in crate::ui::accounts) page: adw::PreferencesPage,
    pub(in crate::ui::accounts) smart_list: gtk::ListBox,
}

impl SmartPage {
    pub(in crate::ui::accounts) fn build() -> Self {
        let page = adw::PreferencesPage::new();
        let group = adw::PreferencesGroup::new();
        group.set_description(Some(&i18n(
            "Shows the messages that match its criteria. In the sidebar, a collapsible “Smart folders” section follows the unified inboxes; it appears only when there is at least one.",
        )));
        let smart_list = gtk::ListBox::new();
        smart_list.add_css_class("boxed-list");
        smart_list.set_selection_mode(gtk::SelectionMode::None);
        group.add(&smart_list);
        page.add(&group);
        Self { page, smart_list }
    }
}
