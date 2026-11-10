//! The tray icon and its menu, and the launcher badge.

use std::collections::HashSet;

use adw::prelude::*;
use relm4::prelude::*;

use crate::config;
use crate::models::{Account, Message};
use crate::ui::sidebar::SidebarInput;

use super::model::AppModel;
use super::msg::AppMsg;

mod indicator;

pub(super) use indicator::TrayHandle;

/// What identifies a tray mail list for change detection: the total, then
/// each card.
fn tray_mail_key(mail: &indicator::TrayMailList) -> Vec<(u32, u32, String, String)> {
    std::iter::once((u32::MAX, mail.unread, String::new(), String::new()))
        .chain(mail.items.iter().map(|m| {
            (
                m.account_id,
                m.message_id,
                m.heading.clone(),
                m.subject.clone(),
            )
        }))
        .collect()
}

impl AppModel {
    /// The folders whose unread count has grown since they were last
    /// looked at (#343). A folder on screen is being looked at, so its mark
    /// follows its count; elsewhere the mark only comes down, as mail is
    /// read. A folder never seen before starts with what it has, so turning
    /// the setting on, or a new account, marks nothing.
    fn fresh_unread(&self) -> HashSet<(u32, u32)> {
        let on_screen = |a: u32, f: u32| {
            self.window.is_visible()
                && if self.unified {
                    self.is_unified_target(a, f)
                } else {
                    self.selected
                        .as_ref()
                        .is_some_and(|s| s.account_id == a && s.folder_id == f)
                }
        };
        let mut seen = self.unread_seen.borrow_mut();
        let mut changed = false;
        let mut fresh = HashSet::new();
        for (&(a, f), &n) in &self.folder_unread {
            let (Some(email), Some(path)) = (self.email_of(a), self.folder_path(a, f)) else {
                continue;
            };
            let key = format!("{email}\u{1f}{path}");
            let mark = seen.get(&key).copied();
            let next = match mark {
                _ if on_screen(a, f) => n,
                None => n,
                Some(m) => m.min(n),
            };
            if mark != Some(next) {
                seen.insert(key, next);
                changed = true;
            }
            if n > next {
                fresh.insert((a, f));
            }
        }
        if changed {
            config::save_unread_seen(&seen);
        }
        fresh
    }

    /// `push_unread_counts` once the view switch has been drawn: it walks
    /// every folder, and the counts can wait a frame where the list cannot.
    pub(super) fn push_unread_counts_later(&self, sender: &ComponentSender<Self>) {
        let s = sender.input_sender().clone();
        gtk::glib::idle_add_local_once(move || {
            let _ = s.send(AppMsg::PushUnreadCounts);
        });
    }

    /// Update the sidebar's unread badges in place (no rebuild), derived from the
    /// loaded message lists. Cheap enough to call on every read/sync.
    pub(super) fn push_unread_counts(&self) {
        let folders = self.folder_unread.clone();
        let unified = self.unified_unread();
        let fresh = self.seen_counts.then(|| self.fresh_unread());
        self.sidebars_emit(SidebarInput::SetUnread { folders, fresh });
        // The counted total is what GNOME shows beside Tegami Mail in Background
        // Apps, so a process with no window still says what it is there for.
        if self.run_in_background.get() {
            crate::desktop::background::set_status(&crate::desktop::background::status_text(
                unified,
            ));
        }
        // And what the tray icon's dot answers to: the inboxes alone, as
        // its menu says.
        if let Some(tray) = &self.tray {
            tray.set_unread(self.inboxes_unread());
        }
        self.push_launcher_count();
        self.push_tray_mail();
    }

    /// The launcher icon's badge (#271): the same inboxes total as the tray
    /// icon's dot, only the inbox mail that came in while the window was
    /// behind (#431), or nothing, as Settings has it.
    pub(super) fn push_launcher_count(&self) {
        let count = match self.launcher_badge {
            config::LauncherBadge::Off => 0,
            config::LauncherBadge::AllUnread => self.inboxes_unread(),
            config::LauncherBadge::NewSinceFront => self.arrivals.count(|a, f| {
                self.message_cache
                    .get(&(a, f))
                    .into_iter()
                    .flatten()
                    .filter(|m| m.unread)
                    .map(|m| m.uid)
            }),
        };
        crate::desktop::launcher_badge::set_count(count);
    }

    /// Send the launcher badge again even though the count is unchanged, for
    /// when the dock may have dropped it (a lock, a wake).
    pub(super) fn resend_launcher_count(&self) {
        crate::desktop::launcher_badge::resend();
        self.push_launcher_count();
    }

    /// Publish the tray item with the current icon, unread total and mail list.
    pub(super) fn start_tray(&mut self, sender: &ComponentSender<Self>) {
        let unified = self.unified_unread();
        let mail = self.tray_mail.then(|| self.tray_mail_list());
        *self.tray_mail_key.borrow_mut() = mail.as_ref().map(tray_mail_key);
        self.tray = indicator::TrayHandle::start(
            sender.input_sender().clone(),
            self.tray_icon,
            &crate::desktop::app_icon::tray_image(),
            unified,
            mail,
        );
    }

    /// Send the tray menu its mail list when it has changed.
    pub(super) fn push_tray_mail(&self) {
        let Some(tray) = &self.tray else { return };
        let mail = self.tray_mail.then(|| self.tray_mail_list());
        let key = mail.as_ref().map(tray_mail_key);
        if *self.tray_mail_key.borrow() == key {
            return;
        }
        *self.tray_mail_key.borrow_mut() = key;
        tray.set_mail(mail);
    }

    /// The newest unread messages across accounts' inboxes, as tray cards,
    /// with the inboxes' unread total (the cards may be fewer). Filter
    /// destinations are left out: the menu says it lists inboxes.
    fn tray_mail_list(&self) -> indicator::TrayMailList {
        let unread = self.inboxes_unread();
        let items = self.tray_mail_cards();
        indicator::TrayMailList { items, unread }
    }

    fn tray_mail_cards(&self) -> Vec<indicator::TrayMail> {
        let several = self.accounts.len() > 1;
        let mut unread: Vec<(&Account, &Message)> = self
            .accounts
            .iter()
            .filter_map(|a| self.inbox_of(a.id).map(|f| (a, f)))
            .flat_map(|(a, f)| {
                self.message_cache
                    .get(&(a.id, f.id))
                    .into_iter()
                    .flat_map(|msgs| msgs.iter())
                    .filter(|m| m.unread)
                    .map(move |m| (a, m))
            })
            .collect();
        unread.sort_by_key(|x| std::cmp::Reverse(x.1.timestamp));
        unread
            .into_iter()
            .take(indicator::MAIL_LIMIT)
            .map(|(a, m)| {
                let sender = crate::contacts::shown_name_or_addr(&m.from_name, &m.from_addr);
                let mut heading = indicator::clip(&sender, 40);
                if several {
                    heading.push_str(" · ");
                    heading.push_str(&indicator::clip(&a.label, 24));
                }
                if !m.date.is_empty() {
                    heading.push_str(" · ");
                    heading.push_str(&m.date);
                }
                let texture = match crate::services::avatar::lookup(&m.from_addr, self.gravatar) {
                    crate::services::avatar::CacheLookup::Texture(t) => Some(t),
                    _ => None,
                };
                indicator::TrayMail {
                    account_id: m.account_id,
                    folder_id: m.folder_id,
                    message_id: m.id,
                    heading,
                    subject: indicator::clip(&m.subject, 60),
                    preview: indicator::clip(&m.preview, 70),
                    icon: indicator::sender_icon(&m.shown_from_name(), &m.from_addr, texture),
                }
            })
            .collect()
    }

    /// The messages the router sends here (see `AppModel::route`).
    pub(super) fn on_tray(&mut self, msg: AppMsg, _sender: ComponentSender<Self>) {
        match msg {
            AppMsg::QuitFromTray => {
                // The same teardown as Ctrl+Q: geometry saved, then exit.
                relm4::main_application().activate_action("quit", None);
            }

            AppMsg::TrayViewUnread => {
                // The first inbox with unread mail, every account counted
                // (#267); with none, the first account's inbox.
                let inboxes: Vec<_> = self
                    .accounts
                    .iter()
                    .filter_map(|a| self.inbox_of(a.id))
                    .collect();
                if let Some((account_id, folder_id, name, path)) = inboxes
                    .iter()
                    .find(|f| self.folder_unread_of(f) > 0)
                    .or(inboxes.first())
                    .map(|f| (f.account_id, f.id, f.name.clone(), f.path.clone()))
                {
                    self.select_folder(account_id, folder_id, name, path);
                }
            }

            AppMsg::PresentWindow => {
                crate::desktop::session::present_window(&self.window);
            }
            _ => unreachable!("not routed to on_tray"),
        }
    }
}
