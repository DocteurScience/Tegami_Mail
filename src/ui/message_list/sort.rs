//! The list's orders: comparing two messages by the chosen sort.

use crate::models::Message;

/// Order two messages by the chosen sort (ties fall back to date).
pub(super) fn message_cmp(a: &Message, b: &Message, order: SortOrder) -> std::cmp::Ordering {
    let name_of = |m: &Message| {
        crate::contacts::shown_name_or_addr(&m.from_name, &m.from_addr).to_lowercase()
    };
    match order {
        SortOrder::DateNewest => b.timestamp.cmp(&a.timestamp),
        SortOrder::DateOldest => a.timestamp.cmp(&b.timestamp),
        SortOrder::Sender => name_of(a)
            .cmp(&name_of(b))
            .then(b.timestamp.cmp(&a.timestamp)),
        SortOrder::Subject => normalize_subject(&a.subject)
            .cmp(&normalize_subject(&b.subject))
            .then(b.timestamp.cmp(&a.timestamp)),
    }
}

/// Lower-case the subject and strip any leading run of reply/forward prefixes
/// (`Re:`, `Fwd:`, …) so subject-sorting keeps a topic and its replies adjacent.
pub(super) fn normalize_subject(subject: &str) -> String {
    const PREFIXES: &[&str] = &["re:", "fwd:", "fw:", "aw:", "sv:", "antw:", "wg:"];
    let mut s = subject.trim();
    loop {
        let lower = s.to_ascii_lowercase();
        let mut stripped = false;
        for p in PREFIXES {
            if lower.starts_with(p) {
                s = s[p.len()..].trim_start();
                stripped = true;
                break;
            }
        }
        if !stripped {
            break;
        }
    }
    s.trim().to_ascii_lowercase()
}

/// How the message list is ordered.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SortOrder {
    DateNewest,
    DateOldest,
    Sender,
    Subject,
}

impl SortOrder {
    /// "unread" and "flagged" (Unread first, Starred first) were removed
    /// on 2026-11-07; the list's order is not saved, so nothing carries
    /// them, and an unknown key reads as newest first.
    pub(super) const KEYS: [(SortOrder, &'static str); 4] = [
        (SortOrder::DateNewest, "date_newest"),
        (SortOrder::DateOldest, "date_oldest"),
        (SortOrder::Sender, "sender"),
        (SortOrder::Subject, "subject"),
    ];

    pub(crate) fn from_key(key: &str) -> Self {
        SortOrder::KEYS
            .iter()
            .find(|(_, k)| *k == key)
            .map(|(o, _)| *o)
            .unwrap_or(SortOrder::DateNewest)
    }
}
