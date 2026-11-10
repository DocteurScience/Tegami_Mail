//! The accounts page (screen 50): the GNOME Online Accounts mail accounts
//! found, each with an "Add" button that becomes a check and "Added", the
//! accounts added by hand beside them, and the other ways in.

use super::*;

use crate::services::goa::GoaMailAccount;

/// One row of the accounts list: an Online Accounts account still to add
/// (`goa` set), or one added either way.
pub(super) struct Listed {
    email: String,
    subtitle: String,
    goa: Option<GoaMailAccount>,
}

impl Listed {
    pub(super) fn found(g: GoaMailAccount) -> Self {
        Listed {
            email: g.email.clone(),
            subtitle: g.provider.clone(),
            goa: Some(g),
        }
    }
}

/// The accounts page's widgets the wizard updates.
pub(super) struct AccountsPage {
    pub(super) page: adw::NavigationPage,
    list: gtk::ListBox,
    /// "found these accounts", or "none were found" while the list is empty.
    text: gtk::Label,
}

pub(super) fn build_accounts_page(sender: &ComponentSender<Welcome>) -> AccountsPage {
    let col = column();
    col.append(&heading(&i18n("Add your accounts")));
    let text = sentence("");
    col.append(&text);
    let list = gtk::ListBox::new();
    list.add_css_class("boxed-list");
    list.set_selection_mode(gtk::SelectionMode::None);
    list.set_margin_top(12);
    col.append(&list);

    let other = adw::PreferencesGroup::new();
    other.set_title(&i18n("Another account"));
    other.set_margin_top(12);
    let manual = adw::ActionRow::new();
    manual.set_title(&i18n("Email account…"));
    manual.set_subtitle(&i18n("With an address and a password (IMAP)"));
    manual.add_prefix(&row_icon("list-add-symbolic"));
    manual.add_suffix(&gtk::Image::from_icon_name("go-next-symbolic"));
    manual.set_activatable(true);
    {
        let s = sender.clone();
        manual.connect_activated(move |_| s.input(WelcomeInput::OpenForm));
    }
    let online = adw::ActionRow::new();
    online.set_title(&i18n("Open Online Accounts"));
    online.set_subtitle(&i18n("For Google or Microsoft 365"));
    online.add_prefix(&row_icon("cloud-symbolic"));
    online.add_suffix(&gtk::Image::from_icon_name("adw-external-link-symbolic"));
    online.set_activatable(true);
    {
        let s = sender.clone();
        online.connect_activated(move |_| s.input(WelcomeInput::OpenOnlineAccounts));
    }
    other.add(&manual);
    other.add(&online);
    col.append(&other);

    let next = pill(&i18n("Continue"));
    {
        let s = sender.clone();
        next.connect_clicked(move |_| s.input(WelcomeInput::Continue));
    }
    col.append(&next);
    AccountsPage {
        page: nav_page("accounts", &i18n("Add your accounts"), &col),
        list,
        text,
    }
}

/// An icon before a row's title, on the neutral disc of the preferences'
/// rows (screen 26).
fn row_icon(name: &str) -> gtk::Image {
    let image = gtk::Image::from_icon_name(name);
    image.add_css_class("pref-row-icon");
    image.set_valign(gtk::Align::Center);
    image.set_accessible_role(gtk::AccessibleRole::Presentation);
    image
}

/// "Added": a check and the word, so the state never rests on colour.
fn added_mark() -> gtk::Box {
    let mark = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    mark.add_css_class("success");
    mark.set_valign(gtk::Align::Center);
    let check = gtk::Image::from_icon_name("object-select-symbolic");
    check.set_accessible_role(gtk::AccessibleRole::Presentation);
    mark.append(&check);
    mark.append(&gtk::Label::new(Some(&i18n("Added"))));
    mark
}

impl Welcome {
    /// Draw the list again from `self.listed`, and the sentence above it.
    pub(super) fn rebuild_accounts(&self, page: &AccountsPage, sender: &ComponentSender<Self>) {
        while let Some(row) = page.list.first_child() {
            page.list.remove(&row);
        }
        for (i, item) in self.listed.iter().enumerate() {
            let row = adw::ActionRow::new();
            row.set_title(&item.email);
            row.set_subtitle(&item.subtitle);
            row.set_use_markup(false);
            row.add_prefix(&row_icon("avatar-default-symbolic"));
            if item.goa.is_some() {
                let btn = gtk::Button::with_label(&i18n("Add"));
                btn.add_css_class("suggested-action");
                btn.set_valign(gtk::Align::Center);
                let s = sender.clone();
                btn.connect_clicked(move |_| s.input(WelcomeInput::ImportGoa(i)));
                row.add_suffix(&btn);
            } else {
                row.add_suffix(&added_mark());
            }
            page.list.append(&row);
        }
        page.list.set_visible(!self.listed.is_empty());
        page.text.set_text(&if self.listed.is_empty() {
            i18n("No mail accounts were found in Online Accounts.")
        } else {
            i18n("Tegami Mail found these accounts in GNOME Online Accounts. One click adds each of them.")
        });
    }

    /// An account was added by hand: it joins the list, already added.
    pub(super) fn listed_added(
        &mut self,
        email: &str,
        subtitle: &str,
        widgets: &WelcomeWidgets,
        sender: &ComponentSender<Self>,
    ) {
        self.added.push(email.to_string());
        self.listed.push(Listed {
            email: email.to_string(),
            subtitle: subtitle.to_string(),
            goa: None,
        });
        self.rebuild_accounts(&widgets.accounts, sender);
    }

    pub(super) fn on_import_goa(
        &mut self,
        index: usize,
        widgets: &WelcomeWidgets,
        sender: ComponentSender<Self>,
    ) {
        let Some(g) = self.listed.get_mut(index).and_then(|l| l.goa.take()) else {
            return;
        };
        let (password, oauth) = if g.password_based {
            (
                crate::services::goa::mail_passwords(&g.id)
                    .0
                    .unwrap_or_default(),
                false,
            )
        } else {
            (String::new(), true)
        };
        let account = g.to_config(password, oauth);
        self.added.push(account.email.clone());
        let _ = sender.output(WelcomeOutput::ImportGoa(Box::new(account)));
        self.rebuild_accounts(&widgets.accounts, &sender);
    }

    /// Back from GNOME Settings: the accounts added there join the list.
    pub(super) fn on_rescan_goa(
        &mut self,
        widgets: &WelcomeWidgets,
        sender: ComponentSender<Self>,
    ) {
        let known: Vec<String> = self
            .listed
            .iter()
            .map(|l| l.email.to_ascii_lowercase())
            .collect();
        let fresh: Vec<Listed> = crate::services::goa::list_mail_accounts()
            .into_iter()
            .filter(|g| g.can_connect() && !known.contains(&g.email.to_ascii_lowercase()))
            .map(Listed::found)
            .collect();
        if !fresh.is_empty() {
            self.listed.extend(fresh);
            self.rebuild_accounts(&widgets.accounts, &sender);
        }
    }
}

/// The accounts screen 50 shows, for its capture (TEGAMI_SHOWCASE_WIZARD):
/// one added, one to add.
pub(super) fn showcase_listed() -> Vec<Listed> {
    let account = |email: &str, provider: &str, kind: &str| GoaMailAccount {
        id: String::new(),
        email: email.into(),
        name: String::new(),
        provider: provider.into(),
        imap_host: String::new(),
        imap_port: 0,
        imap_user: String::new(),
        smtp_host: String::new(),
        smtp_port: 0,
        smtp_user: String::new(),
        smtp_separate: false,
        password_based: false,
        oauth2: true,
        provider_type: kind.into(),
        security: Default::default(),
    };
    let mut added = Listed::found(account("jason@gmail.com", "Google", "google"));
    added.goa = None;
    vec![
        added,
        Listed::found(account("jason@entreprise.fr", "Microsoft 365", "ms_graph")),
    ]
}
