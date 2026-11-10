//! The email account page (screen 51): an address and a password, with
//! "Advanced" folded (provider, display name, servers, and what the
//! provider needs), testing and adding the account, and the browser
//! sign-in for OAuth providers.

use std::cell::Cell;
use std::rc::Rc;

use super::*;

/// The providers the form offers: the shared table less Google, which comes
/// in through Online Accounts.
pub(super) fn wizard_providers() -> impl Iterator<Item = &'static Provider> {
    PROVIDERS.iter().filter(|p| p.wizard_listed())
}

/// The form's provider at a drop-down index.
fn wizard_provider(sel: u32) -> Option<&'static Provider> {
    wizard_providers().nth(sel as usize)
}

/// The drop-down index of the plain IMAP/POP3 entry, the default.
fn manual_index() -> u32 {
    wizard_providers()
        .position(|p| p.wizard_is_manual())
        .unwrap_or(0) as u32
}

/// The drop-down index the address `email` points to: its provider's, or
/// the plain entry when the domain is not a known one.
fn index_for_address(email: &str) -> u32 {
    crate::ui::accounts::provider_for_address(email)
        // By label: an address comparison held only while PROVIDERS was a
        // static, and it once was a const, whose uses need not share a copy.
        .and_then(|found| wizard_providers().position(|p| p.wizard_label() == found.wizard_label()))
        .map_or(manual_index(), |i| i as u32)
}

/// The form's widgets the wizard reads and updates.
pub(super) struct FormPage {
    pub(super) page: adw::NavigationPage,
    pub(super) provider_row: adw::ComboRow,
    /// The provider was picked by hand: the address no longer chooses it.
    picked: Rc<Cell<bool>>,
    /// Up while the code, not a person, moves the provider row.
    setting: Rc<Cell<bool>>,
    hint_lbl: gtk::Label,
    name_row: adw::EntryRow,
    email_row: adw::EntryRow,
    pass_row: adw::PasswordEntryRow,
    host_row: adw::EntryRow,
    port_row: adw::EntryRow,
    smtp_row: adw::EntryRow,
    smtp_port_row: adw::EntryRow,
    oauth_client_id_row: adw::EntryRow,
    oauth_secret_row: adw::PasswordEntryRow,
    oauth_auth_url_row: adw::EntryRow,
    oauth_token_url_row: adw::EntryRow,
    oauth_scope_row: adw::EntryRow,
    /// What went wrong, as a sentence under the fields.
    error_lbl: gtk::Label,
    /// The spinner and its sentence while a test or sign-in runs.
    status_box: gtk::Box,
    status_lbl: gtk::Label,
    add_btn: gtk::Button,
}

/// Screen 51.
pub(super) fn build_form_page(sender: &ComponentSender<Welcome>) -> FormPage {
    let col = column();
    col.append(&heading(&i18n("Email account")));
    col.append(&sentence(&i18n(
        "Tegami Mail finds the servers of most providers from your address.",
    )));
    let form = gtk::ListBox::new();
    form.add_css_class("boxed-list");
    form.set_selection_mode(gtk::SelectionMode::None);
    form.set_margin_top(12);
    let email_row = adw::EntryRow::new();
    email_row.set_title(&i18n("Email address"));
    email_row.set_input_purpose(gtk::InputPurpose::Email);
    let pass_row = adw::PasswordEntryRow::new();
    pass_row.set_title(&i18n("Password"));
    let advanced = adw::ExpanderRow::new();
    advanced.set_title(&i18n("Advanced"));
    advanced.set_subtitle(&i18n("Display name, servers, API token"));
    let provider_row = adw::ComboRow::new();
    provider_row.set_title(&i18n("Provider"));
    let labels: Vec<&str> = wizard_providers().map(|p| p.wizard_label()).collect();
    provider_row.set_model(Some(&gtk::StringList::new(&labels)));
    provider_row.set_factory(Some(&crate::ui::accounts::provider_factory()));
    provider_row.set_selected(manual_index());
    let name_row = adw::EntryRow::new();
    name_row.set_title(&i18n("Display name"));
    let host_row = adw::EntryRow::new();
    host_row.set_title(&i18n("IMAP server"));
    let port_row = adw::EntryRow::new();
    port_row.set_title(&i18n("IMAP port"));
    port_row.set_text("993");
    let smtp_row = adw::EntryRow::new();
    smtp_row.set_title(&i18n("SMTP server"));
    let smtp_port_row = adw::EntryRow::new();
    smtp_port_row.set_title(&i18n("SMTP port"));
    smtp_port_row.set_text("587");
    // Custom OAuth's client, the same fields as the account editor's.
    let oauth_client_id_row = adw::EntryRow::new();
    oauth_client_id_row.set_title(&i18n("OAuth client ID"));
    let oauth_secret_row = adw::PasswordEntryRow::new();
    oauth_secret_row.set_title(&i18n("OAuth Client Secret (optional)"));
    let oauth_auth_url_row = adw::EntryRow::new();
    oauth_auth_url_row.set_title(&i18n("Authorization URL"));
    let oauth_token_url_row = adw::EntryRow::new();
    oauth_token_url_row.set_title(&i18n("Token URL"));
    let oauth_scope_row = adw::EntryRow::new();
    oauth_scope_row.set_title(&i18n("Scopes (space-separated)"));
    advanced.add_row(&provider_row);
    advanced.add_row(&name_row);
    for row in [
        oauth_client_id_row.upcast_ref::<gtk::Widget>(),
        oauth_secret_row.upcast_ref(),
        oauth_auth_url_row.upcast_ref(),
        oauth_token_url_row.upcast_ref(),
        oauth_scope_row.upcast_ref(),
    ] {
        row.set_visible(false);
        advanced.add_row(row);
    }
    advanced.add_row(&host_row);
    advanced.add_row(&port_row);
    advanced.add_row(&smtp_row);
    advanced.add_row(&smtp_port_row);
    form.append(&email_row);
    form.append(&pass_row);
    form.append(&advanced);
    col.append(&form);

    // What the provider asks (an app password, Proton's Bridge…), then
    // what went wrong: sentences under the fields.
    let hint_lbl = gtk::Label::new(None);
    hint_lbl.add_css_class("secondary-text");
    hint_lbl.set_xalign(0.0);
    hint_lbl.set_wrap(true);
    hint_lbl.set_visible(false);
    col.append(&hint_lbl);
    let error_lbl = gtk::Label::new(None);
    error_lbl.add_css_class("error");
    error_lbl.set_xalign(0.0);
    error_lbl.set_wrap(true);
    error_lbl.set_visible(false);
    error_lbl.set_accessible_role(gtk::AccessibleRole::Alert);
    col.append(&error_lbl);
    let status_box = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    status_box.set_halign(gtk::Align::Center);
    status_box.set_margin_top(6);
    status_box.append(&adw::Spinner::new());
    let status_lbl = gtk::Label::new(None);
    status_lbl.add_css_class("secondary-text");
    status_lbl.set_wrap(true);
    status_box.append(&status_lbl);
    status_box.set_visible(false);
    col.append(&status_box);

    let add_btn = pill(&i18n("Test and add"));
    {
        let s = sender.clone();
        add_btn.connect_clicked(move |_| s.input(WelcomeInput::TestAndAdd));
    }
    col.append(&add_btn);

    let picked = Rc::new(Cell::new(false));
    let setting = Rc::new(Cell::new(false));
    // The address picks the provider until one is picked by hand.
    {
        let (row, picked, setting) = (provider_row.clone(), picked.clone(), setting.clone());
        email_row.connect_changed(move |e| {
            if !picked.get() {
                setting.set(true);
                row.set_selected(index_for_address(&e.text()));
                setting.set(false);
            }
        });
    }
    {
        let (s, picked, setting) = (sender.clone(), picked.clone(), setting.clone());
        provider_row.connect_selected_notify(move |_| {
            if !setting.get() {
                picked.set(true);
            }
            s.input(WelcomeInput::ProviderChanged);
        });
    }
    FormPage {
        page: nav_page("email", &i18n("Email account"), &col),
        provider_row,
        picked,
        setting,
        hint_lbl,
        name_row,
        email_row,
        pass_row,
        host_row,
        port_row,
        smtp_row,
        smtp_port_row,
        oauth_client_id_row,
        oauth_secret_row,
        oauth_auth_url_row,
        oauth_token_url_row,
        oauth_scope_row,
        error_lbl,
        status_box,
        status_lbl,
        add_btn,
    }
}

impl FormPage {
    /// A test or sign-in runs: the sentence beside a spinner, the button
    /// disabled, no error.
    fn busy(&self, text: &str) {
        self.error_lbl.set_visible(false);
        self.status_lbl.set_text(text);
        self.status_box.set_visible(true);
        self.add_btn.set_sensitive(false);
    }

    /// Nothing runs any more; `error`, when given, says why it failed.
    fn idle(&self, error: Option<&str>) {
        self.status_box.set_visible(false);
        self.add_btn.set_sensitive(true);
        self.error_lbl.set_visible(error.is_some());
        self.error_lbl.set_text(error.unwrap_or(""));
    }

    /// Empty again for the next account, the provider back to the address's.
    fn clear(&self) {
        for row in [&self.email_row, &self.name_row] {
            row.set_text("");
        }
        self.pass_row.set_text("");
        self.picked.set(false);
        self.setting.set(true);
        self.provider_row.set_selected(manual_index());
        self.setting.set(false);
    }
}

impl Welcome {
    /// Custom OAuth or Microsoft: sign in through the browser, then add the
    /// account with the refresh token that comes back (the connection test
    /// is skipped, as in Settings: it reads the token from the keyring,
    /// where it lands only once the account is saved).
    fn sign_in_and_add(
        &mut self,
        widgets: &WelcomeWidgets,
        sender: &ComponentSender<Self>,
        email: String,
        host: String,
        smtp: String,
    ) {
        let form = &widgets.form;
        let text = |row: &adw::EntryRow| row.text().trim().to_string();
        let builtin =
            wizard_provider(form.provider_row.selected()).and_then(|p| p.wizard_builtin_oauth());
        let protocol = if builtin.is_some() {
            Protocol::Graph
        } else {
            Protocol::Imap
        };
        let settings = builtin.unwrap_or_else(|| crate::config::OAuthSettings {
            auth_url: text(&form.oauth_auth_url_row),
            token_url: text(&form.oauth_token_url_row),
            client_id: text(&form.oauth_client_id_row),
            client_secret: form.oauth_secret_row.text().to_string(),
            scopes: text(&form.oauth_scope_row),
            redirect_uri: String::new(),
        });
        // Microsoft needs nothing typed: the sign-in says who it is.
        if protocol != Protocol::Graph
            && (email.is_empty()
                || host.is_empty()
                || settings.client_id.is_empty()
                || settings.auth_url.is_empty()
                || settings.token_url.is_empty())
        {
            form.idle(Some(&i18n(
                "Enter your email address, the client ID and both OAuth URLs first.",
            )));
            return;
        }
        let graph = protocol == Protocol::Graph;
        let account = AccountConfig {
            name: text(&form.name_row),
            email: email.clone(),
            protocol,
            imap_host: if graph { String::new() } else { host },
            imap_port: form.port_row.text().trim().parse().unwrap_or(993),
            smtp_host: if graph { String::new() } else { smtp },
            smtp_port: form.smtp_port_row.text().trim().parse().unwrap_or(587),
            username: email,
            oauth: true,
            oauth_settings: Some(settings.clone()),
            ..blank_account()
        };
        self.test_seq += 1;
        let seq = self.test_seq;
        form.busy(&i18n("Opening browser… complete sign-in there."));
        sender.oneshot_command(async move {
            let result = tokio::task::spawn_blocking(move || {
                let refresh = crate::services::oauth::run_flow(&settings)?.refresh_token;
                if settings.token_url.contains("microsoftonline") {
                    let (name, address, refresh) =
                        crate::services::oauth::microsoft_whoami(&settings, &refresh, None)?;
                    return Ok((refresh, Some((name, address))));
                }
                Ok((refresh, None))
            })
            .await
            .unwrap_or_else(|_| Err("sign-in task failed".into()));
            WelcomeCmd::SignedIn {
                account: Box::new(account),
                result,
                seq,
            }
        });
    }

    /// Another provider: what it needs shows, and its servers fill in.
    pub(super) fn on_provider_changed(&mut self, widgets: &WelcomeWidgets) {
        let form = &widgets.form;
        let Some(p) = wizard_provider(form.provider_row.selected()) else {
            return;
        };
        let oauth = p.wizard_is_oauth();
        let custom = p.wizard_is_custom_oauth();
        // Microsoft brings its own client and servers (#329).
        let builtin = p.wizard_builtin_oauth().is_some();
        // A newer choice supersedes a sign-in still waiting in the
        // browser; its late result is ignored.
        self.test_seq += 1;
        form.idle(None);
        form.pass_row.set_visible(!oauth);
        for row in [
            form.oauth_client_id_row.upcast_ref::<gtk::Widget>(),
            form.oauth_secret_row.upcast_ref(),
            form.oauth_auth_url_row.upcast_ref(),
            form.oauth_token_url_row.upcast_ref(),
            form.oauth_scope_row.upcast_ref(),
        ] {
            row.set_visible(custom);
        }
        form.add_btn.set_label(&if oauth {
            i18n("Sign in and add")
        } else {
            i18n("Test and add")
        });
        let (ih, ip, sh, sp) = p.wizard_servers();
        let jmap = p.wizard_protocol() == Protocol::Jmap;
        if !ih.is_empty() {
            form.host_row.set_text(ih);
            form.port_row.set_text(&ip.to_string());
            form.smtp_row.set_text(sh);
            form.smtp_port_row.set_text(&sp.to_string());
        } else {
            form.host_row.set_text("");
            form.smtp_row.set_text("");
            form.port_row.set_text(if jmap { "443" } else { "993" });
            form.smtp_port_row.set_text("587");
        }
        // A JMAP server is one address, and it sends the mail itself.
        form.host_row.set_title(&if jmap {
            i18n("Server (host name or URL)")
        } else {
            i18n("IMAP server")
        });
        form.port_row.set_title(&if jmap {
            i18n("Port")
        } else {
            i18n("IMAP port")
        });
        form.host_row.set_visible(!builtin);
        form.port_row.set_visible(!builtin);
        form.smtp_row.set_visible(!jmap && !builtin);
        form.smtp_port_row.set_visible(!jmap && !builtin);
        // Fastmail over JMAP takes an API token, not the password (#356).
        form.pass_row.set_title(&if p.wizard_token() {
            i18n("API token")
        } else {
            i18n("Password")
        });
        let hint = p.wizard_hint();
        // The plain entry's "enter your server details" would only nag.
        let shown = !hint.is_empty() && !p.wizard_is_manual();
        form.hint_lbl.set_visible(shown);
        form.hint_lbl.set_text(&i18n(hint));
    }

    pub(super) fn on_test_and_add(
        &mut self,
        widgets: &WelcomeWidgets,
        sender: ComponentSender<Self>,
    ) {
        let form = &widgets.form;
        let email = form.email_row.text().trim().to_string();
        let password = form.pass_row.text().to_string();
        let provider = wizard_provider(form.provider_row.selected());
        let oauth = provider.is_some_and(|p| p.wizard_is_oauth());
        let protocol = provider.map(|p| p.wizard_protocol()).unwrap_or_default();
        // Missing servers from the address's domain, the common convention;
        // Advanced is there to correct them. A JMAP server's address is the
        // user's to give.
        let domain = email.split('@').nth(1).unwrap_or("").to_string();
        let mut host = form.host_row.text().trim().to_string();
        if host.is_empty() && !domain.is_empty() && protocol != Protocol::Jmap {
            host = format!("imap.{domain}");
            form.host_row.set_text(&host);
        }
        let mut smtp = form.smtp_row.text().trim().to_string();
        if smtp.is_empty() && !domain.is_empty() && protocol != Protocol::Jmap {
            smtp = format!("smtp.{domain}");
            form.smtp_row.set_text(&smtp);
        }
        if oauth {
            self.sign_in_and_add(widgets, &sender, email, host, smtp);
            return;
        }
        if email.is_empty() || password.is_empty() || host.is_empty() {
            form.idle(Some(&i18n("Enter your email address and password first.")));
            return;
        }
        let account = AccountConfig {
            name: form.name_row.text().trim().to_string(),
            email: email.clone(),
            protocol,
            imap_host: host,
            imap_port: form
                .port_row
                .text()
                .trim()
                .parse()
                .unwrap_or(if protocol == Protocol::Jmap { 443 } else { 993 }),
            smtp_host: smtp,
            smtp_port: form.smtp_port_row.text().trim().parse().unwrap_or(587),
            username: email,
            password,
            jmap_token: provider.is_some_and(|p| p.wizard_token()),
            jmap_generic: provider.is_some_and(|p| p.wizard_jmap_generic()),
            ..blank_account()
        };
        self.test_seq += 1;
        let seq = self.test_seq;
        form.busy(&i18n("Checking the connection…"));
        sender.oneshot_command(async move {
            let test = {
                let account = account.clone();
                tokio::task::spawn_blocking(move || worker::test_connection_blocking(account))
                    .await
                    .unwrap_or_else(|_| ConnTest {
                        incoming: Err("test could not run".into()),
                        smtp: Err("test could not run".into()),
                    })
            };
            WelcomeCmd::Tested {
                account: Box::new(account),
                result: test,
                seq,
            }
        });
    }

    /// The account is in: it joins the accounts page, which comes back.
    fn added_by_hand(
        &mut self,
        account: Box<AccountConfig>,
        widgets: &WelcomeWidgets,
        sender: &ComponentSender<Self>,
    ) {
        let subtitle = if account.protocol == Protocol::Graph {
            String::from("Microsoft 365")
        } else {
            account.imap_host.clone()
        };
        widgets.form.idle(None);
        widgets.form.clear();
        self.listed_added(&account.email, &subtitle, widgets, sender);
        let _ = sender.output(WelcomeOutput::AddAccount(account));
        widgets.nav.pop_to_tag("accounts");
    }

    pub(super) fn on_tested(
        &mut self,
        account: Box<AccountConfig>,
        result: ConnTest,
        seq: u32,
        widgets: &WelcomeWidgets,
        sender: ComponentSender<Self>,
    ) {
        if seq != self.test_seq {
            // A newer test was started; this stale verdict no longer matters.
            return;
        }
        match (&result.incoming, &result.smtp) {
            (Ok(_), Ok(_)) => self.added_by_hand(account, widgets, &sender),
            (Err(e), _) => widgets.form.idle(Some(&i18n_f(
                "{email} — mail server: {e}",
                &[("email", &account.email), ("e", e)],
            ))),
            (_, Err(e)) => widgets.form.idle(Some(&i18n_f(
                "{email} — sending (SMTP): {e}",
                &[("email", &account.email), ("e", e)],
            ))),
        }
    }

    pub(super) fn on_signed_in(
        &mut self,
        mut account: Box<AccountConfig>,
        result: Result<(String, Option<(String, String)>), String>,
        seq: u32,
        widgets: &WelcomeWidgets,
        sender: ComponentSender<Self>,
    ) {
        if seq != self.test_seq {
            return;
        }
        match result {
            Ok((refresh, who)) => {
                // Microsoft's account is named for the mailbox the sign-in
                // reached (#329).
                if let Some((name, address)) = who {
                    if !address.is_empty() {
                        account.email = address.clone();
                        account.username = address;
                    }
                    if account.name.is_empty() {
                        account.name = name;
                    }
                }
                account.oauth_refresh = refresh;
                self.added_by_hand(account, widgets, &sender);
            }
            Err(e) => widgets
                .form
                .idle(Some(&i18n_f("Sign-in failed: {e}", &[("e", &e)]))),
        }
    }
}

/// TEGAMI_SHOWCASE_WIZARD=testing: the form of screen 51, mid-test.
pub(super) fn showcase_testing(form: &FormPage) {
    form.email_row.set_text("jason@example.org");
    form.pass_row.set_text("0123456789");
    form.busy(&i18n("Checking the connection…"));
    form.add_btn.set_sensitive(true);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_wizard_lists_every_provider_but_google() {
        assert_eq!(wizard_providers().count(), PROVIDERS.len() - 1);
        assert!(wizard_providers().all(|p| p.wizard_label() != "Google (Gmail)"));
        assert!(wizard_provider(manual_index()).is_some_and(|p| p.wizard_is_manual()));
        assert!(wizard_provider(PROVIDERS.len() as u32).is_none());
    }

    #[test]
    fn an_address_picks_the_form_provider() {
        let label = |e: &str| wizard_provider(index_for_address(e)).map(|p| p.wizard_label());
        assert_eq!(label("ada@icloud.com"), Some("iCloud"));
        assert_eq!(label("ada@outlook.fr"), Some("Microsoft 365 / Outlook"));
        assert_eq!(label("ada@example.org"), Some("IMAP/POP3 Account"));
        assert_eq!(label(""), Some("IMAP/POP3 Account"));
    }
}
