//! The Settings window and its accounts panel.

use adw::prelude::*;
use relm4::prelude::*;

use crate::config::AccountConfig;
use crate::i18n::i18n;
use crate::ui::accounts::{AccountsOutput, AccountsWindow};
use crate::ui::preferences::{PrefInit, PrefInput, PrefOutput, Preferences};

use super::demo::demo_mode;
use super::model::AppModel;
use super::msg::AppMsg;
use crate::ui::dialog::confirm_destructive;

/// What the Settings window reports, as the message it means here.
fn prefs_output_msg(out: PrefOutput) -> AppMsg {
    match out {
        PrefOutput::PageShown(id) => AppMsg::SettingsPageShown(id),
        PrefOutput::Closed => AppMsg::ClosePreferences,
        // Every other output is a setting, which AppMsg::Pref applies.
        other => AppMsg::Pref(other),
    }
}

impl AppModel {
    /// Build the Settings window and its accounts panel, hidden. It is
    /// kept for the life of the app once built: `open_settings_window`
    /// presents it, and closing it hides it, so opening never waits on
    /// hundreds of rows being built and laid out again.
    fn build_settings_window(&mut self, sender: &ComponentSender<Self>, on_accounts: bool) {
        let t_open = std::time::Instant::now();
        // Pass accounts in display order. Passwords stay in the keyring until
        // an account's editor opens (the panel fetches them then, off the
        // main thread): reading every account's secrets up front cost a
        // keyring round trip or three per account, which is what made this
        // window slow to appear.
        let init = self.accounts_panel_init();
        self.accounts_seed = Some(format!("{init:?}"));
        let accounts = Self::launch_accounts_panel(init, sender);
        let init = self.pref_init(&accounts, on_accounts);
        let prefs = Preferences::builder()
            .launch(init)
            .forward(sender.input_sender(), prefs_output_msg);
        accounts.emit(crate::ui::accounts::AccountsInput::SetFolderChoices(
            self.folder_choice_map(),
        ));
        accounts.emit(crate::ui::accounts::AccountsInput::SetTagTotals(
            self.tag_totals.clone(),
        ));
        self.arm_settings_showcase(&accounts, sender);
        self.accounts_win = Some(accounts);
        tracing::debug!("settings window: built in {:?}", t_open.elapsed());
        self.prefs = Some(prefs);
    }

    /// The Settings window's first values: every setting as it stands.
    fn pref_init(&self, accounts: &Controller<AccountsWindow>, on_accounts: bool) -> PrefInit {
        PrefInit {
            auto_remote_content: self.auto_remote_content,
            show_remote_banner: self.show_remote_banner,
            receipts: self.receipts,
            request_receipts: self.request_receipts,
            returned_receipts: self.returned_receipts,
            gravatar: self.gravatar,
            sender_logos: self.sender_logos,
            fetch_interval_secs: self.fetch_interval_secs,
            push: self.push,
            thread_newest_first: self.thread_newest_first,
            confirm_thread_delete: self.confirm_thread_delete,
            compose_format: self.compose_format,
            follow_plain: self.follow_plain,
            signature_position: self.signature_position,
            signature_dashes: self.signature_dashes,
            fold_quote: self.fold_quote,
            reply_header: self.reply_header.clone(),
            undo_send_secs: self.undo_send_secs,
            notifications: self.notifications_enabled,
            notification_content: self.notification_content,
            notify_receipts: self.notify_receipts,
            read_mark: self.read_mark,
            swipe_enabled: self.swipe_enabled,
            files: self.files_prefs,
            compose_default_from: self.compose_default_from.clone(),
            paste_plain: self.paste_plain,
            return_paragraph: self.return_paragraph,
            toolbar_expanded: self.toolbar_expanded,
            compose_font: self.compose_font.clone(),
            compose_font_size: self.compose_font_size,
            compose_zoom: self.compose_zoom,
            spellcheck: self.spellcheck,
            spellcheck_langs: self.spellcheck_langs.clone(),
            app_theme: self.app_theme,
            single_key_shortcuts: self.single_key.get(),
            gmail_keys: self.gmail_keys.get(),
            keep_downloads_days: self.keep_downloads_days,
            run_in_background: self.run_in_background.get(),
            autostart: self.autostart,
            accounts_panel: accounts.widget().clone().upcast::<gtk::Widget>(),
            accounts_sender: accounts.sender().clone(),
            // Sidebar order, so the list reads like the accounts do (#261).
            identities: self.identities_in_sidebar_order(),
            start_on_accounts: on_accounts,
            start_page: if on_accounts {
                Some("accounts".to_string())
            } else {
                self.last_settings_page.clone()
            },
        }
    }

    /// Every enabled account's identities, its own address and then its
    /// aliases, in sidebar order so the list reads like the accounts do
    /// (#261). An alias without a name of its own takes the account's.
    fn identities_in_sidebar_order(&self) -> Vec<(String, String)> {
        let mut enabled: Vec<&AccountConfig> = self.config.iter().filter(|a| a.enabled).collect();
        enabled.sort_by_key(|a| {
            self.account_order
                .iter()
                .position(|e| *e == a.email)
                .unwrap_or(usize::MAX)
        });
        enabled
            .into_iter()
            .flat_map(|a| {
                std::iter::once((a.name.clone(), a.email.clone())).chain(a.aliases.iter().map(
                    |al| {
                        let (name, addr) = crate::config::split_identity(&al.identity);
                        (
                            if name.is_empty() {
                                a.name.clone()
                            } else {
                                name
                            },
                            addr,
                        )
                    },
                ))
            })
            .collect()
    }

    /// Showcase hooks for the Settings window (demo only), each a beat
    /// after the panel is up.
    fn arm_settings_showcase(
        &self,
        accounts: &Controller<AccountsWindow>,
        sender: &ComponentSender<Self>,
    ) {
        // Showcase hook: TEGAMI_SHOWCASE_EDIT_FILTER=<index> opens that
        // filter rule's editor once the panel is up, for a capture.
        // Sent through the app so it reaches whichever panel is current
        // when it fires: an open right after the pre-warm may have swapped
        // in a fresh one.
        if let Some(Ok(i)) = std::env::var("TEGAMI_SHOWCASE_EDIT_FILTER")
            .ok()
            .map(|v| v.parse::<usize>())
        {
            if demo_mode() {
                let s = sender.clone();
                gtk::glib::timeout_add_seconds_local_once(2, move || {
                    s.input(AppMsg::ShowcaseEditFilter(i));
                });
            }
        }
        // TEGAMI_SHOWCASE_FIND_TAGS=1 runs the tag finder once the panel is
        // up (the demo backend answers with a fixed set); the report dialog
        // captures itself two seconds after it appears.
        if std::env::var("TEGAMI_SHOWCASE_FIND_TAGS").is_ok() && demo_mode() {
            let s = sender.clone();
            gtk::glib::timeout_add_seconds_local_once(2, move || s.input(AppMsg::FindTags));
        }
        // TEGAMI_SHOWCASE_EDIT_TAG=<index> likewise opens that tag's editor
        // (#147); an index past the end opens the Add Tag dialog.
        if let Some(Ok(i)) = std::env::var("TEGAMI_SHOWCASE_EDIT_TAG")
            .ok()
            .map(|v| v.parse::<usize>())
        {
            if demo_mode() {
                let a = accounts.sender().clone();
                let n = self.tags.len();
                gtk::glib::timeout_add_seconds_local_once(2, move || {
                    let _ = a.send(if i < n {
                        crate::ui::accounts::AccountsInput::EditTag(i)
                    } else {
                        crate::ui::accounts::AccountsInput::AddTag
                    });
                });
            }
        }
        // TEGAMI_SHOWCASE_EDIT_SMART=<index> likewise opens that smart
        // folder's editor (#417); an index past the end opens a new one.
        if let Some(Ok(i)) = std::env::var("TEGAMI_SHOWCASE_EDIT_SMART")
            .ok()
            .map(|v| v.parse::<usize>())
        {
            if demo_mode() {
                let a = accounts.sender().clone();
                let n = self.smart_folders.len();
                gtk::glib::timeout_add_seconds_local_once(2, move || {
                    let _ = a.send(if i < n {
                        crate::ui::accounts::AccountsInput::EditSmartFolder(i)
                    } else {
                        crate::ui::accounts::AccountsInput::AddSmartFolder
                    });
                });
            }
        }
    }

    /// A fresh accounts panel over the current accounts (in display
    /// order), filters, tags and sender lists. Passwords stay in the keyring
    /// until an account's editor opens (the panel fetches them then, off
    /// the main thread): reading every account's secrets up front cost a
    /// keyring round trip or three per account, which is what made this
    /// window slow to appear with many accounts.
    fn accounts_panel_init(&self) -> crate::ui::accounts::AccountsInit {
        let order = self.ordered_emails();
        let mut accounts: Vec<AccountConfig> = Vec::new();
        for email in &order {
            if let Some(a) = self.config.iter().find(|c| &c.email == email) {
                accounts.push(a.clone());
            }
        }
        for c in &self.config {
            if !accounts.iter().any(|a| a.email == c.email) {
                accounts.push(c.clone());
            }
        }
        // Demo mode: the sample accounts exist only at the backend layer, so
        // the Accounts panel would sit empty in screenshots — hand it
        // matching stand-in configs instead.
        if accounts.is_empty() && demo_mode() {
            accounts = self.demo_config.clone();
        }
        // The accounts panel component (embedded behind the "Accounts" tab).
        crate::ui::accounts::AccountsInit {
            accounts,
            allowed_senders: self.allowed_senders.clone(),
            blacklist: self.blacklist.clone(),
            filters: self.filters.clone(),
            tags: self.tags.clone(),
            smart_folders: self.smart_folders.clone(),
        }
    }

    /// Launch an accounts panel over `init`.
    fn launch_accounts_panel(
        init: crate::ui::accounts::AccountsInit,
        sender: &ComponentSender<Self>,
    ) -> Controller<AccountsWindow> {
        let accounts = AccountsWindow::builder().launch(init).forward(
            sender.input_sender(),
            |out| match out {
                AccountsOutput::Saved {
                    original_email,
                    account,
                } => AppMsg::AccountSaved {
                    original_email,
                    account,
                },
                AccountsOutput::Removed { email } => AppMsg::AccountRemoved { email },
                AccountsOutput::ImportMail { sources, mbox } => {
                    AppMsg::ImportLocalMail { sources, mbox }
                }
                AccountsOutput::Reordered(emails) => AppMsg::AccountsReordered(emails),
                AccountsOutput::EnabledChanged { email, enabled } => {
                    AppMsg::AccountEnabledChanged { email, enabled }
                }
                AccountsOutput::ImportGoa(account) => AppMsg::ImportGoaAccount(account),
                AccountsOutput::EditorOpen(open) => AppMsg::SettingsEditorOpen(open),
                AccountsOutput::OpenSettingsPage(page) => AppMsg::SettingsPushPage(page),
                AccountsOutput::Counts {
                    tags,
                    filters,
                    smart,
                } => AppMsg::SettingsCounts {
                    tags,
                    filters,
                    smart,
                },
                AccountsOutput::AddSender(addr) => AppMsg::AddSender(addr),
                AccountsOutput::RemoveSender(addr) => AppMsg::RemoveSender(addr),
                AccountsOutput::AddBlacklist(addr) => AppMsg::AddBlacklist(addr),
                AccountsOutput::RemoveBlacklist(addr) => AppMsg::RemoveBlacklist(addr),
                AccountsOutput::SetFilters(rules) => AppMsg::SetFilters(rules),
                AccountsOutput::ApplyFilters => AppMsg::ApplyFilters(Vec::new()),
                AccountsOutput::SetTags(tags) => AppMsg::SetTags(tags),
                AccountsOutput::SetSmartFolders(folders) => AppMsg::SetSmartFolders(folders),
                AccountsOutput::FindTags => AppMsg::FindTags,
                AccountsOutput::WantQuota(email) => AppMsg::WantQuota(email),
                AccountsOutput::WantIdentities(email) => AppMsg::WantIdentities(email),
            },
        );

        // The host window: the preferences component, carrying the accounts
        // panel behind its other tab.
        accounts
    }

    /// Opens Settings on the accounts side at the page `page_id`, then
    /// hands its accounts panel `input` (an editor to open, a rule to add).
    pub(super) fn open_settings_at(
        &mut self,
        sender: &ComponentSender<Self>,
        page_id: &str,
        input: crate::ui::accounts::AccountsInput,
    ) {
        self.open_settings_window(sender, true, false);
        // The dialog hands `input` on once the page is up: the account
        // page it may open is pushed over the dialog, and would be popped
        // again by a page change that came after it.
        if let Some(p) = &self.prefs {
            p.emit(PrefInput::ShowPageThen(page_id.to_string(), input));
        }
    }

    /// Show Settings: the window kept from a previous open (its accounts
    /// panel rebuilt over the current accounts) or a newly built one.
    ///
    /// Open (or focus) the combined Settings window on the
    /// requested panel. When `add_new`, jump straight to the "add account"
    /// form — used by the empty-state "Add first account" button.
    pub(super) fn open_settings_window(
        &mut self,
        sender: &ComponentSender<Self>,
        on_accounts: bool,
        add_new: bool,
    ) {
        let t_open = std::time::Instant::now();
        // A closed dialog has no parent until it is presented again.
        let hidden = self
            .prefs
            .as_ref()
            .is_some_and(|p| p.widget().parent().is_none());
        if self.prefs.is_none() {
            self.build_settings_window(sender, on_accounts);
        } else if hidden {
            // Accounts, filters, tags and senders may have changed since: a
            // fresh panel over today's state — or the kept one, when
            // nothing did, with any editor it was left in closed.
            let init = self.accounts_panel_init();
            let seed = format!("{init:?}");
            let page = if on_accounts {
                "accounts".to_string()
            } else {
                self.last_settings_page
                    .clone()
                    .unwrap_or_else(|| "general".to_string())
            };
            if self.accounts_seed.as_deref() == Some(seed.as_str()) {
                if let Some(a) = &self.accounts_win {
                    a.emit(crate::ui::accounts::AccountsInput::CloseEditor);
                }
            } else {
                let accounts = Self::launch_accounts_panel(init, sender);
                // A fresh panel knows no folders until told: without this
                // the filter editor's "Move to" list held only "Leave in
                // Inbox" whenever Settings reopened after a change.
                accounts.emit(crate::ui::accounts::AccountsInput::SetFolderChoices(
                    self.folder_choice_map(),
                ));
                self.accounts_seed = Some(seed);
                if let Some(p) = &self.prefs {
                    p.emit(PrefInput::SetAccountsPanel {
                        panel: accounts.widget().clone().upcast(),
                        sender: accounts.sender().clone(),
                    });
                }
                self.accounts_win = Some(accounts);
            }
            if let Some(p) = &self.prefs {
                p.emit(PrefInput::EditorOpen(None));
                p.emit(PrefInput::ShowPageById(page));
            }
        } else if on_accounts {
            // Already showing: switch to Accounts if asked, else leave the
            // window on whatever category it is showing.
            if let Some(p) = &self.prefs {
                p.emit(PrefInput::ShowAccounts(true));
            }
        }
        if let Some(p) = &self.prefs {
            if p.widget().parent().is_none() {
                p.widget().present(Some(&self.window));
            }
        }
        if add_new {
            if let Some(a) = &self.accounts_win {
                a.emit(crate::ui::accounts::AccountsInput::AddAccount);
            }
        }
        tracing::debug!("settings window: presented in {:?}", t_open.elapsed());
        {
            let t = t_open;
            gtk::glib::idle_add_local_once(move || {
                tracing::debug!(
                    "settings window: first idle after open at {:?}",
                    t.elapsed()
                );
            });
            if let Some(p) = &self.prefs {
                p.widget().add_tick_callback(move |_, _| {
                    tracing::debug!("settings window: first frame at {:?}", t.elapsed());
                    gtk::glib::ControlFlow::Break
                });
            }
        }
    }

    /// Confirm and remove an account (drops its keyring password too).
    pub(super) fn confirm_remove_account(&self, account_id: u32, sender: &ComponentSender<Self>) {
        let Some(email) = self.email_of(account_id) else {
            return;
        };
        let label = self.account_label(account_id);
        let s = sender.clone();
        confirm_destructive(
            Some(self.window.upcast_ref()),
            &i18n("Remove Account?"),
            &format!(
                "Remove {label} from Tegami Mail? Its saved password is deleted. \
                 Mail on the server is not affected."
            ),
            "remove",
            &i18n("Remove"),
            move || s.input(AppMsg::AccountRemoved { email }),
        );
    }

    /// The messages the router sends here (see `AppModel::route`).
    pub(super) fn on_settings_window(&mut self, msg: AppMsg, sender: ComponentSender<Self>) {
        match msg {
            AppMsg::ShowSettingsPage(id) => {
                if let Some(p) = &self.prefs {
                    p.emit(PrefInput::ShowPageById(id));
                }
            }

            AppMsg::OpenSettings => {
                // After the first open of the session the window returns
                // to where it was.
                self.open_settings_window(&sender, false, false);
            }

            AppMsg::SettingsPageShown(id) => {
                self.last_settings_page = Some(id);
            }

            AppMsg::OpenPreferences => self.open_settings_window(&sender, false, false),

            // Closing the combined Settings window hides it (the window's
            // own hide-on-close); both panels stay for the next open, which
            // then only rebuilds the accounts panel.
            AppMsg::ClosePreferences => {
                // The window only hides; let the signature editor's web
                // process go with it (#221).
                if let Some(a) = &self.accounts_win {
                    a.emit(crate::ui::accounts::AccountsInput::ReleaseSignatureEditor);
                }
            }

            // Build the Settings window ahead of its first open, hidden and
            // realized, a moment after startup: its first appearance is
            // then a present() rather than a build, a layout and a paint.
            AppMsg::DebugCloseSettings => {
                if let Some(p) = &self.prefs {
                    p.widget().close();
                }
            }

            AppMsg::PrewarmSettings => {
                if self.prefs.is_none() {
                    self.build_settings_window(&sender, false);
                    // A dialog cannot be realized before it is presented:
                    // building it is what is done ahead.
                }
            }

            AppMsg::SettingsEditorOpen(open) => {
                if let Some(p) = &self.prefs {
                    p.emit(PrefInput::EditorOpen(open));
                }
            }

            AppMsg::SettingsCounts {
                tags,
                filters,
                smart,
            } => {
                if let Some(p) = &self.prefs {
                    p.emit(PrefInput::SetCounts {
                        tags,
                        filters,
                        smart,
                    });
                }
            }

            AppMsg::SettingsPushPage(page) => {
                if let Some(p) = &self.prefs {
                    p.emit(PrefInput::PushPage(page));
                }
            }

            AppMsg::ServerIdentities {
                account_id,
                identities,
            } => {
                if let (Some(email), Some(a)) = (self.email_of(account_id), &self.accounts_win) {
                    a.emit(crate::ui::accounts::AccountsInput::ServerIdentities {
                        email,
                        identities: identities.clone(),
                    });
                }
                self.server_identities.insert(account_id, identities);
            }
            AppMsg::WantIdentities(email) => {
                let Some(id) = self
                    .accounts
                    .iter()
                    .find(|a| a.email.eq_ignore_ascii_case(&email))
                    .map(|a| a.id)
                else {
                    return;
                };
                let identities = self.server_identities.get(&id).cloned().unwrap_or_default();
                if let Some(a) = &self.accounts_win {
                    a.emit(crate::ui::accounts::AccountsInput::ServerIdentities {
                        email,
                        identities,
                    });
                }
            }
            _ => unreachable!("not routed to on_settings_window"),
        }
    }
}
