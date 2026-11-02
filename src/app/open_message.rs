//! Opening one message by where it lives: its account, folder and UID, as a
//! `tegami:` link or a notification names it.

use relm4::prelude::*;

use crate::ui::message_list::MessageListInput;

use super::model::AppModel;

impl AppModel {
    /// Select the message's folder and the message in it.
    pub(super) fn open_message_at(&mut self, account_id: u32, folder_path: &str, uid: u32) {
        self.showing_contacts = false;
        self.showing_outbox = false;
        if let Some(folder) = self.folder_by_path(account_id, folder_path).cloned() {
            self.select_folder(
                account_id,
                folder.id,
                folder.name.clone(),
                folder.path.clone(),
            );
            // Messages use their UID as id, so select by (account, uid).
            self.message_list
                .emit(MessageListInput::SelectAndLoad((account_id, uid)));
        }
    }

    /// The contacts view is the content stack's second page. It brings its
    /// own per-pane headers, so its divider runs to the very top, like the
    /// mail panes'.
    pub(super) fn init_contacts_page(&self, widgets: &super::AppModelWidgets) {
        widgets
            .content_stack
            .add_named(self.contacts_page.widget(), Some("contacts"));
    }
}
