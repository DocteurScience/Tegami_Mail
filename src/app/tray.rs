//! The unread counts the sidebar, Background Apps and the launcher badge
//! show.

use relm4::prelude::*;

use crate::ui::sidebar::SidebarInput;

use super::model::AppModel;
use super::msg::AppMsg;

impl AppModel {
    /// Update the sidebar's unread badges in place (no rebuild), derived from the
    /// loaded message lists. Cheap enough to call on every read/sync.
    pub(super) fn push_unread_counts(&self) {
        let folders = self.folder_unread.clone();
        let unified = self.unified_unread();
        self.sidebars_emit(SidebarInput::SetUnread { folders });
        // The counted total is what GNOME shows beside Tegami Mail in Background
        // Apps, so a process with no window still says what it is there for.
        if self.run_in_background.get() {
            crate::desktop::background::set_status(&crate::desktop::background::status_text(
                unified,
            ));
        }
        self.push_launcher_count();
    }

    /// The launcher icon's badge (#271): every unread message in the
    /// inboxes, the count other mail apps show there.
    pub(super) fn push_launcher_count(&self) {
        crate::desktop::launcher_badge::set_count(self.inboxes_unread());
    }

    /// Send the launcher badge again even though the count is unchanged, for
    /// when the dock may have dropped it (a lock, a wake).
    pub(super) fn resend_launcher_count(&self) {
        crate::desktop::launcher_badge::resend();
        self.push_launcher_count();
    }

    /// The messages the router sends here (see `AppModel::route`).
    pub(super) fn on_tray(&mut self, msg: AppMsg, _sender: ComponentSender<Self>) {
        match msg {
            AppMsg::PresentWindow => {
                crate::desktop::session::present_window(&self.window);
            }
            _ => unreachable!("not routed to on_tray"),
        }
    }
}
