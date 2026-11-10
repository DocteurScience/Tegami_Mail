//! Applying reader settings: zoom, font and color overrides.

use relm4::prelude::*;

use crate::ui::message_list::MessageListInput;
use crate::ui::message_view::MessageViewInput;
use crate::ui::message_window::MessageWindowInput;
use crate::ui::preferences::PrefOutput;
use crate::worker::MailRequest;

use super::model::AppModel;
use super::msg::AppMsg;

impl AppModel {
    /// The session's message zoom: every reader on screen follows.
    pub(super) fn set_zoom(&mut self, zoom: u32) {
        if zoom == self.zoom {
            return;
        }
        self.zoom = zoom;
        self.message_view.emit(MessageViewInput::SetZoom(zoom));
        self.popouts_emit(|| MessageWindowInput::SetZoom(zoom));
    }

    /// The settings changes the router sends here (see `AppModel::route`).
    pub(super) fn on_prefs_reader_pref(&mut self, pref: PrefOutput) {
        match pref {
            PrefOutput::SetShowRemoteBanner(on) => {
                if pref!(self.show_remote_banner = on) {
                    self.message_view.emit(MessageViewInput::SetBannerShown(on));
                }
            }

            PrefOutput::SetReceipts(policy) => {
                if pref!(self.receipts = policy) {
                    self.refresh_receipt_banners();
                }
            }

            PrefOutput::SetAutoRemoteContent(on) => {
                if pref!(self.auto_remote_content = on) {
                    // Re-render what is open so the change takes effect there too:
                    // on, the blocked content loads; off, it is stripped again.
                    if self.current_thread.len() > 1 {
                        self.show_thread();
                    } else {
                        let current = self.current.clone();
                        self.show_message(current, false);
                    }
                }
            }

            PrefOutput::SetThreading(on) => {
                if pref!(self.threading = on) {
                    self.message_list.emit(MessageListInput::SetThreading(on));
                }
                self.sync_show_checks();
            }

            PrefOutput::SetConfirmThreadDelete(on) => {
                pref!(self.confirm_thread_delete = on);
            }

            PrefOutput::SetThreadNewestFirst(on) => {
                if pref!(self.thread_newest_first = on) {
                    // Re-render an open conversation in the new order.
                    if self.current_thread.len() > 1 {
                        self.show_thread();
                    }
                }
            }

            _ => unreachable!("not routed to on_prefs_reader_pref"),
        }
    }

    /// The messages the router sends here (see `AppModel::route`).
    pub(super) fn on_prefs_reader(&mut self, msg: AppMsg) {
        match msg {
            AppMsg::SetRemoteContent {
                account_id,
                id,
                show,
            } => {
                self.remote_override.insert((account_id, id), show);
                // Re-show whatever is open so the decision takes effect now.
                // The reader re-reads `remote_allowed` for every message it
                // paints, so a conversation gets it for the card it applies to.
                if self.current_thread.len() > 1 {
                    self.show_thread();
                } else {
                    let current = self.current.clone();
                    self.show_message(current, false);
                }
            }
            AppMsg::ReloadBody(m) => {
                if let Some(path) = self.resolve_folder_path(&m) {
                    self.send_to(
                        m.account_id,
                        MailRequest::LoadBody {
                            message_id: m.id,
                            path,
                            uid: m.uid,
                        },
                    );
                }
            }
            _ => unreachable!("not routed to on_prefs_reader"),
        }
    }
}
