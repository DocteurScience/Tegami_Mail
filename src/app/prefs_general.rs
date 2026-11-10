//! Applying general and appearance settings.

use relm4::prelude::*;

use crate::ui::message_list::MessageListInput;
use crate::ui::message_view::MessageViewInput;
use crate::ui::notifications::NotifyInput;
use crate::ui::preferences::PrefOutput;

use super::attachments::prune_downloads;
use super::model::AppModel;
use super::theme::apply_app_theme;

impl AppModel {
    /// The settings changes the router sends here (see `AppModel::route`).
    pub(super) fn on_prefs_general_pref(
        &mut self,
        pref: PrefOutput,
        sender: ComponentSender<Self>,
    ) {
        match pref {
            PrefOutput::SetAppTheme(theme) => {
                if self.app_theme != theme {
                    self.app_theme = theme;
                    apply_app_theme(theme);
                    self.save_settings();
                }
            }

            PrefOutput::SetAvatars(on) => {
                if pref!(self.avatars = on) {
                    self.push_list_look();
                }
            }

            PrefOutput::SetSenderLogos(on) => {
                if pref!(self.sender_logos = on) {
                    self.message_list.emit(MessageListInput::SetSenderLogos(on));
                }
            }

            PrefOutput::SetGravatar(on) => {
                if pref!(self.gravatar = on) {
                    self.message_list.emit(MessageListInput::SetGravatar(on));
                    // Refresh the reader's avatar for the open message.
                    let current = self.current.clone();
                    self.show_message(current, false);
                }
            }

            PrefOutput::SetFetchInterval(secs) => {
                if pref!(self.fetch_interval_secs = secs) {
                    self.arm_auto_fetch(&sender);
                }
            }

            PrefOutput::SetPush(on) => {
                if pref!(self.push = on) {
                    // Workers read the push setting at startup; restart to apply.
                    self.reconnect_all(&sender);
                }
            }

            PrefOutput::SetNotifications(on) => {
                pref!(self.notifications_enabled = on);
            }

            PrefOutput::SetNotificationContent(on) => {
                pref!(self.notification_content = on);
            }

            PrefOutput::SetNotifyReceipts(on) => {
                pref!(self.notify_receipts = on);
            }

            PrefOutput::SetRunInBackground(on) => {
                if self.run_in_background.get() != on {
                    self.run_in_background.set(on);
                    self.save_settings();
                    // Ask the portal as the setting changes, so the permission
                    // dialog appears while the user is looking at the switch they
                    // just moved rather than at some later, unexplained moment.
                    crate::desktop::background::request(on && self.autostart);
                    if !on {
                        crate::desktop::background::set_status("");
                    }
                }
            }

            PrefOutput::SetAutostart(on) => {
                if pref!(self.autostart = on) {
                    crate::desktop::background::request(self.run_in_background.get() && on);
                }
            }

            PrefOutput::SetSingleKey(on) => {
                if self.single_key.get() != on {
                    self.single_key.set(on);
                    crate::ui::keymap::set_key_mode(on, self.gmail_keys.get());
                    self.save_settings();
                }
            }

            PrefOutput::SetKeepDownloads(days) => {
                if self.keep_downloads_days != days {
                    self.keep_downloads_days = days;
                    self.save_settings();
                    prune_downloads(days);
                }
            }

            PrefOutput::SetGmailKeys(on) => {
                if self.gmail_keys.get() != on {
                    self.gmail_keys.set(on);
                    crate::ui::keymap::set_key_mode(self.single_key.get(), on);
                    self.message_list.emit(MessageListInput::SetGmailKeys(on));
                    self.save_settings();
                }
            }

            PrefOutput::SetReadMark(policy) => {
                if pref!(self.read_mark = policy) {
                    self.message_view
                        .emit(MessageViewInput::SetReadMark(policy));
                }
            }

            PrefOutput::SetStatusBarTop(top) => {
                if pref!(self.status_bar_top = top) {
                    self.notifications.emit(NotifyInput::SetAtTop(top));
                }
            }

            PrefOutput::SetConsoleMode(on) => {
                if pref!(self.console_mode = on) {
                    self.notifications.emit(NotifyInput::SetConsoleEnabled(on));
                    self.rebuild_help_menu();
                }
            }

            _ => unreachable!("not routed to on_prefs_general_pref"),
        }
    }
}
