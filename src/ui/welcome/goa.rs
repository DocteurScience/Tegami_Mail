//! The accounts page: the GNOME Online Accounts mail accounts to add, and
//! the other ways in.

use super::*;

/// The accounts page's widgets the wizard updates.
pub(super) struct AccountsPage {
    pub(super) page: adw::NavigationPage,
    pub(super) list: gtk::ListBox,
    pub(super) empty: gtk::Label,
}

pub(super) fn build_accounts_page(
    model: &Welcome,
    sender: &ComponentSender<Welcome>,
) -> AccountsPage {
    let col = column();
    col.append(&heading(&i18n("Import your online accounts")));
    col.append(&sentence(&i18n(
        "Mail accounts signed in under GNOME Settings → Online Accounts can be added here in one click. Google accounts are only supported through GNOME Online Accounts.",
    )));
    let list = gtk::ListBox::new();
    list.add_css_class("boxed-list");
    list.set_selection_mode(gtk::SelectionMode::None);
    list.set_margin_top(12);
    list.set_visible(!model.goa.is_empty());
    col.append(&list);
    let empty = sentence(&i18n("No mail accounts were found in Online Accounts."));
    empty.set_visible(model.goa.is_empty());
    col.append(&empty);
    let links = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    links.set_halign(gtk::Align::Center);
    let open_goa = gtk::Button::with_label(&i18n("Open Online Accounts…"));
    let manual = gtk::Button::with_label(&i18n("Email account…"));
    for b in [&open_goa, &manual] {
        b.add_css_class("pill");
    }
    {
        let s = sender.clone();
        open_goa.connect_clicked(move |_| s.input(WelcomeInput::OpenOnlineAccounts));
        let s = sender.clone();
        manual.connect_clicked(move |_| s.input(WelcomeInput::OpenForm));
    }
    links.append(&manual);
    links.append(&open_goa);
    col.append(&links);
    let next = pill(&i18n("Continue"));
    {
        let s = sender.clone();
        next.connect_clicked(move |_| s.input(WelcomeInput::Continue));
    }
    col.append(&next);
    AccountsPage {
        page: nav_page("accounts", &i18n("Add your accounts"), &col),
        list,
        empty,
    }
}

pub(super) fn rebuild_goa_rows(
    list: &gtk::ListBox,
    goa: &[crate::services::goa::GoaMailAccount],
    sender: &ComponentSender<Welcome>,
) {
    while let Some(row) = list.first_child() {
        list.remove(&row);
    }
    for (i, g) in goa.iter().enumerate() {
        let row = adw::ActionRow::new();
        row.set_title(&g.email);
        row.set_subtitle(&g.provider);
        let btn = gtk::Button::with_label(&i18n("Add"));
        btn.add_css_class("suggested-action");
        btn.set_valign(gtk::Align::Center);
        let s = sender.clone();
        btn.connect_clicked(move |_| s.input(WelcomeInput::ImportGoa(i)));
        row.add_suffix(&btn);
        list.append(&row);
    }
}

impl Welcome {
    pub(super) fn on_import_goa(
        &mut self,
        index: usize,
        widgets: &WelcomeWidgets,
        sender: ComponentSender<Self>,
    ) {
        if let Some(g) = self.goa.get(index).cloned() {
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
            self.goa.remove(index);
            let _ = sender.output(WelcomeOutput::ImportGoa(Box::new(account)));
            rebuild_goa_rows(&widgets.accounts.list, &self.goa, &sender);
            widgets.accounts.list.set_visible(!self.goa.is_empty());
        }
    }

    pub(super) fn on_rescan_goa(
        &mut self,
        widgets: &WelcomeWidgets,
        sender: ComponentSender<Self>,
    ) {
        self.goa = crate::services::goa::list_mail_accounts();
        self.goa
            .retain(|g| g.can_connect() && !self.added.contains(&g.email));
        rebuild_goa_rows(&widgets.accounts.list, &self.goa, &sender);
        widgets.accounts.list.set_visible(!self.goa.is_empty());
        widgets
            .accounts
            .empty
            .set_visible(self.goa.is_empty() && self.added.is_empty());
    }
}
