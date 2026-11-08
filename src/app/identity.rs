//! Accounts as the interface shows them: names, colors, faces and the
//! addresses they send from.

use relm4::prelude::*;

use crate::config::{self, AccountConfig};
use crate::ui::message_list::MessageListInput;
use crate::ui::message_view::MessageViewInput;
use crate::ui::message_window::MessageWindowInput;

use super::demo::demo_mode;
use super::model::AppModel;

impl AppModel {
    /// The account to act on by default: the selected folder's account,
    /// else the first in sidebar order. With a merged view open that is the
    /// account a new message starts from, so it has to be the one the
    /// person put on top, not the one added first (#261).
    pub(super) fn active_account(&self) -> u32 {
        self.selected
            .as_ref()
            .map(|s| s.account_id)
            .or_else(|| {
                let first = self.ordered_emails().into_iter().next()?;
                self.accounts
                    .iter()
                    .find(|a| a.email == first)
                    .map(|a| a.id)
            })
            .or_else(|| self.accounts.first().map(|a| a.id))
            .unwrap_or(1)
    }

    /// Resolved avatar/accent color for an account (custom, else auto accent).
    pub(super) fn account_color(&self, account_id: u32) -> String {
        self.account_config(account_id)
            .and_then(|c| c.color.clone())
            .or_else(|| {
                self.accounts
                    .iter()
                    .find(|a| a.id == account_id)
                    .map(|a| a.accent.clone())
            })
            .unwrap_or_else(|| "#3584e4".to_string())
    }

    /// The account that sends for `m`: its own, unless `m` is filed On This
    /// Computer (#552), which sends nothing. Then it is the account the
    /// message was addressed to, by its address or an alias, else the first
    /// in the sidebar that sends.
    pub(super) fn sending_account(&self, m: &crate::models::Message) -> u32 {
        if !self
            .saved_account_config(m.account_id)
            .is_some_and(AccountConfig::is_local)
        {
            return m.account_id;
        }
        let to: Vec<String> = [&m.to, &m.cc]
            .into_iter()
            .flat_map(|f| crate::worker::parse_recipients(f))
            .map(|(_, e)| e.to_lowercase())
            .collect();
        let sends = |c: &&AccountConfig| c.enabled && !c.is_local();
        let addressed = self.config.iter().filter(sends).find(|c| {
            to.contains(&c.email.to_lowercase())
                || c.aliases
                    .iter()
                    .any(|al| to.contains(&config::split_identity(&al.identity).1.to_lowercase()))
        });
        let first = || {
            self.ordered_emails()
                .into_iter()
                .find_map(|e| self.config.iter().filter(sends).find(|c| c.email == e))
        };
        addressed
            .or_else(first)
            .and_then(|c| self.config.iter().position(|x| std::ptr::eq(x, c)))
            .map_or(m.account_id, |i| i as u32 + 1)
    }

    /// Every address the accounts send from, their aliases included, lower
    /// case.
    pub(super) fn own_addresses(&self) -> std::collections::HashSet<String> {
        self.effective_config()
            .iter()
            .filter(|a| !a.is_local())
            .flat_map(|a| {
                std::iter::once(a.email.clone()).chain(
                    a.aliases
                        .iter()
                        .map(|alias| config::split_identity(&alias.identity).1),
                )
            })
            .map(|e| e.trim().to_lowercase())
            .filter(|e| !e.is_empty())
            .collect()
    }

    /// The configs the UI describes accounts from: the real ones, or the
    /// demo's in-memory stand-ins while no account is configured.
    pub(super) fn effective_config(&self) -> &[AccountConfig] {
        if self.config.is_empty() && demo_mode() {
            &self.demo_config
        } else {
            &self.config
        }
    }

    /// An account's settings by id (an account's id is its place in the
    /// config plus one), the demo's stand-ins included. Id 0 reads the
    /// first account.
    pub(super) fn account_config(&self, account_id: u32) -> Option<&AccountConfig> {
        self.effective_config()
            .get(account_id.saturating_sub(1) as usize)
    }

    /// An account's saved settings by id, never the demo's stand-ins. Id 0
    /// reads the first account.
    pub(super) fn saved_account_config(&self, account_id: u32) -> Option<&AccountConfig> {
        self.config.get(account_id.saturating_sub(1) as usize)
    }

    /// Tell the views which addresses are the user's own, every account's
    /// and alias's, so the list's correspondents and the reader's To line
    /// say "me" for them. Refreshed with the sidebar, which is rebuilt
    /// whenever the accounts change. (An account's circle is always drawn
    /// from its initials and colour: the picture, emoji and Gravatar
    /// choices went with docs/DESIGN.md, "Preferences".)
    pub(super) fn refresh_own_faces(&self) {
        crate::services::avatar::set_own_addresses(self.own_addresses());
    }

    /// Redraw the faces on screen after what a mailbox shows has changed
    /// (#189): the list rebuilds its rows, and the reader — in the pane and
    /// in any popped-out window — its cards.
    pub(super) fn refresh_faces(&self) {
        self.message_list
            .emit(MessageListInput::ContactPhotosChanged);
        self.message_view.emit(MessageViewInput::FacesChanged);
        self.popouts_emit(|| MessageWindowInput::FacesChanged);
    }

    /// A label for an account in messages (name, else email, else "Account N").
    /// Uses config (available even before the account connects), then live data.
    pub(super) fn account_label(&self, account_id: u32) -> String {
        let pick = |name: &str, email: &str| -> Option<String> {
            if !name.trim().is_empty() {
                Some(name.to_string())
            } else if !email.trim().is_empty() {
                Some(email.to_string())
            } else {
                None
            }
        };
        self.saved_account_config(account_id)
            .and_then(|c| pick(&c.name, &c.email))
            .or_else(|| {
                self.accounts
                    .iter()
                    .find(|a| a.id == account_id)
                    .and_then(|a| pick(&a.name, &a.email))
            })
            .unwrap_or_else(|| format!("Account {account_id}"))
    }

    /// Display name for an account (name, else email).
    pub(super) fn account_name(&self, account_id: u32) -> String {
        // The account's UI label (its sidebar header, the reader's account line).
        self.accounts
            .iter()
            .find(|a| a.id == account_id)
            .map(|a| a.label.clone())
            .unwrap_or_default()
    }

    /// Whether the list on show mixes accounts: Favorites, a tag or a smart
    /// folder across every account, or a search's results.
    pub(super) fn view_spans_accounts(&self) -> bool {
        self.accounts.len() > 1
            && (self.unified
                || self.smart_view.is_some()
                || self.search_active
                || self
                    .tag_view
                    .as_ref()
                    .is_some_and(|(scope, _)| scope.is_none()))
    }

    /// The account the reader names above the subject, as plain text: only
    /// where the list mixes accounts (docs/DESIGN.md, "Reading pane and
    /// conversations"), since in one account's folder it says nothing the
    /// sidebar does not.
    pub(super) fn reader_account_name(&self, account_id: u32) -> Option<String> {
        self.view_spans_accounts()
            .then(|| self.account_name(account_id))
    }

    /// The account email for an id, if known.
    pub(super) fn email_of(&self, account_id: u32) -> Option<String> {
        self.accounts
            .iter()
            .find(|a| a.id == account_id)
            .map(|a| a.email.clone())
    }

    /// Every address this account sends as: its own, plus its send-as
    /// aliases (#34). Lower-cased comparison is the caller's job.
    pub(super) fn own_identities(&self, account_id: u32) -> Vec<String> {
        let Some(cfg) = self.account_config(account_id) else {
            return self.email_of(account_id).into_iter().collect();
        };
        std::iter::once(cfg.email.clone())
            .chain(
                cfg.aliases
                    .iter()
                    .map(|al| config::split_identity(&al.identity).1),
            )
            .filter(|a| !a.trim().is_empty())
            .collect()
    }
}
