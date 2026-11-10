//! The manual account form: its providers, testing and adding an account,
//! and the browser sign-in for OAuth providers.

use super::*;

/// The providers the manual form offers: the shared table less Google and
/// Microsoft, which come in through the Online Accounts page before it.
pub(super) fn wizard_providers() -> impl Iterator<Item = &'static Provider> {
    PROVIDERS.iter().filter(|p| p.wizard_listed())
}

/// The manual form's provider at a drop-down index.
fn wizard_provider(sel: u32) -> Option<&'static Provider> {
    wizard_providers().nth(sel as usize)
}

/// The form's widgets the wizard reads and updates.
pub(super) struct FormPage {
    pub(super) page: adw::NavigationPage,
    pub(super) provider_row: adw::ComboRow,
    pub(super) hint_lbl: gtk::Label,
    pub(super) name_row: adw::EntryRow,
    pub(super) email_row: adw::EntryRow,
    pub(super) pass_row: adw::PasswordEntryRow,
    pub(super) server_exp: adw::ExpanderRow,
    pub(super) host_row: adw::EntryRow,
    pub(super) port_row: adw::EntryRow,
    pub(super) smtp_row: adw::EntryRow,
    pub(super) smtp_port_row: adw::EntryRow,
    pub(super) oauth_client_id_row: adw::EntryRow,
    pub(super) oauth_secret_row: adw::PasswordEntryRow,
    pub(super) oauth_auth_url_row: adw::EntryRow,
    pub(super) oauth_token_url_row: adw::EntryRow,
    pub(super) oauth_scope_row: adw::EntryRow,
    pub(super) status_lbl: gtk::Label,
    pub(super) test_spinner: gtk::Spinner,
    pub(super) add_btn: gtk::Button,
}

/// The email account page: an account by hand.
pub(super) fn build_form_page(sender: &ComponentSender<Welcome>) -> FormPage {
    let col = column();
    col.append(&heading(&i18n("Add your email account")));
    let form = gtk::ListBox::new();
    form.add_css_class("boxed-list");
    form.set_selection_mode(gtk::SelectionMode::None);
    let provider_row = adw::ComboRow::new();
    provider_row.set_title(&i18n("Provider"));
    let labels: Vec<&str> = wizard_providers().map(|p| p.wizard_label()).collect();
    provider_row.set_model(Some(&gtk::StringList::new(&labels)));
    provider_row.set_factory(Some(&crate::ui::accounts::provider_factory()));
    let manual = wizard_providers()
        .position(|p| p.wizard_is_manual())
        .unwrap_or(0);
    provider_row.set_selected(manual as u32);
    {
        let s = sender.clone();
        provider_row.connect_selected_notify(move |_| s.input(WelcomeInput::ProviderChanged));
    }
    let name_row = adw::EntryRow::new();
    name_row.set_title(&i18n("Your name"));
    let email_row = adw::EntryRow::new();
    email_row.set_title(&i18n("Email address"));
    let pass_row = adw::PasswordEntryRow::new();
    pass_row.set_title(&i18n("Password"));
    let server_exp = adw::ExpanderRow::new();
    server_exp.set_title(&i18n("Server details"));
    server_exp.set_subtitle(&i18n("Filled in for known providers"));
    server_exp.set_expanded(true);
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
    server_exp.add_row(&host_row);
    server_exp.add_row(&port_row);
    server_exp.add_row(&smtp_row);
    server_exp.add_row(&smtp_port_row);
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
    form.append(&provider_row);
    form.append(&name_row);
    form.append(&email_row);
    form.append(&pass_row);
    for row in [
        oauth_client_id_row.upcast_ref::<gtk::Widget>(),
        oauth_secret_row.upcast_ref(),
        oauth_auth_url_row.upcast_ref(),
        oauth_token_url_row.upcast_ref(),
        oauth_scope_row.upcast_ref(),
    ] {
        row.set_visible(false);
        form.append(row);
    }
    form.append(&server_exp);
    col.append(&form);

    let hint_lbl = gtk::Label::new(None);
    hint_lbl.add_css_class("secondary-text");
    hint_lbl.set_halign(gtk::Align::Start);
    hint_lbl.set_wrap(true);
    hint_lbl.set_visible(false);
    col.append(&hint_lbl);

    let add_btn = pill(&i18n("Test & add"));
    {
        let s = sender.clone();
        add_btn.connect_clicked(move |_| s.input(WelcomeInput::TestAndAdd));
    }
    col.append(&add_btn);
    let status_box = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    status_box.set_halign(gtk::Align::Center);
    let test_spinner = gtk::Spinner::new();
    test_spinner.set_visible(false);
    let status_lbl = gtk::Label::new(None);
    status_lbl.add_css_class("secondary-text");
    status_lbl.set_wrap(true);
    status_lbl.set_justify(gtk::Justification::Center);
    status_box.append(&test_spinner);
    status_box.append(&status_lbl);
    col.append(&status_box);
    FormPage {
        page: nav_page("email", &i18n("Email account"), &col),
        provider_row,
        hint_lbl,
        name_row,
        email_row,
        pass_row,
        server_exp,
        host_row,
        port_row,
        smtp_row,
        smtp_port_row,
        oauth_client_id_row,
        oauth_secret_row,
        oauth_auth_url_row,
        oauth_token_url_row,
        oauth_scope_row,
        status_lbl,
        test_spinner,
        add_btn,
    }
}

impl Welcome {
    /// Custom OAuth: sign in through the browser with the form's client,
    /// then add the account with the refresh token that comes back (the
    /// connection test is skipped, as in Settings: it reads the token from
    /// the keyring, where it lands only once the account is saved).
    fn sign_in_and_add(
        &mut self,
        widgets: &WelcomeWidgets,
        sender: &ComponentSender<Self>,
        email: String,
        host: String,
        smtp: String,
    ) {
        let text = |row: &adw::EntryRow| row.text().trim().to_string();
        let builtin = wizard_provider(widgets.form.provider_row.selected())
            .and_then(|p| p.wizard_builtin_oauth());
        let protocol = if builtin.is_some() {
            Protocol::Graph
        } else {
            Protocol::Imap
        };
        let settings = builtin.unwrap_or_else(|| crate::config::OAuthSettings {
            auth_url: text(&widgets.form.oauth_auth_url_row),
            token_url: text(&widgets.form.oauth_token_url_row),
            client_id: text(&widgets.form.oauth_client_id_row),
            client_secret: widgets.form.oauth_secret_row.text().to_string(),
            scopes: text(&widgets.form.oauth_scope_row),
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
            widgets.form.status_lbl.set_css_classes(&["error"]);
            widgets.form.status_lbl.set_text(&i18n(
                "Enter your email address, the client ID and both OAuth URLs first.",
            ));
            return;
        }
        let account = AccountConfig {
            name: widgets.form.name_row.text().trim().to_string(),
            email: email.clone(),
            protocol,
            imap_host: if protocol == Protocol::Graph {
                String::new()
            } else {
                host
            },
            imap_port: widgets.form.port_row.text().trim().parse().unwrap_or(993),
            smtp_host: if protocol == Protocol::Graph {
                String::new()
            } else {
                smtp
            },
            smtp_port: widgets
                .form
                .smtp_port_row
                .text()
                .trim()
                .parse()
                .unwrap_or(587),
            username: email,
            oauth: true,
            oauth_settings: Some(settings.clone()),
            ..blank_account()
        };
        self.test_seq += 1;
        let seq = self.test_seq;
        widgets.form.add_btn.set_sensitive(false);
        widgets.form.test_spinner.set_visible(true);
        widgets.form.test_spinner.start();
        widgets.form.status_lbl.set_css_classes(&["secondary-text"]);
        widgets
            .form
            .status_lbl
            .set_text(&i18n("Opening browser… complete sign-in there."));
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
}

impl Welcome {
    pub(super) fn on_provider_changed(&mut self, widgets: &WelcomeWidgets) {
        let sel = widgets.form.provider_row.selected();
        let oauth = wizard_provider(sel).is_some_and(|p| p.wizard_is_oauth());
        // Microsoft brings its own client and servers (#329): only
        // the address is asked for, and even that is filled in by
        // the sign-in.
        let custom = wizard_provider(sel).is_some_and(|p| p.wizard_is_custom_oauth());
        let builtin = wizard_provider(sel).is_some_and(|p| p.wizard_builtin_oauth().is_some());
        // A newer choice supersedes a sign-in still waiting in the
        // browser; its late result is ignored.
        self.test_seq += 1;
        widgets.form.test_spinner.stop();
        widgets.form.test_spinner.set_visible(false);
        widgets.form.add_btn.set_sensitive(true);
        widgets.form.status_lbl.set_text("");
        widgets.form.pass_row.set_visible(!oauth);
        for row in [
            widgets.form.oauth_client_id_row.upcast_ref::<gtk::Widget>(),
            widgets.form.oauth_secret_row.upcast_ref(),
            widgets.form.oauth_auth_url_row.upcast_ref(),
            widgets.form.oauth_token_url_row.upcast_ref(),
            widgets.form.oauth_scope_row.upcast_ref(),
        ] {
            row.set_visible(custom);
        }
        widgets.form.server_exp.set_visible(!builtin);
        widgets.form.add_btn.set_label(&if oauth {
            i18n("Sign in & add")
        } else {
            i18n("Test & add")
        });
        if let Some(p) = wizard_provider(sel) {
            let (ih, ip, sh, sp) = p.wizard_servers();
            let jmap = p.wizard_protocol() == Protocol::Jmap;
            if !ih.is_empty() {
                widgets.form.host_row.set_text(ih);
                widgets.form.port_row.set_text(&ip.to_string());
                widgets.form.smtp_row.set_text(sh);
                widgets.form.smtp_port_row.set_text(&sp.to_string());
                widgets.form.server_exp.set_expanded(false);
            } else {
                widgets.form.host_row.set_text("");
                widgets.form.smtp_row.set_text("");
                widgets
                    .form
                    .port_row
                    .set_text(if jmap { "443" } else { "993" });
                widgets.form.server_exp.set_expanded(true);
            }
            // A JMAP server is one address, and it sends the mail
            // itself: no SMTP rows.
            widgets.form.host_row.set_title(&if jmap {
                i18n("Server (host name or URL)")
            } else {
                i18n("IMAP server")
            });
            widgets.form.port_row.set_title(&if jmap {
                i18n("Port")
            } else {
                i18n("IMAP port")
            });
            widgets.form.smtp_row.set_visible(!jmap);
            widgets.form.smtp_port_row.set_visible(!jmap);
            // Fastmail over JMAP takes an API token, not the password (#356).
            widgets.form.pass_row.set_title(&if p.wizard_token() {
                i18n("API token")
            } else {
                i18n("Password")
            });
            let hint = p.wizard_hint();
            widgets.form.hint_lbl.set_visible(!hint.is_empty());
            widgets.form.hint_lbl.set_text(&i18n(hint));
        }
    }

    pub(super) fn on_test_and_add(
        &mut self,
        widgets: &WelcomeWidgets,
        sender: ComponentSender<Self>,
    ) {
        let email = widgets.form.email_row.text().trim().to_string();
        let password = widgets.form.pass_row.text().to_string();
        // Derive missing servers from the address's domain — the common
        // convention, and the expander is right there to correct it.
        let provider = wizard_provider(widgets.form.provider_row.selected());
        let oauth = provider.is_some_and(|p| p.wizard_is_oauth());
        let protocol = provider.map(|p| p.wizard_protocol()).unwrap_or_default();
        let domain = email.split('@').nth(1).unwrap_or("").to_string();
        let mut host = widgets.form.host_row.text().trim().to_string();
        // A JMAP server's address is the user's to give; imap.<domain>
        // is the convention only for IMAP.
        if host.is_empty() && !domain.is_empty() && protocol != Protocol::Jmap {
            host = format!("imap.{domain}");
            widgets.form.host_row.set_text(&host);
        }
        let mut smtp = widgets.form.smtp_row.text().trim().to_string();
        if smtp.is_empty() && !domain.is_empty() && protocol != Protocol::Jmap {
            smtp = format!("smtp.{domain}");
            widgets.form.smtp_row.set_text(&smtp);
        }
        if oauth {
            self.sign_in_and_add(widgets, &sender, email, host, smtp);
            return;
        }
        if email.is_empty() || password.is_empty() || host.is_empty() {
            widgets.form.status_lbl.set_css_classes(&["error"]);
            widgets
                .form
                .status_lbl
                .set_text(&i18n("Enter your email address and password first."));
            return;
        }
        let account = AccountConfig {
            name: widgets.form.name_row.text().trim().to_string(),
            email: email.clone(),
            protocol,
            imap_host: host,
            imap_port: widgets
                .form
                .port_row
                .text()
                .trim()
                .parse()
                .unwrap_or(if protocol == Protocol::Jmap { 443 } else { 993 }),
            smtp_host: smtp,
            smtp_port: widgets
                .form
                .smtp_port_row
                .text()
                .trim()
                .parse()
                .unwrap_or(587),
            username: email,
            password,
            jmap_token: provider.is_some_and(|p| p.wizard_token()),
            jmap_generic: provider.is_some_and(|p| p.wizard_jmap_generic()),
            ..blank_account()
        };
        self.test_seq += 1;
        let seq = self.test_seq;
        widgets.form.test_spinner.set_visible(true);
        widgets.form.test_spinner.start();
        widgets.form.status_lbl.set_css_classes(&["secondary-text"]);
        widgets
            .form
            .status_lbl
            .set_text(&i18n("Checking the connection…"));
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

    pub(super) fn on_tested(
        &mut self,
        account: Box<AccountConfig>,
        result: ConnTest,
        seq: u32,
        widgets: &WelcomeWidgets,
        sender: ComponentSender<Self>,
    ) {
        if seq != self.test_seq {
            // A newer test was started (the user corrected a field and
            // clicked again); this stale verdict no longer matters.
            return;
        }
        widgets.form.test_spinner.stop();
        widgets.form.test_spinner.set_visible(false);
        let email = account.email.clone();
        match (&result.incoming, &result.smtp) {
            (Ok(_), Ok(_)) => {
                self.added.push(account.email.clone());
                widgets.form.status_lbl.set_css_classes(&["success"]);
                widgets.form.status_lbl.set_text(&i18n_f(
                    "✓ {email} — connected and added",
                    &[("email", &(email).to_string())],
                ));
                widgets.form.pass_row.set_text("");
                widgets.form.email_row.set_text("");
                let _ = sender.output(WelcomeOutput::AddAccount(account));
            }
            (Err(e), _) => {
                widgets.form.status_lbl.set_css_classes(&["error"]);
                widgets.form.status_lbl.set_text(&i18n_f(
                    "{email} — mail server: {e}",
                    &[("email", &(email).to_string()), ("e", &(e).to_string())],
                ));
            }
            (_, Err(e)) => {
                widgets.form.status_lbl.set_css_classes(&["error"]);
                widgets.form.status_lbl.set_text(&i18n_f(
                    "{email} — sending (SMTP): {e}",
                    &[("email", &(email).to_string()), ("e", &(e).to_string())],
                ));
            }
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
        widgets.form.test_spinner.stop();
        widgets.form.test_spinner.set_visible(false);
        widgets.form.add_btn.set_sensitive(true);
        match result {
            Ok((refresh, who)) => {
                // Microsoft's account is named for the mailbox the
                // sign-in reached (#329).
                if let Some((name, address)) = who {
                    if !address.is_empty() {
                        account.email = address.clone();
                        account.username = address;
                    }
                    if account.name.is_empty() {
                        account.name = name;
                    }
                }
                let email = account.email.clone();
                account.oauth_refresh = refresh;
                self.added.push(email.clone());
                widgets.form.status_lbl.set_css_classes(&["success"]);
                widgets.form.status_lbl.set_text(&i18n_f(
                    "✓ {email} — signed in and added",
                    &[("email", &email)],
                ));
                widgets.form.email_row.set_text("");
                let _ = sender.output(WelcomeOutput::AddAccount(account));
            }
            Err(e) => {
                widgets.form.status_lbl.set_css_classes(&["error"]);
                widgets
                    .form
                    .status_lbl
                    .set_text(&i18n_f("Sign-in failed: {e}", &[("e", &e)]));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_wizard_lists_every_provider_but_google() {
        assert_eq!(wizard_providers().count(), PROVIDERS.len() - 1);
        assert!(wizard_providers().all(|p| p.wizard_label() != "Google (Gmail)"));
        assert!(wizard_provider(0).is_some_and(|p| p.wizard_is_manual()));
        assert!(wizard_provider(PROVIDERS.len() as u32).is_none());
    }
}
