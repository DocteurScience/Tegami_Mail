//! The preferences dialog (docs/DESIGN.md, "Preferences"; screens 22 to
//! 25): five pages in a view switcher — Accounts, Messages, Notifications,
//! Privacy, Application — with libadwaita's own search, and sub-pages for
//! the account panels, the services and Composing's advanced rows.
//!
//! The rows kept from the former twenty pages (`pages::kept`, and the
//! Composing page's) are set in the dialog's groups; what the design
//! removed is gone (DESIGN.md, "Removed or moved").

use super::*;

/// The sub-pages the accounts component shows, by its page id.
const ACCOUNT_PANELS: [&str; 4] = ["tags", "filters", "smart", "senders"];

/// The sub-pages hosted by components of their own.
const SERVICES: [&str; 5] = ["openpgp", "cloud", "translation", "contacts", "backup"];

/// Where a page id of the former sidebar now lives: one of [`PAGES`], and
/// the sub-page to open over it, if any. Ids of removed pages land on the
/// page that took their kept settings.
pub(super) fn place_of(id: &str) -> (&'static str, Option<&'static str>) {
    if let Some(panel) = ACCOUNT_PANELS.iter().find(|p| **p == id) {
        return ("accounts", Some(panel));
    }
    if let Some(service) = SERVICES.iter().find(|p| **p == id) {
        return ("accounts", Some(service));
    }
    match id {
        "accounts" => ("accounts", None),
        "messages" | "composing" | "reading" | "conversations" | "list" => ("messages", None),
        "notifications" | "general" => ("notifications", None),
        "privacy" => ("privacy", None),
        _ => ("application", None),
    }
}

/// The title a sub-page goes by, in its header.
fn subpage_title(id: &str) -> String {
    i18n(match id {
        "tags" => i18n_noop("Tags"),
        "filters" => i18n_noop("Filters"),
        "smart" => i18n_noop("Smart folders"),
        "senders" => i18n_noop("Senders"),
        "openpgp" => i18n_noop("OpenPGP keys"),
        "cloud" => i18n_noop("Cloud storage"),
        "translation" => i18n_noop("Translation"),
        "contacts" => i18n_noop("Address books"),
        "backup" => i18n_noop("Backup and restore"),
        _ => i18n_noop("Advanced"),
    })
}

/// A sub-page: a header bar with the back button over `content`.
fn subpage(title: &str, content: &impl IsA<gtk::Widget>) -> (adw::NavigationPage, adw::HeaderBar) {
    let header = adw::HeaderBar::new();
    let view = adw::ToolbarView::new();
    view.add_top_bar(&header);
    view.set_content(Some(content));
    (adw::NavigationPage::new(&view, title), header)
}

/// Move `row` out of whatever group holds it into `group`.
fn take(group: &adw::PreferencesGroup, row: &impl IsA<gtk::Widget>) {
    let row = row.as_ref();
    if let Some(old) = row
        .ancestor(adw::PreferencesGroup::static_type())
        .and_downcast::<adw::PreferencesGroup>()
    {
        old.remove(row);
    } else if let Some(parent) = row.parent() {
        if let Some(list) = parent.downcast_ref::<gtk::ListBox>() {
            list.remove(row);
        }
    }
    group.add(row);
}

/// A titled group.
fn group(title: &str) -> adw::PreferencesGroup {
    let g = adw::PreferencesGroup::new();
    g.set_title(title);
    g
}

/// A row that opens a sub-page: a title, a subtitle, a chevron.
fn link_row(title: &str, subtitle: Option<&str>) -> adw::ActionRow {
    let row = adw::ActionRow::new();
    row.set_title(title);
    if let Some(s) = subtitle {
        row.set_subtitle(s);
    }
    row.set_activatable(true);
    let next = gtk::Image::from_icon_name("go-next-symbolic");
    next.set_accessible_role(gtk::AccessibleRole::Presentation);
    row.add_suffix(&next);
    row
}

/// The icon of a row on the Accounts page, on a disc (screen 26).
fn row_icon(row: &adw::ActionRow, icon: &str) {
    let image = gtk::Image::from_icon_name(icon);
    image.add_css_class("pref-row-icon");
    image.set_valign(gtk::Align::Center);
    image.set_accessible_role(gtk::AccessibleRole::Presentation);
    row.add_prefix(&image);
}

/// Move the accounts panel's list of accounts onto the Accounts page
/// (screen 26): the panel builds and fills it, the dialog shows it. A
/// fresh panel brings a fresh list, which takes the old one's place.
pub(super) fn mount_accounts_list(slot: &adw::Bin, panel: &gtk::Widget) {
    let Some(list) = named_descendant(panel, crate::ui::accounts::ACCOUNTS_LIST_NAME) else {
        return;
    };
    // The panel keeps it in its stack until the dialog takes it.
    if let Some(stack) = list.parent().and_downcast::<gtk::Stack>() {
        stack.remove(&list);
    }
    slot.set_child(Some(&list));
}

/// The widget named `name` under `root`, depth first.
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

/// The pages the dialog shows and the sub-pages it pushes, kept to move
/// between them from `ShowPageById`.
pub(super) struct DialogPages {
    pages: Vec<(&'static str, adw::PreferencesPage)>,
    /// The sub-page that hosts the accounts component, re-titled for the
    /// panel it shows.
    accounts_sub: adw::NavigationPage,
    services: Vec<(&'static str, adw::NavigationPage)>,
    /// The Accounts page's rows, by the page they open, for their counts.
    links: Vec<(&'static str, adw::ActionRow)>,
    /// Each service sub-page's header, hidden while an editor of the
    /// service shows its own.
    headers: Vec<(&'static str, adw::HeaderBar)>,
    advanced: adw::NavigationPage,
}

impl Preferences {
    /// Build the five pages from the rows the page builders made, add them
    /// to the dialog, and wire the rows that open sub-pages.
    pub(super) fn assemble_dialog(
        &mut self,
        widgets: &PreferencesWidgets,
        init: &PrefInit,
        slots: super::init_layout::HostedSlots,
        root: &adw::PreferencesDialog,
        sender: &ComponentSender<Self>,
    ) {
        // Titles only in the switcher, as screens 22 to 25 draw it: the
        // icons, which libadwaita requires, are hidden by styles.css, as
        // with them the longer French names were cut short.
        let page = |id: &str, title: &str, icon: &str| {
            let p = adw::PreferencesPage::new();
            p.set_name(Some(id));
            p.set_title(title);
            p.set_icon_name(Some(icon));
            p
        };

        // ---- Accounts (screen 26): the accounts themselves, listed by
        // the accounts panel, then the rows to what organizes mail and to
        // the services.
        let accounts = page("accounts", &i18n("Accounts"), "avatar-default-symbolic");
        let mail = group(&i18n("Mail accounts"));
        mail.set_description(Some(&i18n(
            "Drag an account to change its order in the sidebar.",
        )));
        let list_slot = adw::Bin::new();
        mount_accounts_list(&list_slot, &init.accounts_panel);
        mail.add(&list_slot);
        self.accounts_list_slot = Some(list_slot);
        let organize = group(&i18n("Organize mail"));
        let services = group(&i18n("Services"));
        let mut rows: Vec<(&'static str, adw::ActionRow)> = Vec::new();
        for (id, g, icon, subtitle) in [
            ("tags", &organize, "tag-outline-symbolic", None),
            ("filters", &organize, "filter-symbolic", None),
            ("smart", &organize, "folder-saved-search-symbolic", None),
            (
                "senders",
                &organize,
                "system-users-symbolic",
                Some(i18n("Allowed and blocked")),
            ),
            ("openpgp", &services, "dialog-password-symbolic", None),
            ("cloud", &services, "cloud-symbolic", None),
            ("translation", &services, "translate-symbolic", None),
            (
                "contacts",
                &services,
                "x-office-address-book-symbolic",
                None,
            ),
            ("backup", &services, "folder-download-symbolic", None),
        ] {
            let row = link_row(&subpage_title(id), subtitle.as_deref());
            row_icon(&row, icon);
            g.add(&row);
            rows.push((id, row));
        }
        let links = rows.clone();
        accounts.add(&mail);
        accounts.add(&organize);
        accounts.add(&services);
        // What each service is set to, read again each time the page
        // shows: the sub-pages change it (screen 26).
        {
            let rows = rows.clone();
            accounts.connect_map(move |_| service_subtitles(&rows));
        }

        // ---- Messages (screen 22).
        let messages = page("messages", &i18n("Messages"), "mail-unread-symbolic");
        let reading = group(&i18n("Reading"));
        let k = &widgets.kept;
        for row in [
            k.read_mark_row.upcast_ref::<gtk::Widget>(),
            k.thread_newest_first_row.upcast_ref(),
            k.confirm_thread_delete_row.upcast_ref(),
            k.swipe_enabled_row.upcast_ref(),
        ] {
            reading.add(row);
        }
        messages.add(&reading);
        let composing = group(&i18n("Composing"));
        let c = &widgets.composing;
        c.default_from_row.set_subtitle(&i18n(
            "Replies are sent from the address that received the message.",
        ));
        take(&composing, &c.default_from_row);
        c.compose_format_row
            .set_title(&i18n("Format of new messages"));
        take(&composing, &c.compose_format_row);
        c.undo_send_row
            .set_subtitle(&i18n("How long a message can still be recalled."));
        take(&composing, &c.undo_send_row);
        c.spellcheck_row.set_title(&i18n("Check spelling"));
        take(&composing, &c.spellcheck_row);
        // The rest of Composing (the owner's decision, 2026-11-08: kept,
        // under Advanced): what is left on the former Composing page.
        let advanced_row = link_row(&i18n("Advanced"), None);
        composing.add(&advanced_row);
        messages.add(&composing);
        let (advanced, _) = subpage(&i18n("Advanced"), &widgets.composing.page);

        // ---- Notifications (screen 23).
        let notifications = page(
            "notifications",
            &i18n("Notifications"),
            "preferences-system-notifications-symbolic",
        );
        let new_mail = group(&i18n("New messages"));
        for row in [
            k.notifications_row.upcast_ref::<gtk::Widget>(),
            k.notification_content_row.upcast_ref(),
            k.notify_receipts_row.upcast_ref(),
            k.fetch_row.upcast_ref(),
            k.push_row.upcast_ref(),
        ] {
            new_mail.add(row);
        }
        notifications.add(&new_mail);
        let banners = group(&i18n("Sound and banners"));
        let system = adw::ActionRow::new();
        system.set_title(&i18n("Open the system notification settings"));
        system.set_subtitle(&i18n("Sound, banners and Do Not Disturb are set there."));
        system.set_activatable(true);
        system.add_suffix(&gtk::Image::from_icon_name("adw-external-link-symbolic"));
        system.connect_activated(|_| open_system_notifications());
        banners.add(&system);
        notifications.add(&banners);

        // ---- Privacy (screen 24 comes in step 2): the former page.
        let privacy = &widgets.privacy.page;
        privacy.set_name(Some("privacy"));
        privacy.set_title(&i18n("Privacy"));
        privacy.set_icon_name(Some("security-high-symbolic"));

        // ---- Application (screen 25).
        let application = page(
            "application",
            &i18n("Application"),
            "applications-system-symbolic",
        );
        let look = group(&i18n("Appearance"));
        look.add(&k.app_theme_row);
        application.add(&look);
        let background = group(&i18n("Background"));
        background.add(&k.background_row);
        background.add(&k.autostart_row);
        application.add(&background);
        let keys = group(&i18n("Keyboard and storage"));
        for row in [
            k.single_key_row.upcast_ref::<gtk::Widget>(),
            k.gmail_keys_row.upcast_ref(),
            k.keep_downloads_row.upcast_ref(),
            k.files_large_row.upcast_ref(),
        ] {
            keys.add(row);
        }
        application.add(&keys);

        for p in [&accounts, &messages, &notifications, privacy, &application] {
            root.add(p);
        }

        // ---- Sub-pages.
        let accounts_slot = adw::Bin::new();
        accounts_slot.set_child(Some(&init.accounts_panel));
        let (accounts_sub, accounts_header) = subpage(&i18n("Mail accounts"), &accounts_slot);
        self.host_header = Some(accounts_header);
        self.accounts_slot = Some(accounts_slot);
        let mut subs = Vec::new();
        let mut headers = Vec::new();
        for (id, slot) in [
            ("openpgp", slots.pgp.upcast::<gtk::Widget>()),
            ("cloud", slots.cloud.upcast()),
            ("translation", slots.translation.upcast()),
            ("contacts", slots.contacts.upcast()),
            ("backup", widgets.backup.page.clone().upcast()),
        ] {
            let (page, header) = subpage(&subpage_title(id), &slot);
            subs.push((id, page));
            headers.push((id, header));
        }
        for (id, row) in rows {
            let s = sender.clone();
            row.connect_activated(move |_| s.input(PrefInput::ShowPageById(id.to_string())));
        }
        let s = sender.clone();
        advanced_row
            .connect_activated(move |_| s.input(PrefInput::ShowPageById("advanced".into())));

        self.dialog_pages = Some(Rc::new(DialogPages {
            pages: vec![
                ("accounts", accounts),
                ("messages", messages),
                ("notifications", notifications),
                ("privacy", privacy.clone()),
                ("application", application),
            ],
            accounts_sub,
            services: subs,
            links,
            headers,
            advanced,
        }));
        self.dialog = Some(root.clone());
    }

    /// Show a page, or a sub-page over its page, by id (the former
    /// sidebar's ids are understood too).
    pub(super) fn show_by_id(&self, id: &str, sender: &ComponentSender<Self>) {
        let (Some(dialog), Some(pages)) = (&self.dialog, &self.dialog_pages) else {
            return;
        };
        // A sub-page already up goes first.
        while dialog.pop_subpage() {}
        if id == "advanced" {
            dialog.set_visible_page_name("messages");
            dialog.push_subpage(&pages.advanced);
            return;
        }
        let (page, sub) = place_of(id);
        if let Some((_, p)) = pages.pages.iter().find(|(n, _)| *n == page) {
            dialog.set_visible_page(p);
        }
        let Some(sub) = sub else {
            let _ = sender.output(PrefOutput::PageShown(page.to_string()));
            return;
        };
        if ACCOUNT_PANELS.contains(&sub) {
            let _ = self
                .accounts_sender
                .send(crate::ui::accounts::AccountsInput::ShowPage(
                    sub.to_string(),
                ));
            pages.accounts_sub.set_title(&subpage_title(sub));
            dialog.push_subpage(&pages.accounts_sub);
        } else if let Some((_, p)) = pages.services.iter().find(|(n, _)| *n == sub) {
            dialog.push_subpage(p);
        }
        let _ = sender.output(PrefOutput::PageShown(sub.to_string()));
    }
}

impl Preferences {
    /// Push a service's sub-page over what is shown ("Manage keys" on an
    /// account's OpenPGP page): the account page stays under it.
    pub(super) fn push_page(&self, id: &str) {
        let (Some(dialog), Some(pages)) = (&self.dialog, &self.dialog_pages) else {
            return;
        };
        if let Some((_, p)) = pages.services.iter().find(|(n, _)| *n == id) {
            if p.parent().is_none() {
                dialog.push_subpage(p);
            }
        }
    }

    /// The counts under Tags, Filters and Smart folders (screen 26).
    pub(super) fn set_counts(&self, tags: usize, filters: usize, smart: usize) {
        let Some(pages) = &self.dialog_pages else {
            return;
        };
        let line = |id: &str| match id {
            "tags" => Some(ni18n_f(
                "{n} tag",
                "{n} tags",
                tags as u32,
                &[("n", &tags.to_string())],
            )),
            "filters" => Some(ni18n_f(
                "{n} filter",
                "{n} filters",
                filters as u32,
                &[("n", &filters.to_string())],
            )),
            "smart" => Some(ni18n_f(
                "{n} smart folder",
                "{n} smart folders",
                smart as u32,
                &[("n", &smart.to_string())],
            )),
            _ => None,
        };
        for (id, row) in &pages.links {
            if let Some(text) = line(id) {
                row.set_subtitle(&text);
            }
        }
    }

    /// Show or hide a service sub-page's header (its editor brings its own).
    pub(super) fn show_sub_header(&self, id: &str, visible: bool) {
        if let Some(pages) = &self.dialog_pages {
            if let Some((_, h)) = pages.headers.iter().find(|(n, _)| *n == id) {
                h.set_visible(visible);
            }
        }
    }
}

/// The Services rows' subtitles: the cloud accounts by name, and the
/// translation service when one is set.
fn service_subtitles(rows: &[(&'static str, adw::ActionRow)]) {
    for (id, row) in rows {
        let text = match *id {
            "cloud" => crate::services::cloud::load_accounts()
                .iter()
                .map(|a| {
                    if a.name.trim().is_empty() {
                        a.where_shown()
                    } else {
                        a.name.trim().to_string()
                    }
                })
                .collect::<Vec<_>>()
                .join(", "),
            "translation" => {
                let s = crate::services::translate::load();
                if s.enabled() {
                    s.service.name()
                } else {
                    String::new()
                }
            }
            _ => continue,
        };
        row.set_subtitle(&text);
    }
}

/// GNOME Settings on its Notifications panel, where sound, banners and Do
/// Not Disturb are set (screen 23).
fn open_system_notifications() {
    crate::desktop::settings_panel::open_settings_panel("notifications");
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every id of the former sidebar still lands somewhere, the account
    /// panels and services on a sub-page of Accounts.
    #[test]
    fn former_page_ids_find_their_place() {
        assert_eq!(place_of("tags"), ("accounts", Some("tags")));
        assert_eq!(place_of("backup"), ("accounts", Some("backup")));
        assert_eq!(place_of("composing"), ("messages", None));
        assert_eq!(place_of("general"), ("notifications", None));
        assert_eq!(place_of("sidebar"), ("application", None));
        for p in [
            "accounts",
            "messages",
            "notifications",
            "privacy",
            "application",
        ] {
            assert_eq!(place_of(p).0, p);
        }
    }
}
