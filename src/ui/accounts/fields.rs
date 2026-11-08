//! The account editor's fields as they change: the provider
//! and protocol, the connection test, and what the app reports about the
//! account (folders, quota, server identities).

use super::*;

impl AccountsWindow {
    pub(super) fn on_debug_provider(&mut self, label: String, widgets: &AccountsWindowWidgets) {
        if let Some(i) = PROVIDERS.iter().position(|p| p.label == label) {
            widgets.form.provider_row.set_selected(i as u32);
        }
    }

    pub(super) fn on_email_changed(&mut self, widgets: &AccountsWindowWidgets) {
        let email = trimmed(&widgets.form.email_row);
        let label = widgets.form.label_row.text().to_string();
        // Mirror while the label is still tracking the email (or empty);
        // once the user types a custom label, stop.
        if label.is_empty() || label == self.label_synced {
            widgets.form.label_row.set_text(&email);
        }
        self.label_synced = email;
    }

    pub(super) fn on_set_server_saves_sent(&mut self, on: bool, widgets: &AccountsWindowWidgets) {
        widgets.form.folder_sent_copy_row.set_sensitive(!on);
        widgets.form.reply_copies_row.set_sensitive(!on);
    }

    pub(super) fn on_set_folder_sort(
        &mut self,
        email: String,
        sort: Option<crate::config::FolderSort>,
        widgets: &AccountsWindowWidgets,
    ) {
        if let Some(i) = self.accounts.iter().position(|a| a.email == email) {
            self.accounts[i].folder_sort = sort;
            if self.editing == Some(i) {
                widgets
                    .form
                    .folder_sort_row
                    .set_selected(sort.map_or(0, |s| s.index() + 1));
            }
        }
    }

    pub(super) fn on_test_connection(
        &mut self,
        widgets: &AccountsWindowWidgets,
        sender: ComponentSender<Self>,
    ) {
        let account = read_account(widgets);
        widgets.form.test_btn.set_sensitive(false);
        show_test_result(widgets, None, &i18n("Testing…"), "");
        sender.oneshot_command(async move {
            let r = tokio::task::spawn_blocking(move || worker::test_connection_blocking(account))
                .await
                .unwrap_or_else(|_| ConnTest {
                    incoming: Err("test could not run".into()),
                    smtp: Err("test could not run".into()),
                });
            AccountsCmd::Test(r)
        });
    }

    pub(super) fn on_protocol_changed(&mut self, widgets: &AccountsWindowWidgets) {
        apply_protocol(widgets);
        self.show_push_row(widgets);
        self.refresh_provider_mark(widgets);
    }

    pub(super) fn on_provider_changed(&mut self, widgets: &AccountsWindowWidgets) {
        // Editing a GOA account: the provider dropdown is hidden and the
        // connection section deliberately not shown — but filling the
        // editor sets the dropdown, whose notify lands here right after
        // EditAccount and would undo that. GNOME owns those fields;
        // leave them hidden.
        let editing_goa = self
            .editing
            .and_then(|i| self.accounts.get(i))
            .is_some_and(|a| a.goa_id.is_some());
        if !editing_goa {
            self.apply_provider(widgets);
        }
        self.show_push_row(widgets);
    }

    pub(super) fn on_set_folder_choices(
        &mut self,
        map: std::collections::HashMap<String, Vec<(String, String)>>,
    ) {
        self.folders_by_email = map;
    }

    pub(super) fn on_server_identities(
        &mut self,
        email: String,
        identities: Vec<(String, String)>,
        widgets: &AccountsWindowWidgets,
        sender: ComponentSender<Self>,
    ) {
        let open = self
            .editing
            .and_then(|i| self.accounts.get(i))
            .is_some_and(|a| a.email.eq_ignore_ascii_case(&email));
        if open && self.server_identities != identities {
            self.server_identities = identities;
            self.rebuild_alias_list(&widgets.form.aliases_list, &sender);
        }
    }

    pub(super) fn on_quota(
        &mut self,
        email: String,
        quota: Option<crate::models::MailboxQuota>,
        widgets: &AccountsWindowWidgets,
    ) {
        let open = self
            .editing
            .and_then(|i| self.accounts.get(i))
            .is_some_and(|a| a.email.eq_ignore_ascii_case(&email));
        if open {
            show_quota(widgets, quota);
        }
    }

    pub(super) fn on_connection_tested(
        &mut self,
        result: ConnTest,
        widgets: &AccountsWindowWidgets,
    ) {
        // One line per side that failed, with what the server said.
        let line = |label: &str, r: &Result<(), String>| {
            match r {
            Ok(()) => None,
            Err(e) if crate::worker::is_hostname_mismatch(e) => Some(format!(
                "{label}: {e}\n{}",
                i18n("The certificate is for another name. If this server is the right one, turn on “Accept a certificate for another name” under Advanced and test again.")
            )),
            Err(e) => Some(format!("{label}: {e}")),
        }
        };
        let protocol = form_protocol(widgets);
        let incoming_label = match protocol {
            Protocol::Pop3 => "POP3",
            Protocol::Jmap => "JMAP",
            _ => "IMAP",
        };
        let mut failures = vec![line(incoming_label, &result.incoming)];
        // A JMAP account sends through the server: no SMTP side.
        if protocol != Protocol::Jmap {
            failures.push(line("SMTP", &result.smtp));
        }
        let failures: Vec<String> = failures.into_iter().flatten().collect();
        if failures.is_empty() {
            show_test_result(
                widgets,
                Some(true),
                &i18n("Connected: receiving and sending work."),
                "",
            );
        } else {
            show_test_result(
                widgets,
                Some(false),
                &i18n("The connection failed."),
                &failures.join("\n"),
            );
        }
        widgets.form.test_btn.set_sensitive(true);
    }
}

/// The connection test's row (screen 29): an icon — a check, or an error
/// sign, none while it runs — and a sentence, the server's own words
/// under it. Both can be copied out (#246).
pub(super) fn show_test_result(
    widgets: &AccountsWindowWidgets,
    ok: Option<bool>,
    title: &str,
    detail: &str,
) {
    let row = &widgets.form.test_result;
    let icon = &widgets.form.test_icon;
    row.set_visible(true);
    row.set_title(title);
    row.set_subtitle(detail);
    row.set_subtitle_selectable(!detail.is_empty());
    for class in ["success", "error"] {
        icon.remove_css_class(class);
    }
    match ok {
        Some(true) => {
            icon.set_icon_name(Some("object-select-symbolic"));
            icon.add_css_class("success");
        }
        Some(false) => {
            icon.set_icon_name(Some("dialog-error-symbolic"));
            icon.add_css_class("error");
        }
        None => icon.set_icon_name(None),
    }
    icon.set_visible(ok.is_some());
}
