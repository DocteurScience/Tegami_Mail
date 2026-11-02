//! The forgotten-attachment check (docs/DESIGN.md, "Attachments"): before a
//! message with no file goes out, whether what the person wrote mentions one
//! ("attached", "ci-joint"), so the composer can ask before sending.
//!
//! This module only decides; the composer reads the text, asks and sends.
//! The text is the person's own: the quoted history and the signature are
//! left out by whoever reads it, or a reply to a message that did have an
//! attachment would always ask.

use crate::i18n::i18n;

/// The English words, always looked for: mail is often written in English
/// whatever language the app is in.
const ENGLISH: &[&str] = &["attached", "attachment", "attachments", "enclosed"];

/// The words that mention an attachment: the English ones, and the list of
/// the app's language, which its translator writes as one line of comma
/// separated words, the way Thunderbird's is.
pub(super) fn attachment_words() -> Vec<String> {
    // TRANSLATORS: words that, written in a message, mean a file should be
    // attached ("attached", "enclosed"), comma separated, in lower case.
    // Before a message with no attachment is sent, Tegami Mail asks if one
    // was forgotten when the message contains one of them. Keep them
    // specific: a common word would make it ask too often.
    let local = i18n("attached, attachment, attachments, enclosed");
    let mut words: Vec<String> = ENGLISH.iter().map(|w| (*w).to_string()).collect();
    for w in local.split(',') {
        let w = w.trim().to_lowercase();
        if !w.is_empty() && !words.contains(&w) {
            words.push(w);
        }
    }
    words
}

/// The first of `words` that `text` contains as a whole word or phrase,
/// whatever its case: "attached" in "I attached it", not in "unattached".
/// The word is given back as the list has it, for the dialog to quote.
pub(super) fn mentioned_attachment(text: &str, words: &[String]) -> Option<String> {
    let text = text.to_lowercase();
    words.iter().find_map(|word| {
        if word.is_empty() {
            return None;
        }
        let mut from = 0;
        while let Some(at) = text[from..].find(word.as_str()) {
            let start = from + at;
            let end = start + word.len();
            let before = text[..start].chars().next_back();
            let after = text[end..].chars().next();
            let edge = |c: Option<char>| c.is_none_or(|c| !c.is_alphanumeric());
            if edge(before) && edge(after) {
                return Some(word.clone());
            }
            from = end;
        }
        None
    })
}

#[cfg(test)]
mod tests {
    use super::mentioned_attachment;

    fn words() -> Vec<String> {
        ["attached", "enclosed", "ci-joint", "pièce jointe"]
            .iter()
            .map(|w| (*w).to_string())
            .collect()
    }

    #[test]
    fn a_word_or_phrase_is_found_whatever_its_case() {
        assert_eq!(
            mentioned_attachment("Please find the report Attached.", &words()),
            Some("attached".into())
        );
        assert_eq!(
            mentioned_attachment("Voici la Pièce Jointe promise", &words()),
            Some("pièce jointe".into())
        );
        assert_eq!(
            mentioned_attachment("Je vous envoie ci-joint les maquettes", &words()),
            Some("ci-joint".into())
        );
    }

    #[test]
    fn only_a_whole_word_counts() {
        assert_eq!(mentioned_attachment("an unattached note", &words()), None);
        assert_eq!(mentioned_attachment("attachedness", &words()), None);
        assert_eq!(
            mentioned_attachment("unattached, then attached", &words()),
            Some("attached".into())
        );
    }

    #[test]
    fn nothing_mentioned_is_nothing_found() {
        assert_eq!(mentioned_attachment("See you on Thursday.", &words()), None);
        assert_eq!(mentioned_attachment("", &words()), None);
    }
}
