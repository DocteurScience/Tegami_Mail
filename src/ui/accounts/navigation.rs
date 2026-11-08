//! Moving between the panel's pages: the account page and its sub-pages,
//! pushed over the preferences dialog (screens 28 to 34); the Add account
//! menu (27) and the GNOME Online Accounts page it opens; and what the
//! settings window asks of the panel when it closes.

use super::editor::editor_up;
use super::*;

impl AccountsWindow {
    /// The preferences dialog the account list sits in, once the dialog
    /// has taken it onto its Accounts page.
    fn dialog(widgets: &AccountsWindowWidgets) -> Option<adw::PreferencesDialog> {
        widgets
            .accounts
            .accounts_list
            .ancestor(adw::PreferencesDialog::static_type())
            .and_downcast()
    }

    /// Slide the account page in over the dialog.
    pub(super) fn push_editor(&self, widgets: &AccountsWindowWidgets) {
        if editor_up(widgets) {
            return;
        }
        if let Some(dialog) = Self::dialog(widgets) {
            dialog.push_subpage(&widgets.form.editor_page);
        }
    }

    /// Leave the account page, back to the Accounts page. Save, Remove and
    /// the GNOME Online Accounts switch all act on the account page itself,
    /// so it is the page on top.
    pub(super) fn pop_editor(&self, widgets: &AccountsWindowWidgets) {
        if editor_up(widgets) {
            if let Some(dialog) = Self::dialog(widgets) {
                dialog.pop_subpage();
            }
        }
    }

    /// The account page's rows say what is behind them (screen 28).
    pub(super) fn refresh_links(&self, widgets: &AccountsWindowWidgets) {
        let f = &widgets.form;
        let incoming = match form_protocol(widgets) {
            Protocol::Pop3 => "POP3",
            Protocol::Jmap => "JMAP",
            Protocol::Graph => "Microsoft 365",
            _ => "IMAP",
        };
        let host = trimmed(&f.host_row);
        let smtp = trimmed(&f.smtp_row);
        let mut servers = if host.is_empty() {
            incoming.to_string()
        } else {
            format!("{incoming} {host}")
        };
        if !smtp.is_empty() {
            servers.push_str(&format!(" \u{b7} SMTP {smtp}"));
        }
        f.servers_link.set_subtitle(&servers);
        let aliases = self.alias_edits.len() as u32;
        f.aliases_link.set_subtitle(&if aliases == 0 {
            String::new()
        } else {
            ni18n_f(
                "{n} address",
                "{n} addresses",
                aliases,
                &[("n", &aliases.to_string())],
            )
        });
        // An empty one stands for "none yet".
        let signatures = self
            .sig_edits
            .iter()
            .filter(|s| !s.html.trim().is_empty())
            .count() as u32;
        f.signatures_link.set_subtitle(&if signatures == 0 {
            String::new()
        } else {
            ni18n_f(
                "{n} signature",
                "{n} signatures",
                signatures,
                &[("n", &signatures.to_string())],
            )
        });
        let key = f
            .pgp_key_row
            .selected_item()
            .and_downcast::<gtk::StringObject>()
            .map(|o| o.string().to_string())
            .unwrap_or_else(|| i18n("Automatic"));
        f.openpgp_link
            .set_subtitle(&i18n_f("Key: {key}", &[("key", &key.to_lowercase())]));
    }

    /// Showcase: open a sub-page of the account page by name.
    pub(super) fn on_debug_subpage(&self, sub: &str, widgets: &AccountsWindowWidgets) {
        let at = [
            "servers",
            "sync",
            "folders",
            "aliases",
            "signatures",
            "openpgp",
        ]
        .iter()
        .position(|s| *s == sub);
        if let (Some(i), Some(dialog)) = (at, Self::dialog(widgets)) {
            if let Some(page) = widgets.form.sub_pages.get(i) {
                dialog.push_subpage(page);
            }
        }
    }

    /// Tell the dialog how many tags, filters and smart folders there are.
    pub(super) fn send_counts(&self, sender: &ComponentSender<Self>) {
        let _ = sender.output(AccountsOutput::Counts {
            tags: self.tags.len(),
            filters: self.filter_rules.len(),
            smart: self.smart_folders.len(),
        });
    }

    /// The Add account menu (screen 27), under its row.
    pub(super) fn on_add_menu(
        &mut self,
        widgets: &AccountsWindowWidgets,
        sender: &ComponentSender<Self>,
    ) {
        let Some(row) = widgets.accounts.accounts_list.last_child() else {
            return;
        };
        let emit = |msg: fn() -> AccountsInput| {
            let s = sender.input_sender().clone();
            move || s.emit(msg())
        };
        let mut second = Vec::new();
        // There is one On This Computer at most.
        if !self.accounts.iter().any(AccountConfig::is_local) {
            second.push(crate::ui::context_menu::MenuEntry::new(
                i18n("Local mail on this computer"),
                emit(|| AccountsInput::AddLocal),
            ));
        }
        second.push(crate::ui::context_menu::MenuEntry::new(
            i18n("Import from Thunderbird or Evolution…"),
            emit(|| AccountsInput::PickImport),
        ));
        let sections = vec![
            vec![
                crate::ui::context_menu::MenuEntry::new(
                    i18n("E-mail account…"),
                    emit(|| AccountsInput::AddAccount),
                ),
                crate::ui::context_menu::MenuEntry::new(
                    i18n("GNOME Online Account…"),
                    emit(|| AccountsInput::ShowGoa),
                ),
            ],
            second,
        ];
        // Under the row's start, as screen 27 draws it, rather than
        // centred on its left edge.
        let y = f64::from(row.height());
        let x = f64::from(row.width()).min(400.0) / 2.0;
        crate::ui::context_menu::show_context_menu(&row, x, y, sections);
    }

    /// The GNOME Online Accounts that can be switched on, on their page.
    pub(super) fn on_show_goa(
        &mut self,
        widgets: &AccountsWindowWidgets,
        sender: ComponentSender<Self>,
    ) {
        if widgets.accounts.goa_page.parent().is_none() {
            if let Some(dialog) = Self::dialog(widgets) {
                dialog.push_subpage(&widgets.accounts.goa_page);
            }
        }
        // Read again: one may have been added in GNOME Settings since.
        self.on_refresh_goa(widgets, sender);
    }

    pub(super) fn on_close_editor(&mut self, widgets: &AccountsWindowWidgets) {
        if widgets
            .nav
            .visible_page()
            .and_then(|p| p.tag())
            .is_some_and(|t| t == "filter")
        {
            widgets.nav.pop();
        }
    }
}
