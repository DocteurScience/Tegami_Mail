//! Folded cards (#326): which cards open folded, and their bars.

use super::html::attr_escape;
use super::snippets::inline_icon_svg;
use super::MessageView;
use crate::encoding::escape_html;
use crate::i18n::i18n;
use crate::models::Message;

impl MessageView {
    /// The cards of the conversation on screen that show folded (#326):
    /// those folded by hand, and the ones Settings folds: the read
    /// messages, or every message, but the newest either way. Opening a conversation makes its first message the
    /// current one, so "the one opened" would have kept the oldest open; a
    /// message stepped to inside the conversation is opened by hand instead
    /// (see `Show`).
    pub(super) fn folded_cards(&self) -> std::collections::HashSet<(u32, u32)> {
        if !self.carded() {
            return Default::default();
        }
        let newest = self
            .thread
            .iter()
            .max_by_key(|m| (m.timestamp, m.id))
            .map(|m| (m.account_id, m.id));
        self.thread
            .iter()
            .map(|m| (m.account_id, m.id, m.unread))
            .filter(|&(aid, id, unread)| match self.folds.get(&(aid, id)) {
                Some(folded) => *folded,
                None => {
                    Some((aid, id)) != newest
                        && match self.fold_messages {
                            crate::config::FoldMessages::Never => false,
                            crate::config::FoldMessages::Read => !unread,
                            crate::config::FoldMessages::AllButNewest => true,
                        }
                }
            })
            .map(|(aid, id, _)| (aid, id))
            .collect()
    }

    /// The marks in a card's header, beside the date, as a folded card
    /// showed them once on a bar of its own (#326): whether it is starred and
    /// whether it carries files. The star is the toolbar's, in its color,
    /// and only a starred card shows it: the card's `starred` class does, so
    /// a star toggled on the card shows or goes in place. Mail you sent says
    /// so in a chip instead (see `card_labels`).
    pub(super) fn card_marks_html(&self, m: &Message) -> String {
        let icon = |class: &str, name: &str, title: &str| {
            format!(
                "<span class=\"tegami-mark{class}\" title=\"{}\">{}</span>",
                attr_escape(title),
                inline_icon_svg(name)
            )
        };
        // An empty slot when there are no files, so the marks of every card
        // stand in the same columns.
        let clip = if m.has_attachment {
            icon("", "mail-attachment-symbolic", &i18n("Has attachments"))
        } else {
            "<span class=\"tegami-mark tegami-mark-none\"></span>".to_string()
        };
        format!(
            "<span class=\"tegami-marks\">{}{clip}</span>",
            icon(
                " tegami-mark-star",
                "tegami-non-starred-symbolic",
                &i18n("Starred")
            ),
        )
    }

    /// The chip beside each card's sender: the folder a message was read
    /// from when it isn't the one on screen, and "Sent" on a card of mail you
    /// sent, which takes the place of its folder's name. Received mail is
    /// the rule, and carries no chip for it.
    pub(super) fn card_labels(&self) -> std::collections::HashMap<(u32, u32), String> {
        let mut labels = self.folder_labels.clone();
        if self.carded() {
            for key in &self.sent {
                labels.insert(*key, i18n("Sent"));
            }
        }
        labels
    }

    /// The first words of a folded card, on its header's one line between
    /// the sender and the date (docs/DESIGN.md, "Reading pane and
    /// conversations"), so a folded card is one line, whatever it holds.
    /// Nothing when the message list shows no preview either (#326). Hidden
    /// while the card is open. A click on the header opens the card.
    pub(super) fn fold_bar_html(&self, m: &Message) -> String {
        if self.fold_preview_lines == 0 {
            return String::new();
        }
        let preview = crate::models::preview_display(&m.preview);
        format!(
            "<span class=\"tegami-fbar\" title=\"{}\">{}</span>",
            attr_escape(&i18n("Click to open this message")),
            escape_html(preview.trim()),
        )
    }

    /// Fold and open the cards of the live document to match
    /// [`MessageView::folded_cards`], without building it again.
    pub(super) fn patch_folds(&self) {
        if !self.webview_ready || !self.carded() {
            return;
        }
        let keys: Vec<String> = self
            .folded_cards()
            .iter()
            .map(|(a, i)| format!("{a}:{i}"))
            .collect();
        let js = format!(
            "(function(k){{var ms=document.querySelectorAll('.tegami-msg');\
             for(var i=0;i<ms.length;i++){{var on=k.indexOf(ms[i].dataset.key)>=0;\
             if(ms[i].classList.contains('tegami-folded')===on)continue;\
             ms[i].classList.toggle('tegami-folded',on);\
             if(!on){{var f=ms[i].querySelector('iframe.tegami-frame');if(f){{f._h=0;s(f);}}}}}}}})({})",
            serde_json::to_string(&keys).unwrap_or_else(|_| "[]".into()),
        );
        self.run_js(&js);
    }
}

impl MessageView {
    /// The line under a conversation's subject: how many messages it holds
    /// and how many people take part in it.
    pub(super) fn conversation_summary(&self) -> String {
        let messages = self.thread.len();
        let people = participant_count(&self.thread);
        let (m, p) = (messages.to_string(), people.to_string());
        // One sentence with both parts, so a language can order them as it
        // needs; each count is its own plural.
        crate::i18n::i18n_f(
            "{messages} · {participants}",
            &[
                (
                    "messages",
                    &crate::i18n::ni18n_f(
                        "{n} message",
                        "{n} messages",
                        messages as u32,
                        &[("n", &m)],
                    ),
                ),
                (
                    "participants",
                    &crate::i18n::ni18n_f(
                        "{n} participant",
                        "{n} participants",
                        people as u32,
                        &[("n", &p)],
                    ),
                ),
            ],
        )
    }
}

/// How many different people a conversation involves: every sender and
/// every To and Cc recipient, by address, whatever the case. Bcc is left
/// out: only the sender's own copy names it, so counting it would give the
/// same conversation two sizes depending on who looks.
fn participant_count(thread: &[Message]) -> usize {
    let mut people = std::collections::HashSet::new();
    for m in thread {
        for addr in std::iter::once(m.from_addr.as_str())
            .chain(m.to.split(','))
            .chain(m.cc.split(','))
        {
            // An entry may be written "Name <addr>"; the address decides.
            let addr = match (addr.rfind('<'), addr.rfind('>')) {
                (Some(open), Some(close)) if open < close => &addr[open + 1..close],
                _ => addr,
            };
            let addr = addr.trim().to_lowercase();
            if !addr.is_empty() {
                people.insert(addr);
            }
        }
    }
    people.len()
}

#[cfg(test)]
mod summary_tests {
    use super::participant_count;
    use crate::models::Message;

    fn msg(from: &str, to: &str, cc: &str) -> Message {
        let mut m = crate::models::test_util::msg(1, 1, 1);
        m.from_addr = from.into();
        m.to = to.into();
        m.cc = cc.into();
        m
    }

    #[test]
    fn senders_and_recipients_count_once_each() {
        let thread = [
            msg(
                "camille@example.org",
                "me@example.org, julien@example.org",
                "",
            ),
            msg(
                "me@example.org",
                "Camille@Example.org",
                "julien@example.org",
            ),
        ];
        assert_eq!(participant_count(&thread), 3);
    }

    #[test]
    fn a_named_address_counts_by_its_address() {
        let thread = [msg(
            "a@example.org",
            "A <a@example.org>, B <b@example.org>",
            "",
        )];
        assert_eq!(participant_count(&thread), 2);
    }

    #[test]
    fn empty_fields_count_nobody() {
        assert_eq!(participant_count(&[msg("a@example.org", "", " , ")]), 1);
    }
}
