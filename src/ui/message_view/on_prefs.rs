//! The reader's preferences, set from the app: each stores the value and
//! renders or patches what it changes.

use super::MessageView;
use relm4::prelude::*;
use webkit6::prelude::WebViewExt;

impl MessageView {
    pub(super) fn on_set_fold_look(&mut self, preview_lines: u32) {
        let preview_lines = preview_lines.min(3);
        if self.fold_preview_lines != preview_lines {
            self.fold_preview_lines = preview_lines;
            self.render();
        }
    }

    pub(super) fn on_set_zoom(&mut self, percent: u32) {
        if self.zoom != percent {
            self.zoom = percent;
            // The frames remeasure at the new size; the heights
            // remembered for the old one would only make them lurch.
            self.frame_heights.clear();
            if self.current.is_some() && !self.loading {
                let js = format!(
                    "if(window.vfZoom)window.vfZoom({});",
                    percent as f64 / 100.0
                );
                self.webview.evaluate_javascript(
                    &js,
                    None,
                    None,
                    None::<&gtk::gio::Cancellable>,
                    |_| {},
                );
            }
        }
    }

    pub(super) fn on_set_unsubscribed(&mut self, lists: std::collections::HashMap<String, i64>) {
        self.unsubscribed = lists;
        let keys: Vec<(u32, u32)> = self.thread.iter().map(|m| (m.account_id, m.id)).collect();
        for (account_id, id) in keys {
            self.patch_unsub(account_id, id);
        }
    }

    pub(super) fn on_set_identities(
        &mut self,
        identities: std::collections::HashMap<u32, Vec<String>>,
    ) {
        if self.identities == identities {
            return;
        }
        self.identities = identities;
        let keys: Vec<(u32, u32)> = self.thread.iter().map(|m| (m.account_id, m.id)).collect();
        for (account_id, id) in keys {
            self.patch_invite(account_id, id);
        }
    }

    pub(super) fn on_set_invite_answers(
        &mut self,
        answers: std::collections::HashMap<String, (String, i64)>,
    ) {
        self.invite_answers = answers;
        let keys: Vec<(u32, u32)> = self.thread.iter().map(|m| (m.account_id, m.id)).collect();
        for (account_id, id) in keys {
            self.patch_invite(account_id, id);
        }
    }

    pub(super) fn on_set_read_mark(&mut self, policy: crate::config::ReadMark) {
        if self.read_mark != policy {
            self.read_mark = policy;
            if self.current.is_some() && !self.loading {
                self.render();
            }
        }
    }

    pub(super) fn on_set_tags(&mut self, tags: Vec<crate::config::Tag>) {
        if self.tags != tags {
            self.tags = tags;
            *self.header_tags_rendered.borrow_mut() = None;
            if !self.thread.is_empty() {
                // The chips are baked into the document: draw it again.
                self.shown_fingerprint = None;
                self.render();
            }
        }
    }

    pub(super) fn on_set_template_folders(
        &mut self,
        folders: std::collections::HashSet<(u32, u32)>,
    ) {
        if self.template_folders != folders {
            let shown = |set: &std::collections::HashSet<(u32, u32)>| {
                self.thread
                    .iter()
                    .any(|m| set.contains(&(m.account_id, m.folder_id)))
            };
            let redraw = shown(&self.template_folders) || shown(&folders);
            self.template_folders = folders;
            if redraw {
                self.render();
            }
        }
    }
}

impl MessageView {}
