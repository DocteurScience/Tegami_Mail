//! HTML read as text: a message's words for previews and quotes
//! (`plain_text`), its text part as Markdown-style plain text
//! (`html_to_text`, from `from_html`), and the sanitizer for HTML that goes
//! on the wire. Writing in Markdown was removed (owner's decision,
//! docs/SUIVI.md); the text form it produces still reads well as a mail's
//! plain-text alternative.

mod dom;
mod outgoing;
#[cfg(test)]
mod tests;
mod writer;

pub(crate) use dom::plain_text;
pub(crate) use outgoing::sanitize_outgoing;
pub(crate) use writer::html_to_text;
