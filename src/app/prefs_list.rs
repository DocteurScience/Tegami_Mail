//! Applying message list settings.

use relm4::prelude::*;

use crate::ui::message_list::MessageListInput;
use crate::ui::preferences::PrefOutput;

use super::model::AppModel;
use super::msg::AppMsg;

impl AppModel {
    /// The settings changes the router sends here (see `AppModel::route`).
    pub(super) fn on_prefs_list_pref(&mut self, pref: PrefOutput, sender: ComponentSender<Self>) {
        match pref {
            PrefOutput::SetContactNames(on) => {
                if pref!(self.contact_names = on) {
                    self.read_contact_names(&sender);
                }
            }
            PrefOutput::SetPreviewLines(lines) => {
                if self.preview_lines != lines {
                    let was_off = self.preview_lines == 0;
                    self.preview_lines = lines;
                    self.save_settings();
                    self.push_list_look();
                    // Previews switched back on: IMAP summaries fetched while
                    // they were off carry no preview text (the setting also
                    // stops the body slice being downloaded), so nothing would
                    // show until the next scheduled sync. Refresh now so they
                    // fill in right away. (Graph always has bodyPreview in
                    // hand, which is why Microsoft accounts showed instantly.)
                    if was_off && lines > 0 {
                        sender.input(AppMsg::Refresh);
                    }
                }
            }

            PrefOutput::SetSwipeEnabled(on) => {
                if pref!(self.swipe_enabled = on) {
                    self.message_list
                        .emit(MessageListInput::SetSwipeEnabled(on));
                }
            }

            _ => unreachable!("not routed to on_prefs_list_pref"),
        }
    }
}
