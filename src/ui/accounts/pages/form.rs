//! The account page (docs/DESIGN.md, "Accounts"; screen 28) and its
//! sub-pages: servers and connection (29), synchronisation (30), folders
//! (31), send-as aliases (32), signatures (33) and OpenPGP (34). Filled
//! and read by `form`, wired by `setup`.
//!
//! The account page is short: who the account is, its colour, and one row
//! per sub-page. The rare fields sit under "Advanced" on their sub-page.
//! Every sub-page is pushed over the preferences dialog, so its back button
//! returns to the account page, whose Save reads them all.

use adw::prelude::*;
use relm4::prelude::*;

use super::super::form::{REPLY_COPIES_HINT, SENT_COPY_HINT};
use super::super::{AccountsInput, AccountsWindow};
use crate::i18n::i18n;

/// The page, its sub-pages, and the widgets on them that the panel
/// reaches later.
#[derive(Debug)]
pub(in crate::ui::accounts) struct EditorForm {
    /// The account page as the dialog shows it: its header carries Save
    /// (kept by the owner, 2026-11-08); the sub-pages have Back only.
    pub(in crate::ui::accounts) editor_page: adw::NavigationPage,
    pub(in crate::ui::accounts) goa_banner: adw::PreferencesGroup,
    pub(in crate::ui::accounts) goa_enabled_row: adw::SwitchRow,
    pub(in crate::ui::accounts) quota_group: adw::PreferencesGroup,
    pub(in crate::ui::accounts) quota_row: adw::ActionRow,
    pub(in crate::ui::accounts) quota_bar: gtk::LevelBar,
    pub(in crate::ui::accounts) label_row: adw::EntryRow,
    pub(in crate::ui::accounts) name_row: adw::EntryRow,
    pub(in crate::ui::accounts) email_row: adw::EntryRow,
    pub(in crate::ui::accounts) color_btn: gtk::ColorDialogButton,
    pub(in crate::ui::accounts) in_unified_row: adw::SwitchRow,
    // The rows that open the sub-pages, and the sub-pages.
    pub(in crate::ui::accounts) servers_link: adw::ActionRow,
    pub(in crate::ui::accounts) aliases_link: adw::ActionRow,
    pub(in crate::ui::accounts) signatures_link: adw::ActionRow,
    pub(in crate::ui::accounts) openpgp_link: adw::ActionRow,
    pub(in crate::ui::accounts) remove_group: adw::PreferencesGroup,
    /// The sub-pages, for what is set on all of them (`prepare_editor_page`).
    pub(in crate::ui::accounts) sub_pages: Vec<adw::NavigationPage>,
    // Servers and connection.
    pub(in crate::ui::accounts) provider_group: adw::PreferencesGroup,
    pub(in crate::ui::accounts) incoming_group: adw::PreferencesGroup,
    pub(in crate::ui::accounts) advanced_row: adw::ExpanderRow,
    pub(in crate::ui::accounts) provider_mark: gtk::Box,
    pub(in crate::ui::accounts) provider_row: adw::ComboRow,
    pub(in crate::ui::accounts) protocol_row: adw::ComboRow,
    pub(in crate::ui::accounts) host_row: adw::EntryRow,
    pub(in crate::ui::accounts) port_row: adw::EntryRow,
    pub(in crate::ui::accounts) smtp_row: adw::EntryRow,
    pub(in crate::ui::accounts) smtp_port_row: adw::EntryRow,
    pub(in crate::ui::accounts) user_row: adw::EntryRow,
    pub(in crate::ui::accounts) pass_row: adw::PasswordEntryRow,
    pub(in crate::ui::accounts) jmap_token_row: adw::SwitchRow,
    pub(in crate::ui::accounts) oauth_client_id_row: adw::EntryRow,
    pub(in crate::ui::accounts) oauth_secret_row: adw::PasswordEntryRow,
    pub(in crate::ui::accounts) oauth_auth_url_row: adw::EntryRow,
    pub(in crate::ui::accounts) oauth_token_url_row: adw::EntryRow,
    pub(in crate::ui::accounts) oauth_scope_row: adw::EntryRow,
    pub(in crate::ui::accounts) ms_client_row: adw::EntryRow,
    pub(in crate::ui::accounts) ms_tenant_row: adw::EntryRow,
    pub(in crate::ui::accounts) ms_scopes_row: adw::EntryRow,
    pub(in crate::ui::accounts) ms_redirect_row: adw::EntryRow,
    pub(in crate::ui::accounts) oauth_signin_btn: gtk::Button,
    pub(in crate::ui::accounts) oauth_status: gtk::Label,
    pub(in crate::ui::accounts) goa_hint: gtk::Box,
    pub(in crate::ui::accounts) smtp_separate_row: adw::SwitchRow,
    pub(in crate::ui::accounts) smtp_user_row: adw::EntryRow,
    pub(in crate::ui::accounts) smtp_pass_row: adw::PasswordEntryRow,
    pub(in crate::ui::accounts) tls_mismatch_row: adw::SwitchRow,
    pub(in crate::ui::accounts) test_group: adw::PreferencesGroup,
    pub(in crate::ui::accounts) test_btn: adw::ActionRow,
    pub(in crate::ui::accounts) test_result: adw::ActionRow,
    pub(in crate::ui::accounts) test_icon: gtk::Image,
    pub(in crate::ui::accounts) graph_group: adw::PreferencesGroup,
    pub(in crate::ui::accounts) graph_directory_row: adw::SwitchRow,
    pub(in crate::ui::accounts) directory_only_row: adw::SwitchRow,
    pub(in crate::ui::accounts) graph_status: gtk::Label,
    // Synchronisation.
    pub(in crate::ui::accounts) push_group: adw::PreferencesGroup,
    pub(in crate::ui::accounts) push_row: adw::ComboRow,
    pub(in crate::ui::accounts) empty_junk_row: adw::ComboRow,
    pub(in crate::ui::accounts) empty_trash_row: adw::ComboRow,
    pub(in crate::ui::accounts) offline_group: adw::PreferencesGroup,
    pub(in crate::ui::accounts) offline_mail_row: adw::ComboRow,
    pub(in crate::ui::accounts) offline_attachments_row: adw::SwitchRow,
    pub(in crate::ui::accounts) copy_local_row: adw::SwitchRow,
    // Folders.
    pub(in crate::ui::accounts) folder_sort_row: adw::ComboRow,
    pub(in crate::ui::accounts) folder_sent_row: adw::ComboRow,
    pub(in crate::ui::accounts) folder_sent_copy_row: adw::ComboRow,
    pub(in crate::ui::accounts) reply_copies_row: adw::ComboRow,
    pub(in crate::ui::accounts) server_saves_row: adw::SwitchRow,
    pub(in crate::ui::accounts) folder_drafts_row: adw::ComboRow,
    pub(in crate::ui::accounts) folder_templates_row: adw::ComboRow,
    pub(in crate::ui::accounts) folder_trash_row: adw::ComboRow,
    pub(in crate::ui::accounts) folder_junk_row: adw::ComboRow,
    pub(in crate::ui::accounts) folder_archive_row: adw::ComboRow,
    pub(in crate::ui::accounts) folder_snoozed_row: adw::ComboRow,
    pub(in crate::ui::accounts) hidden_list: gtk::ListBox,
    pub(in crate::ui::accounts) hidden_empty: gtk::Label,
    // Aliases and signatures.
    pub(in crate::ui::accounts) aliases_list: gtk::ListBox,
    pub(in crate::ui::accounts) sig_choice_row: adw::ComboRow,
    pub(in crate::ui::accounts) sig_name_row: adw::EntryRow,
    pub(in crate::ui::accounts) sig_default_row: adw::SwitchRow,
    pub(in crate::ui::accounts) sig_delete_row: adw::ActionRow,
    pub(in crate::ui::accounts) sig_holder: gtk::Box,
    // OpenPGP.
    pub(in crate::ui::accounts) pgp_key_row: adw::ComboRow,
    pub(in crate::ui::accounts) sign_default_row: adw::SwitchRow,
    pub(in crate::ui::accounts) hide_subject_row: adw::SwitchRow,
}

/// A titled group, with a description when one is given.
fn group(title: &str, description: Option<&str>) -> adw::PreferencesGroup {
    let g = adw::PreferencesGroup::new();
    if !title.is_empty() {
        g.set_title(title);
    }
    if let Some(d) = description {
        g.set_description(Some(d));
    }
    g
}

fn entry(title: &str) -> adw::EntryRow {
    let row = adw::EntryRow::new();
    row.set_title(title);
    row
}

fn password(title: &str) -> adw::PasswordEntryRow {
    let row = adw::PasswordEntryRow::new();
    row.set_title(title);
    row
}

fn switch(title: &str, subtitle: &str) -> adw::SwitchRow {
    let row = adw::SwitchRow::new();
    row.set_title(title);
    if !subtitle.is_empty() {
        row.set_subtitle(subtitle);
    }
    row
}

fn combo(title: &str, subtitle: &str) -> adw::ComboRow {
    let row = adw::ComboRow::new();
    row.set_title(title);
    if !subtitle.is_empty() {
        row.set_subtitle(subtitle);
    }
    row
}

/// A row with a chevron, opening `sub` over the dialog it is in.
fn link(title: &str, sub: &adw::NavigationPage) -> adw::ActionRow {
    let row = adw::ActionRow::new();
    row.set_title(title);
    row.set_activatable(true);
    let next = gtk::Image::from_icon_name("go-next-symbolic");
    next.set_accessible_role(gtk::AccessibleRole::Presentation);
    row.add_suffix(&next);
    let sub = sub.clone();
    row.connect_activated(move |row| {
        if let Some(dialog) = row
            .ancestor(adw::PreferencesDialog::static_type())
            .and_downcast::<adw::PreferencesDialog>()
        {
            dialog.push_subpage(&sub);
        }
    });
    row
}

/// A sub-page: a header with the back button over `page`.
fn sub_page(title: &str, page: &adw::PreferencesPage) -> adw::NavigationPage {
    let view = adw::ToolbarView::new();
    view.add_top_bar(&adw::HeaderBar::new());
    view.set_content(Some(page));
    adw::NavigationPage::new(&view, title)
}

/// A row that acts rather than sets, as screens 26 and 29 to 33 draw them:
/// its title in the accent colour (`add-row`), or the destructive one
/// (`error`), with an icon before it when one is given. Left-aligned,
/// where an `AdwButtonRow` centres its title.
pub(crate) fn action_row(title: &str, icon: Option<&str>, class: &str) -> adw::ActionRow {
    let row = adw::ActionRow::new();
    row.set_title(title);
    row.set_activatable(true);
    row.add_css_class(class);
    if let Some(icon) = icon {
        let image = gtk::Image::from_icon_name(icon);
        image.set_accessible_role(gtk::AccessibleRole::Presentation);
        row.add_prefix(&image);
    }
    row
}

/// A row that adds something: "+" and its title in the accent colour.
pub(crate) fn add_row(title: &str) -> adw::ActionRow {
    action_row(title, Some("list-add-symbolic"), "add-row")
}

fn emit(
    row: &adw::ActionRow,
    sender: &ComponentSender<AccountsWindow>,
    msg: fn() -> AccountsInput,
) {
    let s = sender.input_sender().clone();
    row.connect_activated(move |_| s.emit(msg()));
}

impl EditorForm {
    pub(in crate::ui::accounts) fn build(sender: &ComponentSender<AccountsWindow>) -> Self {
        let servers = Self::servers(sender);
        let sync = Self::sync();
        let folders = Self::folders(sender);
        let (aliases_page, aliases_list) = Self::aliases(sender);
        let sigs = Self::signatures(sender);
        let (openpgp_page, pgp_key_row, sign_default_row, hide_subject_row) = Self::openpgp(sender);

        let page = adw::PreferencesPage::new();

        // GNOME Online Accounts owns this account's servers and
        // credentials; Tegami Mail only mirrors them, and can hide it
        // locally. Both facts belong together, above the fields they
        // explain.
        let goa_banner = group(&i18n("GNOME Online Account"), None);
        goa_banner.set_visible(false);
        let goa_enabled_row = switch(
            &i18n("Show in Tegami Mail"),
            &i18n("Off returns it to the import list. It stays in GNOME Online Accounts."),
        );
        {
            let s = sender.input_sender().clone();
            goa_enabled_row.connect_active_notify(move |row| {
                s.emit(AccountsInput::ToggleCurrentEnabled(row.is_active()))
            });
        }
        goa_banner.add(&goa_enabled_row);
        let goa_row = adw::ActionRow::new();
        goa_row.set_title(&i18n("Managed by GNOME Online Accounts"));
        goa_row.set_subtitle(&i18n(
            "Its address, servers and password are changed in Settings \u{2192} Online Accounts.",
        ));
        let open_goa = gtk::Button::with_label(&i18n("Open Online Accounts"));
        open_goa.set_valign(gtk::Align::Center);
        {
            let s = sender.input_sender().clone();
            open_goa.connect_clicked(move |_| s.emit(AccountsInput::OpenOnlineAccounts));
        }
        goa_row.add_suffix(&open_goa);
        goa_banner.add(&goa_row);
        page.add(&goa_banner);

        // Who the account is (screen 28).
        let identity = group("", None);
        let name_row = entry(&i18n("Display name"));
        let email_row = entry(&i18n("Email address"));
        email_row.set_input_purpose(gtk::InputPurpose::Email);
        // The nickname names the account in the sidebar and lists, the
        // address standing in while there is none (owner, 2026-11-08).
        let label_row = entry(&i18n("Nickname"));
        let color_row = adw::ActionRow::new();
        color_row.set_title(&i18n("Account colour"));
        color_row.set_subtitle(&i18n("Accent of this account’s badges and outlines."));
        let color_btn = gtk::ColorDialogButton::new(Some(gtk::ColorDialog::new()));
        color_btn.set_valign(gtk::Align::Center);
        color_btn.update_property(&[gtk::accessible::Property::Label(&i18n("Account colour"))]);
        color_row.add_suffix(&color_btn);
        color_row.set_activatable_widget(Some(&color_btn));
        // Off keeps a mailbox read on its own out of the merged lists
        // (#267); its own section, the tray and notifications are
        // unchanged.
        let in_unified_row = switch(
            &i18n("Show in combined views"),
            &i18n("Favorites, tags and smart folders from all accounts."),
        );
        in_unified_row.set_active(true);
        for row in [
            name_row.upcast_ref::<gtk::Widget>(),
            email_row.upcast_ref(),
            label_row.upcast_ref(),
            color_row.upcast_ref(),
            in_unified_row.upcast_ref(),
        ] {
            identity.add(row);
        }
        page.add(&identity);

        // The mailbox's storage (#298): shown once the server has said how
        // much is in use, never for one that keeps no limit or cannot say.
        let quota_group = group("", None);
        quota_group.set_visible(false);
        let quota_row = adw::ActionRow::new();
        quota_row.set_title(&i18n("Storage"));
        quota_row.add_css_class("property");
        let quota_bar = gtk::LevelBar::new();
        quota_bar.set_valign(gtk::Align::Center);
        quota_bar.set_width_request(160);
        quota_bar.set_min_value(0.0);
        quota_bar.set_max_value(1.0);
        quota_row.add_suffix(&quota_bar);
        quota_group.add(&quota_row);
        page.add(&quota_group);

        let links = group("", None);
        let servers_link = link(&i18n("Servers and connection"), &servers.page);
        let sync_link = link(&i18n("Synchronisation"), &sync.page);
        sync_link.set_subtitle(&i18n("Instant mail, automatic cleanup"));
        let folders_link = link(&i18n("Folders"), &folders.page);
        folders_link.set_subtitle(&i18n("Sent, drafts, trash, archive"));
        let aliases_link = link(&i18n("Send-as aliases"), &aliases_page);
        let signatures_link = link(&i18n("Signatures"), &sigs.page);
        let openpgp_link = link(&i18n("OpenPGP encryption"), &openpgp_page);
        for row in [
            &servers_link,
            &sync_link,
            &folders_link,
            &aliases_link,
            &signatures_link,
            &openpgp_link,
        ] {
            links.add(row);
        }
        page.add(&links);

        // In place of the header's Remove button: the one destructive
        // action, last (screen 28). For an account from GNOME Online
        // Accounts it removes it from Tegami Mail only.
        let remove_group = group("", None);
        let remove_row = action_row(
            &i18n("Remove account"),
            Some("user-trash-symbolic"),
            "error",
        );
        remove_row.set_subtitle(&i18n(
            "Also removes its saved password. Mail on the server is not touched.",
        ));
        {
            let s = sender.input_sender().clone();
            remove_row.connect_activated(move |_| s.emit(AccountsInput::RemoveCurrent));
        }
        remove_group.add(&remove_row);
        page.add(&remove_group);

        let sub_pages = vec![
            servers.page.clone(),
            sync.page.clone(),
            folders.page.clone(),
            aliases_page.clone(),
            sigs.page.clone(),
            openpgp_page.clone(),
        ];
        let header = adw::HeaderBar::new();
        let save = gtk::Button::with_label(&i18n("Save"));
        save.add_css_class("suggested-action");
        {
            let s = sender.input_sender().clone();
            save.connect_clicked(move |_| s.emit(AccountsInput::Save));
        }
        header.pack_end(&save);
        let view = adw::ToolbarView::new();
        view.add_top_bar(&header);
        view.set_content(Some(&page));
        let editor_page = adw::NavigationPage::new(&view, &i18n("Account"));

        Self {
            editor_page,
            goa_banner,
            goa_enabled_row,
            quota_group,
            quota_row,
            quota_bar,
            label_row,
            name_row,
            email_row,
            color_btn,
            in_unified_row,
            servers_link,
            aliases_link,
            signatures_link,
            openpgp_link,
            remove_group,
            sub_pages,
            provider_group: servers.provider_group,
            incoming_group: servers.incoming_group,
            advanced_row: servers.advanced_row,
            provider_mark: servers.provider_mark,
            provider_row: servers.provider_row,
            protocol_row: servers.protocol_row,
            host_row: servers.host_row,
            port_row: servers.port_row,
            smtp_row: servers.smtp_row,
            smtp_port_row: servers.smtp_port_row,
            user_row: servers.user_row,
            pass_row: servers.pass_row,
            jmap_token_row: servers.jmap_token_row,
            oauth_client_id_row: servers.oauth_client_id_row,
            oauth_secret_row: servers.oauth_secret_row,
            oauth_auth_url_row: servers.oauth_auth_url_row,
            oauth_token_url_row: servers.oauth_token_url_row,
            oauth_scope_row: servers.oauth_scope_row,
            ms_client_row: servers.ms_client_row,
            ms_tenant_row: servers.ms_tenant_row,
            ms_scopes_row: servers.ms_scopes_row,
            ms_redirect_row: servers.ms_redirect_row,
            oauth_signin_btn: servers.oauth_signin_btn,
            oauth_status: servers.oauth_status,
            goa_hint: servers.goa_hint,
            smtp_separate_row: servers.smtp_separate_row,
            smtp_user_row: servers.smtp_user_row,
            smtp_pass_row: servers.smtp_pass_row,
            tls_mismatch_row: servers.tls_mismatch_row,
            test_group: servers.test_group,
            test_btn: servers.test_btn,
            test_result: servers.test_result,
            test_icon: servers.test_icon,
            graph_group: servers.graph_group,
            graph_directory_row: servers.graph_directory_row,
            directory_only_row: servers.directory_only_row,
            graph_status: servers.graph_status,
            push_group: sync.push_group,
            push_row: sync.push_row,
            empty_junk_row: sync.empty_junk_row,
            empty_trash_row: sync.empty_trash_row,
            offline_group: sync.offline_group,
            offline_mail_row: sync.offline_mail_row,
            offline_attachments_row: sync.offline_attachments_row,
            copy_local_row: sync.copy_local_row,
            folder_sort_row: folders.folder_sort_row,
            folder_sent_row: folders.folder_sent_row,
            folder_sent_copy_row: folders.folder_sent_copy_row,
            reply_copies_row: folders.reply_copies_row,
            server_saves_row: folders.server_saves_row,
            folder_drafts_row: folders.folder_drafts_row,
            folder_templates_row: folders.folder_templates_row,
            folder_trash_row: folders.folder_trash_row,
            folder_junk_row: folders.folder_junk_row,
            folder_archive_row: folders.folder_archive_row,
            folder_snoozed_row: folders.folder_snoozed_row,
            hidden_list: folders.hidden_list,
            hidden_empty: folders.hidden_empty,
            aliases_list,
            sig_choice_row: sigs.choice_row,
            sig_name_row: sigs.name_row,
            sig_default_row: sigs.default_row,
            sig_delete_row: sigs.delete_row,
            sig_holder: sigs.holder,
            pgp_key_row,
            sign_default_row,
            hide_subject_row,
        }
    }
}

/// Servers and connection (screen 29).
struct ServersPage {
    page: adw::NavigationPage,
    provider_group: adw::PreferencesGroup,
    incoming_group: adw::PreferencesGroup,
    advanced_row: adw::ExpanderRow,
    provider_mark: gtk::Box,
    provider_row: adw::ComboRow,
    protocol_row: adw::ComboRow,
    host_row: adw::EntryRow,
    port_row: adw::EntryRow,
    smtp_row: adw::EntryRow,
    smtp_port_row: adw::EntryRow,
    user_row: adw::EntryRow,
    pass_row: adw::PasswordEntryRow,
    jmap_token_row: adw::SwitchRow,
    oauth_client_id_row: adw::EntryRow,
    oauth_secret_row: adw::PasswordEntryRow,
    oauth_auth_url_row: adw::EntryRow,
    oauth_token_url_row: adw::EntryRow,
    oauth_scope_row: adw::EntryRow,
    ms_client_row: adw::EntryRow,
    ms_tenant_row: adw::EntryRow,
    ms_scopes_row: adw::EntryRow,
    ms_redirect_row: adw::EntryRow,
    oauth_signin_btn: gtk::Button,
    oauth_status: gtk::Label,
    goa_hint: gtk::Box,
    smtp_separate_row: adw::SwitchRow,
    smtp_user_row: adw::EntryRow,
    smtp_pass_row: adw::PasswordEntryRow,
    tls_mismatch_row: adw::SwitchRow,
    test_group: adw::PreferencesGroup,
    test_btn: adw::ActionRow,
    test_result: adw::ActionRow,
    test_icon: gtk::Image,
    graph_group: adw::PreferencesGroup,
    graph_directory_row: adw::SwitchRow,
    directory_only_row: adw::SwitchRow,
    graph_status: gtk::Label,
}

/// Synchronisation (screen 30).
struct SyncPage {
    page: adw::NavigationPage,
    push_group: adw::PreferencesGroup,
    push_row: adw::ComboRow,
    empty_junk_row: adw::ComboRow,
    empty_trash_row: adw::ComboRow,
    offline_group: adw::PreferencesGroup,
    offline_mail_row: adw::ComboRow,
    offline_attachments_row: adw::SwitchRow,
    copy_local_row: adw::SwitchRow,
}

/// Folders (screen 31).
struct FoldersPage {
    page: adw::NavigationPage,
    folder_sort_row: adw::ComboRow,
    folder_sent_row: adw::ComboRow,
    folder_sent_copy_row: adw::ComboRow,
    reply_copies_row: adw::ComboRow,
    server_saves_row: adw::SwitchRow,
    folder_drafts_row: adw::ComboRow,
    folder_templates_row: adw::ComboRow,
    folder_trash_row: adw::ComboRow,
    folder_junk_row: adw::ComboRow,
    folder_archive_row: adw::ComboRow,
    folder_snoozed_row: adw::ComboRow,
    hidden_list: gtk::ListBox,
    hidden_empty: gtk::Label,
}

/// Signatures (screen 33).
struct SignaturesPage {
    page: adw::NavigationPage,
    choice_row: adw::ComboRow,
    name_row: adw::EntryRow,
    default_row: adw::SwitchRow,
    delete_row: adw::ActionRow,
    holder: gtk::Box,
}

impl EditorForm {
    fn servers(sender: &ComponentSender<AccountsWindow>) -> ServersPage {
        let page = adw::PreferencesPage::new();

        // The provider's mark, following the Provider picker (a protocol
        // tile for manual IMAP and custom OAuth), over the picker itself
        // while an account is being added. Once it exists the provider is
        // a rare change, and moves under Advanced (`place_provider`).
        let provider_group = group("", None);
        let provider_mark = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        provider_mark.set_halign(gtk::Align::Center);
        provider_mark.set_height_request(56);
        provider_mark.set_margin_bottom(12);
        provider_group.add(&provider_mark);
        // The rest of the page adapts to it (server fields or OAuth
        // sign-in).
        let provider_row = combo(&i18n("Provider"), "");
        {
            let s = sender.input_sender().clone();
            provider_row.connect_selected_notify(move |_| s.emit(AccountsInput::ProviderChanged));
        }
        page.add(&provider_group);

        let incoming_group = group(
            &i18n("Incoming"),
            Some(&i18n(
                "The password is kept in the system keyring, never in plain text.",
            )),
        );
        let protocol_row = combo(&i18n("Protocol"), "");
        {
            let s = sender.input_sender().clone();
            protocol_row.connect_selected_notify(move |_| s.emit(AccountsInput::ProtocolChanged));
        }
        let host_row = entry(&i18n("Server"));
        let port_row = entry(&i18n("Port"));
        port_row.set_input_purpose(gtk::InputPurpose::Digits);
        let user_row = entry(&i18n("User name"));
        let pass_row = password(&i18n("Password"));
        for row in [
            protocol_row.upcast_ref::<gtk::Widget>(),
            host_row.upcast_ref(),
            port_row.upcast_ref(),
            user_row.upcast_ref(),
            pass_row.upcast_ref(),
        ] {
            incoming_group.add(row);
        }

        // OAuth: the user just signs in.
        let oauth_signin_btn = gtk::Button::with_label(&i18n("Sign in with browser"));
        oauth_signin_btn.set_halign(gtk::Align::Start);
        oauth_signin_btn.set_margin_top(12);
        oauth_signin_btn.set_visible(false);
        oauth_signin_btn.add_css_class("suggested-action");
        {
            let s = sender.input_sender().clone();
            oauth_signin_btn.connect_clicked(move |_| s.emit(AccountsInput::OAuthSignIn));
        }
        incoming_group.add(&oauth_signin_btn);
        let oauth_status = gtk::Label::new(None);
        oauth_status.set_visible(false);
        oauth_status.set_halign(gtk::Align::Start);
        oauth_status.set_xalign(0.0);
        oauth_status.set_wrap(true);
        oauth_status.set_margin_top(6);
        incoming_group.add(&oauth_status);

        // Shown for Google/Microsoft when no built-in or own OAuth client
        // is available: GNOME Online Accounts is then the way in.
        let goa_hint = gtk::Box::new(gtk::Orientation::Vertical, 12);
        goa_hint.set_margin_top(8);
        goa_hint.set_visible(false);
        let hint = gtk::Label::new(Some(&i18n(
            "Google and Microsoft sign in through GNOME Online Accounts. Sign in there, then reopen this window and switch the account on under GNOME Online Accounts.",
        )));
        hint.set_wrap(true);
        hint.set_xalign(0.0);
        hint.add_css_class("dim-label");
        goa_hint.append(&hint);
        let goa_btn = gtk::Button::with_label(&i18n("Open Online Accounts…"));
        goa_btn.set_halign(gtk::Align::Start);
        goa_btn.add_css_class("suggested-action");
        {
            let s = sender.input_sender().clone();
            goa_btn.connect_clicked(move |_| s.emit(AccountsInput::OpenOnlineAccounts));
        }
        goa_hint.append(&goa_btn);
        incoming_group.add(&goa_hint);
        page.add(&incoming_group);

        let smtp_group = group(&i18n("Outgoing (SMTP)"), None);
        let smtp_row = entry(&i18n("Server"));
        let smtp_port_row = entry(&i18n("Port"));
        smtp_port_row.set_input_purpose(gtk::InputPurpose::Digits);
        let smtp_separate_row = switch(
            &i18n("Use other credentials"),
            &i18n("Off: the incoming credentials are used."),
        );
        let smtp_user_row = entry(&i18n("User name"));
        let smtp_pass_row = password(&i18n("Password"));
        for row in [
            smtp_row.upcast_ref::<gtk::Widget>(),
            smtp_port_row.upcast_ref(),
            smtp_separate_row.upcast_ref(),
            smtp_user_row.upcast_ref(),
            smtp_pass_row.upcast_ref(),
        ] {
            smtp_group.add(row);
        }
        page.add(&smtp_group);

        // The rare fields (DESIGN.md, "Accounts").
        let advanced_group = group("", None);
        let advanced_row = adw::ExpanderRow::new();
        advanced_row.set_title(&i18n("Advanced"));
        advanced_row.set_subtitle(&i18n(
            "API token, custom OAuth app, certificate for another name",
        ));
        // JMAP with a bearer token in place of the username and password
        // (#356).
        let jmap_token_row = switch(
            &i18n("Sign in with an API token"),
            &i18n("For servers, like Fastmail, that take a token instead of a password."),
        );
        jmap_token_row.set_visible(false);
        {
            let s = sender.input_sender().clone();
            jmap_token_row.connect_active_notify(move |_| s.emit(AccountsInput::ProtocolChanged));
        }
        let oauth_client_id_row = entry(&i18n("OAuth client ID"));
        let oauth_secret_row = password(&i18n("OAuth client secret (optional)"));
        let oauth_auth_url_row = entry(&i18n("Authorization URL"));
        let oauth_token_url_row = entry(&i18n("Token URL"));
        let oauth_scope_row = entry(&i18n("Scopes (space-separated)"));
        // Microsoft's own sign-in, made with another app registration
        // (#329): one an organization has approved, say. Empty rows keep
        // Tegami Mail's.
        let ms_client_row = entry(&i18n("Client ID"));
        let ms_tenant_row = entry(&i18n("Tenant (common, organizations, consumers or an ID)"));
        let ms_scopes_row = entry(&i18n("Scopes (space-separated)"));
        let ms_redirect_row = entry(&i18n("Redirect URI"));
        {
            // A redirect of its own signs in in Tegami Mail's window, not
            // the browser.
            let signin = oauth_signin_btn.clone();
            ms_redirect_row.connect_changed(move |row| {
                signin.set_label(&if row.text().trim().is_empty() {
                    i18n("Sign in with browser")
                } else {
                    i18n("Sign in")
                });
            });
        }
        let tls_mismatch_row = switch(
            &i18n("Accept a certificate for another name"),
            &i18n(
                "For shared hosting. Turn on only when the connection test says the names differ.",
            ),
        );
        for row in [
            jmap_token_row.upcast_ref::<gtk::Widget>(),
            oauth_client_id_row.upcast_ref(),
            oauth_secret_row.upcast_ref(),
            oauth_auth_url_row.upcast_ref(),
            oauth_token_url_row.upcast_ref(),
            oauth_scope_row.upcast_ref(),
            ms_client_row.upcast_ref(),
            ms_tenant_row.upcast_ref(),
            ms_scopes_row.upcast_ref(),
            ms_redirect_row.upcast_ref(),
            tls_mismatch_row.upcast_ref(),
        ] {
            advanced_row.add_row(row);
        }
        advanced_group.add(&advanced_row);
        page.add(&advanced_group);

        // A Microsoft 365 account's organization directory (#410): its
        // suggestions, asking for their own access when turned on.
        let graph_group = group(&i18n("Microsoft 365"), None);
        graph_group.set_visible(false);
        let graph_directory_row = switch(
            &i18n("Search the organization directory"),
            &i18n("Suggests people from your organization as you type."),
        );
        {
            let s = sender.input_sender().clone();
            graph_directory_row.connect_active_notify(move |row| {
                s.emit(AccountsInput::SetGraphDirectory(row.is_active()))
            });
        }
        let directory_only_row = switch(
            &i18n("Suggest only from this account"),
            &i18n("Suggestions come from the directory and this account's mail only."),
        );
        graph_group.add(&graph_directory_row);
        graph_group.add(&directory_only_row);
        // How the last request for access went.
        let graph_status = gtk::Label::new(None);
        graph_status.set_visible(false);
        graph_status.set_halign(gtk::Align::Start);
        graph_status.set_xalign(0.0);
        graph_status.set_wrap(true);
        graph_status.set_selectable(true);
        graph_status.set_margin_top(8);
        graph_group.add(&graph_status);
        page.add(&graph_group);

        // The test, and its result as an icon and a sentence (DESIGN.md,
        // "Accounts"). The text can be copied out (#246): an error is
        // often something to paste into a bug report.
        let test_group = group("", None);
        let test_btn = action_row(&i18n("Test connection"), None, "add-row");
        {
            let s = sender.input_sender().clone();
            test_btn.connect_activated(move |_| s.emit(AccountsInput::TestConnection));
        }
        let test_result = adw::ActionRow::new();
        test_result.set_visible(false);
        test_result.set_title_selectable(true);
        test_result.set_use_markup(false);
        let test_icon = gtk::Image::new();
        test_result.add_prefix(&test_icon);
        test_group.add(&test_btn);
        test_group.add(&test_result);
        page.add(&test_group);

        ServersPage {
            page: sub_page(&i18n("Servers and connection"), &page),
            provider_group,
            incoming_group,
            advanced_row,
            provider_mark,
            provider_row,
            protocol_row,
            host_row,
            port_row,
            smtp_row,
            smtp_port_row,
            user_row,
            pass_row,
            jmap_token_row,
            oauth_client_id_row,
            oauth_secret_row,
            oauth_auth_url_row,
            oauth_token_url_row,
            oauth_scope_row,
            ms_client_row,
            ms_tenant_row,
            ms_scopes_row,
            ms_redirect_row,
            oauth_signin_btn,
            oauth_status,
            goa_hint,
            smtp_separate_row,
            smtp_user_row,
            smtp_pass_row,
            tls_mismatch_row,
            test_group,
            test_btn,
            test_result,
            test_icon,
            graph_group,
            graph_directory_row,
            directory_only_row,
            graph_status,
        }
    }

    fn sync() -> SyncPage {
        let page = adw::PreferencesPage::new();
        // Per-account push override (#91): some servers mishandle IDLE,
        // and one bad account shouldn't cost the good ones their instant
        // delivery.
        let push_group = group(&i18n("New messages"), None);
        let push_row = combo(
            &i18n("Instant new mail (push)"),
            &i18n("Turn off for servers that block push connections."),
        );
        push_group.add(&push_row);
        page.add(&push_group);

        // Auto-empty (#140): mail past the chosen age in Junk / Trash is
        // deleted for good at each sync.
        let cleanup = group(&i18n("Automatic cleanup"), None);
        let empty_junk_row = combo(&i18n("Empty Junk"), "");
        let empty_trash_row = combo(&i18n("Empty Trash"), "");
        cleanup.add(&empty_junk_row);
        cleanup.add(&empty_trash_row);
        page.add(&cleanup);

        // Mail downloaded ahead of being opened (#536), and the copies
        // kept in On This Computer (#552; kept by the owner, 2026-11-08).
        let offline_group = group(
            &i18n("On this computer"),
            Some(&i18n("The general setting is in Application.")),
        );
        let offline_mail_row = combo(
            &i18n("Keep downloaded mail"),
            &i18n("Leaves out Trash, Junk and Drafts."),
        );
        let offline_attachments_row = switch(
            &i18n("Include attachments"),
            &i18n("Can consume a lot of hard drive space."),
        );
        let copy_local_row = switch(
            &i18n("Copy mail to On This Computer"),
            &i18n("Downloads copies of every message from every folder of this account to this computer (except Trash, Junk and Drafts) and stores them in On This Computer."),
        );
        offline_group.add(&offline_mail_row);
        offline_group.add(&offline_attachments_row);
        offline_group.add(&copy_local_row);
        page.add(&offline_group);

        SyncPage {
            page: sub_page(&i18n("Synchronisation"), &page),
            push_group,
            push_row,
            empty_junk_row,
            empty_trash_row,
            offline_group,
            offline_mail_row,
            offline_attachments_row,
            copy_local_row,
        }
    }

    fn folders(sender: &ComponentSender<AccountsWindow>) -> FoldersPage {
        let page = adw::PreferencesPage::new();
        // Manual special-folder mapping (#82): for servers whose Sent,
        // Trash… aren't detected, pin each role to one of the account's
        // real folders. The Inbox is offered only in "Save a copy of sent
        // mail in" (#199): relabelling it as a role costs the account its
        // inbox (#136).
        let special = group(
            &i18n("Special folders"),
            Some(&i18n("Follows the server’s own markings unless you pick a folder. Nothing already on the server moves.")),
        );
        let folder_sent_row = combo(&i18n("Sent"), "");
        let folder_drafts_row = combo(&i18n("Drafts"), "");
        let folder_archive_row = combo(&i18n("Archive"), "");
        let folder_trash_row = combo(&i18n("Trash"), "");
        let folder_junk_row = combo(&i18n("Junk"), "");
        for row in [
            &folder_sent_row,
            &folder_drafts_row,
            &folder_archive_row,
            &folder_trash_row,
            &folder_junk_row,
        ] {
            special.add(row);
        }
        page.add(&special);

        let advanced_group = group("", None);
        let advanced = adw::ExpanderRow::new();
        advanced.set_title(&i18n("Advanced"));
        advanced.set_subtitle(&i18n("Templates, snoozed, reply copies, server copy"));
        // Nothing on a server marks a Templates folder (#360), nor the one
        // snoozed mail waits in (#527): found by name, or pointed at here.
        let folder_templates_row = combo(&i18n("Templates"), "");
        let folder_snoozed_row = combo(&i18n("Snoozed"), "");
        // Replies beside what they answer (#468), for IMAP and JMAP.
        let reply_copies_row = combo(&i18n("Save a copy of replies in"), &i18n(REPLY_COPIES_HINT));
        let folder_sent_copy_row =
            combo(&i18n("Save a copy of sent mail in"), &i18n(SENT_COPY_HINT));
        // Gmail files a copy of anything sent through its SMTP, so Tegami
        // Mail's append makes a second one. Off by default: a server that
        // does not, paired with a Tegami Mail that has stopped appending,
        // keeps no sent mail at all.
        let server_saves_row = switch(
            &i18n("Server saves its own copy of sent mail"),
            &i18n("For Gmail and others that file sent mail themselves."),
        );
        {
            let s = sender.input_sender().clone();
            server_saves_row.connect_active_notify(move |row| {
                s.emit(AccountsInput::SetServerSavesSent(row.is_active()))
            });
        }
        for row in [
            folder_templates_row.upcast_ref::<gtk::Widget>(),
            folder_snoozed_row.upcast_ref(),
            reply_copies_row.upcast_ref(),
            folder_sent_copy_row.upcast_ref(),
            server_saves_row.upcast_ref(),
        ] {
            advanced.add_row(row);
        }
        advanced_group.add(&advanced);
        page.add(&advanced_group);

        // Its own folder order, or the application's.
        let order = group(&i18n("Order"), None);
        let folder_sort_row = combo(
            &i18n("Folder order"),
            &i18n("Otherwise, drag them in the sidebar."),
        );
        order.add(&folder_sort_row);
        page.add(&order);

        // Hidden folders (#239): what "Hide folder" in the sidebar took out
        // of sight, each with the way back.
        let hidden = group(
            &i18n("Hidden folders"),
            Some(&i18n(
                "Hidden folders are not synced. Hide one from its right-click menu in the sidebar.",
            )),
        );
        let hidden_list = gtk::ListBox::new();
        hidden_list.add_css_class("boxed-list");
        hidden_list.set_selection_mode(gtk::SelectionMode::None);
        hidden.add(&hidden_list);
        let hidden_empty = gtk::Label::new(Some(&i18n("No hidden folders.")));
        hidden_empty.add_css_class("dim-label");
        hidden_empty.set_xalign(0.0);
        hidden.add(&hidden_empty);
        page.add(&hidden);

        FoldersPage {
            page: sub_page(&i18n("Folders"), &page),
            folder_sort_row,
            folder_sent_row,
            folder_sent_copy_row,
            reply_copies_row,
            server_saves_row,
            folder_drafts_row,
            folder_templates_row,
            folder_trash_row,
            folder_junk_row,
            folder_archive_row,
            folder_snoozed_row,
            hidden_list,
            hidden_empty,
        }
    }

    /// Send-as aliases (#34; screen 32): extra From identities the
    /// composer offers. Each sends through this account's SMTP, or — for a
    /// forwarded mailbox whose provider would rewrite the sender — through
    /// its own. The list ends with the row that adds one
    /// (`rebuild_alias_list`).
    fn aliases(sender: &ComponentSender<AccountsWindow>) -> (adw::NavigationPage, gtk::ListBox) {
        let page = adw::PreferencesPage::new();
        let g = group(
            "",
            Some(&i18n("Extra addresses this account can send as. They appear in the composer’s From field.")),
        );
        let list = gtk::ListBox::new();
        list.add_css_class("boxed-list");
        list.set_selection_mode(gtk::SelectionMode::None);
        {
            let s = sender.input_sender().clone();
            list.connect_row_activated(move |_, row| {
                if row.has_css_class("add-row") {
                    s.emit(AccountsInput::AliasAdd);
                } else {
                    s.emit(AccountsInput::AliasEdit(row.index() as usize));
                }
            });
        }
        g.add(&list);
        page.add(&g);
        (sub_page(&i18n("Send-as aliases"), &page), list)
    }

    fn signatures(sender: &ComponentSender<AccountsWindow>) -> SignaturesPage {
        let page = adw::PreferencesPage::new();
        let g = group("", None);
        // Which signature the editor holds, and its name: shown once there
        // are two.
        let choice_row = combo(&i18n("Signature"), "");
        choice_row.set_visible(false);
        {
            let s = sender.input_sender().clone();
            choice_row.connect_selected_notify(move |_| s.emit(AccountsInput::SigShow));
        }
        let name_row = entry(&i18n("Name"));
        name_row.set_visible(false);
        {
            let s = sender.input_sender().clone();
            name_row.connect_changed(move |_| s.emit(AccountsInput::SigRename));
        }
        // The rich editor, made when an editor first opens: a WebView.
        let holder = gtk::Box::new(gtk::Orientation::Vertical, 0);
        holder.set_margin_top(12);
        holder.set_margin_bottom(12);
        holder.set_margin_start(12);
        holder.set_margin_end(12);
        let editor_row = adw::PreferencesRow::new();
        editor_row.set_activatable(false);
        editor_row.set_selectable(false);
        editor_row.set_focusable(false);
        editor_row.set_child(Some(&holder));
        let default_row = switch(&i18n("Start new messages with this signature"), "");
        {
            let s = sender.input_sender().clone();
            default_row.connect_active_notify(move |_| s.emit(AccountsInput::SigDefault));
        }
        let delete_row = action_row(
            &i18n("Delete signature"),
            Some("user-trash-symbolic"),
            "error",
        );
        delete_row.set_visible(false);
        emit(&delete_row, sender, || AccountsInput::SigDelete);
        for row in [
            choice_row.upcast_ref::<gtk::Widget>(),
            name_row.upcast_ref(),
            editor_row.upcast_ref(),
            default_row.upcast_ref(),
            delete_row.upcast_ref(),
        ] {
            g.add(row);
        }
        page.add(&g);

        let more = group(
            "",
            Some(&i18n(
                "With several signatures, the composer offers a button to choose.",
            )),
        );
        let add = add_row(&i18n("Add signature"));
        emit(&add, sender, || AccountsInput::SigAdd);
        // A designed signature usually exists as HTML already (#120).
        let import = add_row(&i18n("Import file…"));
        import.set_subtitle(&i18n("HTML or text"));
        emit(&import, sender, || AccountsInput::SignatureImport);
        more.add(&add);
        more.add(&import);
        page.add(&more);

        SignaturesPage {
            page: sub_page(&i18n("Signatures"), &page),
            choice_row,
            name_row,
            default_row,
            delete_row,
            holder,
        }
    }

    /// OpenPGP (#133; screen 34): which of the user's keys this account
    /// signs and decrypts with, and the way to the keys themselves.
    fn openpgp(
        sender: &ComponentSender<AccountsWindow>,
    ) -> (
        adw::NavigationPage,
        adw::ComboRow,
        adw::SwitchRow,
        adw::SwitchRow,
    ) {
        let page = adw::PreferencesPage::new();
        let g = group("", None);
        let key_row = combo(
            &i18n("Key"),
            &i18n("Automatic: the key whose address matches."),
        );
        // For an account whose mail is always signed (#267): the composer's
        // toggle starts on, and still turns it off for one message.
        let sign_row = switch(&i18n("Sign messages by default"), "");
        // Protected headers (#493): on, an encrypted message says "..."
        // where its subject would be.
        let hide_row = switch(
            &i18n("Hide the subject of encrypted mail"),
            &i18n("Only the recipients see the subject."),
        );
        g.add(&key_row);
        g.add(&sign_row);
        g.add(&hide_row);
        page.add(&g);

        let keys = group("", None);
        let manage = adw::ActionRow::new();
        manage.set_title(&i18n("Manage keys"));
        manage.set_subtitle(&i18n("Import, generate, trust."));
        manage.set_activatable(true);
        manage.add_suffix(&gtk::Image::from_icon_name("go-next-symbolic"));
        {
            let s = sender.output_sender().clone();
            manage.connect_activated(move |_| {
                let _ = s.send(super::super::AccountsOutput::OpenSettingsPage("openpgp"));
            });
        }
        keys.add(&manage);
        page.add(&keys);
        (
            sub_page(&i18n("OpenPGP encryption"), &page),
            key_row,
            sign_row,
            hide_row,
        )
    }
}
