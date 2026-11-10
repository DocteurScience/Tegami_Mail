//! Per-account colour classes for the GTK chrome around the page.

use super::MessageView;
use crate::i18n::{i18n, ni18n_f};

use adw::prelude::*;
use relm4::prelude::*;

/// Style rules for a [`DisplayCss`](crate::ui::DisplayCss), each named by a
/// class. A message's own colors used to be the one rule a provider held, so
/// every message opened reloaded it and restyled the whole window (#323);
/// kept side by side, they leave a class change on one widget to move
/// between them.
pub(super) struct ClassRules {
    css: crate::ui::DisplayCss,
    rules: std::cell::RefCell<std::collections::BTreeMap<String, String>>,
}

impl ClassRules {
    pub(super) fn new() -> Self {
        ClassRules {
            css: crate::ui::DisplayCss::new(),
            rules: Default::default(),
        }
    }

    /// Add or replace rules, as (selector, declarations).
    pub(super) fn set(&self, new: Vec<(String, String)>) {
        let mut rules = self.rules.borrow_mut();
        rules.extend(new);
        self.css.load(
            rules
                .iter()
                .map(|(s, d)| format!("{s} {{ {d} }}\n"))
                .collect(),
        );
    }
}

/// Give `widget` the class `class` in place of any other starting with
/// `prefix`. Nothing is touched when it already wears it.
pub(super) fn wear_class(widget: &impl IsA<gtk::Widget>, prefix: &str, class: &str) {
    if widget.has_css_class(class) {
        return;
    }
    for c in widget.css_classes() {
        if c.starts_with(prefix) {
            widget.remove_css_class(&c);
        }
    }
    widget.add_css_class(class);
}

impl MessageView {
    /// What the empty pane is titled: how many rows are checked, while
    /// Gmail's x has some (#392).
    pub(super) fn empty_title(&self) -> String {
        let n = self.checked.0;
        if n > 0 {
            let count = n.to_string();
            if self.checked.1 {
                ni18n_f(
                    "{n} conversation selected",
                    "{n} conversations selected",
                    n as u32,
                    &[("n", &count)],
                )
            } else {
                ni18n_f(
                    "{n} message selected",
                    "{n} messages selected",
                    n as u32,
                    &[("n", &count)],
                )
            }
        } else if self.drafts_view {
            i18n("No draft selected")
        } else {
            i18n("No message selected")
        }
    }

    /// Whether the GTK header carries tag chips, under the subject (mockups
    /// 07 and 08): a conversation's, gathered from all its messages, or a
    /// lone message's shown full-bleed. A lone message drawn as a card
    /// carries its own in the document.
    pub(super) fn header_tags_shown(&self) -> bool {
        self.thread.len() > 1 && {
            let (keywords, labels) = self.header_tag_sets();
            !crate::models::pills(&keywords, &labels, &self.tags).is_empty()
        }
    }

    /// The keywords and labels the header's chips are drawn from.
    fn header_tag_sets(&self) -> (Vec<String>, Vec<String>) {
        if self.thread.len() > 1 {
            thread_tag_sets(&self.thread)
        } else {
            self.current
                .as_ref()
                .map(|c| (c.keywords.clone(), c.labels.clone()))
                .unwrap_or_default()
        }
    }

    /// Rebuild the GTK header's chips when the keywords or labels they show
    /// (or the tags) changed since they were last built.
    pub(super) fn sync_header_tags(&self, header_tags: &gtk::Box) {
        let shown = self.header_tag_sets();
        if self.header_tags_rendered.borrow().as_ref() == Some(&shown) {
            return;
        }
        while let Some(child) = header_tags.first_child() {
            header_tags.remove(&child);
        }
        for pill in crate::models::pills(&shown.0, &shown.1, &self.tags) {
            let chip = crate::ui::message_row::pill_chip(&pill, 20);
            header_tags.append(&chip);
        }
        *self.header_tags_rendered.borrow_mut() = Some(shown);
    }
}

/// Every keyword and label a conversation's messages carry, each once, in
/// the order they first appear: the conversation's tags, as the list's row
/// for it shows them.
pub(super) fn thread_tag_sets(thread: &[crate::models::Message]) -> (Vec<String>, Vec<String>) {
    let mut keywords: Vec<String> = Vec::new();
    let mut labels: Vec<String> = Vec::new();
    for m in thread {
        for k in &m.keywords {
            if !keywords.iter().any(|x| x.eq_ignore_ascii_case(k)) {
                keywords.push(k.clone());
            }
        }
        for l in &m.labels {
            if !labels.contains(l) {
                labels.push(l.clone());
            }
        }
    }
    (keywords, labels)
}

#[cfg(test)]
mod thread_tag_tests {
    use super::thread_tag_sets;
    use crate::models::test_util::msg;

    #[test]
    fn a_conversations_tags_are_gathered_once_in_order() {
        let mut a = msg(1, 1, 10);
        a.keywords = vec!["work".into(), "$label1".into()];
        let mut b = msg(1, 1, 11);
        b.keywords = vec!["Work".into(), "todo".into()];
        b.labels = vec!["Receipts".into()];
        let mut c = msg(1, 1, 12);
        c.labels = vec!["Receipts".into(), "Travel".into()];
        let (keywords, labels) = thread_tag_sets(&[a, b, c]);
        assert_eq!(keywords, ["work", "$label1", "todo"]);
        assert_eq!(labels, ["Receipts", "Travel"]);
    }
}
