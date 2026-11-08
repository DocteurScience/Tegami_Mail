//! Settings → Accounts (screens 26, 27): the list of mail accounts, which
//! the preferences dialog sets at the top of its Accounts page, and the
//! page of GNOME Online Accounts that can be switched on.

use adw::prelude::*;
use relm4::prelude::*;

use super::super::{AccountsInput, AccountsWindow};
use crate::i18n::i18n;

/// The name the preferences dialog finds the list by, to move it onto its
/// Accounts page (`ui::preferences`).
pub(crate) const ACCOUNTS_LIST_NAME: &str = "accounts-list";

/// The list, the GNOME Online Accounts page, and the widgets on them that
/// the panel reaches later.
#[derive(Debug)]
pub(in crate::ui::accounts) struct AccountsPage {
    /// Each account, and "Add account…" last (`rebuild_account_list`).
    pub(in crate::ui::accounts) accounts_list: gtk::ListBox,
    pub(in crate::ui::accounts) goa_page: adw::NavigationPage,
    pub(in crate::ui::accounts) goa_refresh_btn: gtk::Button,
    pub(in crate::ui::accounts) goa_group: adw::PreferencesGroup,
    pub(in crate::ui::accounts) goa_list: gtk::ListBox,
}

impl AccountsPage {
    pub(in crate::ui::accounts) fn build(sender: &ComponentSender<AccountsWindow>) -> Self {
        let accounts_list = gtk::ListBox::new();
        accounts_list.set_widget_name(ACCOUNTS_LIST_NAME);
        accounts_list.add_css_class("boxed-list");
        accounts_list.set_selection_mode(gtk::SelectionMode::None);
        {
            let s = sender.input_sender().clone();
            accounts_list.connect_row_activated(move |_, row| {
                // "Add account…" opens its menu itself.
                if !row.has_css_class("add-row") {
                    s.emit(AccountsInput::EditAccount(row.index() as usize));
                }
            });
        }

        // The accounts GNOME has that are not here yet, each with the
        // switch that brings it in.
        let page = adw::PreferencesPage::new();
        let goa_group = adw::PreferencesGroup::new();
        goa_group.set_title(&i18n("GNOME Online Accounts"));
        goa_group.set_description(Some(&i18n(
            "Mail accounts from GNOME Settings. Switch one on to use it.",
        )));
        goa_group.set_visible(false);
        // An account just added in GNOME Settings, without waiting for
        // GOA to say so.
        let goa_refresh_btn = gtk::Button::from_icon_name("view-refresh-symbolic");
        goa_refresh_btn.set_valign(gtk::Align::Start);
        goa_refresh_btn.add_css_class("flat");
        goa_refresh_btn
            .set_tooltip_text(Some(&i18n("Check GNOME Online Accounts for new accounts")));
        {
            let s = sender.input_sender().clone();
            goa_refresh_btn.connect_clicked(move |_| s.emit(AccountsInput::RefreshGoa));
        }
        goa_group.set_header_suffix(Some(&goa_refresh_btn));
        let goa_list = gtk::ListBox::new();
        goa_list.add_css_class("boxed-list");
        goa_list.set_selection_mode(gtk::SelectionMode::None);
        goa_group.add(&goa_list);
        page.add(&goa_group);

        let more = adw::PreferencesGroup::new();
        more.set_description(Some(&i18n(
            "Accounts added in GNOME Settings → Online Accounts appear here.",
        )));
        let open = adw::ButtonRow::new();
        open.set_title(&i18n("Open Online Accounts"));
        open.set_end_icon_name(Some("adw-external-link-symbolic"));
        {
            let s = sender.input_sender().clone();
            open.connect_activated(move |_| s.emit(AccountsInput::OpenOnlineAccounts));
        }
        more.add(&open);
        page.add(&more);

        let view = adw::ToolbarView::new();
        view.add_top_bar(&adw::HeaderBar::new());
        view.set_content(Some(&page));
        let goa_page = adw::NavigationPage::new(&view, &i18n("GNOME Online Accounts"));

        Self {
            accounts_list,
            goa_page,
            goa_refresh_btn,
            goa_group,
            goa_list,
        }
    }
}
