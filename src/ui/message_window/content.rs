//! What the pop-out shows: the conversation in its reader, the attachments
//! in its drawer, and the actions taken on a member.

use super::types::MessageWindowOutput;
use super::MessageWindow;
use crate::models::{Attachment, Message};
use crate::ui::message_list::RowAction;
use crate::ui::message_view::MessageViewInput;
use relm4::prelude::*;

impl MessageWindow {
    pub(super) fn emit_action(&self, action: RowAction, sender: &ComponentSender<Self>) {
        let _ = sender.output(MessageWindowOutput::Action {
            action,
            message: Box::new(self.msg.clone()),
        });
    }

    pub(super) fn render_body(&self) {
        self.view.emit(MessageViewInput::Show {
            thread: self.thread.clone(),
            allow_remote: self.allow_remote,
            account_name: self.account_name.clone(),
            primary: None,
            folder_labels: std::collections::HashMap::new(),
            sent: Default::default(),
            loading: self.loading,
            // The popout renders what it was handed; no staged reveal.
            instant: true,
        });
    }

    /// A member of the window's conversation (or the lone message).
    pub(super) fn member(&self, account_id: u32, id: u32) -> Option<&Message> {
        self.thread
            .iter()
            .find(|m| m.account_id == account_id && m.id == id)
    }

    /// The message a file of the drawer came with: the first carrying one
    /// of that name and size, since the drawer shows such a pair once.
    pub(super) fn owner(&self, att: &Attachment) -> Option<&Message> {
        let (key, _) = self.attachments.iter().find(|(_, files)| {
            files
                .iter()
                .any(|a| a.name == att.name && a.data.len() == att.data.len())
        })?;
        self.member(key.0, key.1)
    }

    /// The files on each card (#213).
    pub(super) fn sync_attachments(&self) {
        let rows = card_rows(&self.thread, &self.attachments);
        self.view.emit(MessageViewInput::SetCardAttachments(rows));
        // The previews the cards still lack, made off the main thread.
        let wanted: Vec<_> = self
            .attachments
            .iter()
            .flat_map(|(key, files)| {
                files
                    .iter()
                    .enumerate()
                    .filter(|(_, a)| {
                        crate::ui::message_view::cached_card_preview(&a.data).is_none()
                    })
                    .map(|(index, a)| (*key, index, a.name.clone(), a.data.clone()))
            })
            .collect();
        crate::ui::message_view::request_card_previews(wanted, self.view.sender());
        self.fit_min_width();
    }
}

/// The window/title text for a message (its subject, or a placeholder).
pub(super) fn title_text(m: &Message) -> String {
    if m.subject.trim().is_empty() {
        "(No subject)".to_string()
    } else {
        m.subject.clone()
    }
}

/// Each card's own attachment row, by message key.
type CardRows = std::collections::HashMap<(u32, u32), Vec<crate::ui::message_view::CardAttachment>>;

/// Each card's row of files (#213), from each member's files; a member
/// without files gets no row.
fn card_rows(thread: &[Message], attachments: &[((u32, u32), Vec<Attachment>)]) -> CardRows {
    let mut rows = std::collections::HashMap::new();
    for m in thread {
        let key = (m.account_id, m.id);
        let Some((_, files)) = attachments.iter().find(|(k, _)| *k == key) else {
            continue;
        };
        if !files.is_empty() {
            let row = files
                .iter()
                .map(|a| crate::ui::message_view::CardAttachment {
                    name: a.name.clone(),
                    size: a.data.len() as u64,
                    preview: crate::ui::message_view::cached_card_preview(&a.data),
                })
                .collect::<Vec<_>>();
            rows.insert(key, row);
        }
    }
    rows
}

#[cfg(test)]
mod tests {
    use super::{card_rows, title_text};
    use crate::models::{Attachment, Message};

    fn member(id: u32, subject: &str) -> Message {
        Message {
            id,
            account_id: 1,
            folder_id: 1,
            uid: id,
            from_name: String::new(),
            from_addr: String::new(),
            reply_to: String::new(),
            to: String::new(),
            cc: String::new(),
            bcc: String::new(),
            subject: subject.into(),
            preview: String::new(),
            body: String::new(),
            date: String::new(),
            timestamp: 0,
            unread: false,
            starred: false,
            answered: false,
            forwarded: false,
            keywords: Vec::new(),
            has_attachment: false,
            message_id: String::new(),
            references: String::new(),
            importance: Default::default(),
            due: 0,
            labels: Vec::new(),
        }
    }

    fn file(name: &str, len: usize) -> Attachment {
        Attachment {
            name: name.into(),
            data: vec![0; len],
        }
    }

    #[test]
    fn each_card_lists_its_own_files() {
        let thread = [member(1, ""), member(2, ""), member(3, "")];
        let attachments = vec![
            // Not in thread order: the thread decides the order.
            ((1, 2), vec![file("a.pdf", 3), file("b.png", 5)]),
            ((1, 1), vec![file("a.pdf", 3), file("a.pdf", 4)]),
            ((1, 3), vec![]),
            ((1, 9), vec![file("stranger.txt", 1)]),
        ];
        let rows = card_rows(&thread, &attachments);
        assert_eq!(rows.len(), 2, "no row for a member without files");
        let row = |k| {
            rows[&k]
                .iter()
                .map(|c| (c.name.clone(), c.size))
                .collect::<Vec<_>>()
        };
        assert_eq!(row((1, 1)), [("a.pdf".into(), 3), ("a.pdf".into(), 4)]);
        assert_eq!(row((1, 2)), [("a.pdf".into(), 3), ("b.png".into(), 5)]);
    }

    #[test]
    fn a_window_without_a_subject_says_so() {
        assert_eq!(title_text(&member(1, "Hello")), "Hello");
        // Untranslated, a known quirk.
        assert_eq!(title_text(&member(1, "  ")), "(No subject)");
        assert_eq!(title_text(&member(1, " x ")), " x ");
    }
}
