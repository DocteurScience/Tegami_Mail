//! The address books and contacts of the offline demo.

use super::details::GroupFormat;
use super::read::Book;

/// The books demo mode offers new contacts.
pub(crate) fn demo_books() -> Vec<Book> {
    vec![
        Book {
            uid: "On This Computer".into(),
            name: "On This Computer".into(),
            groups: Some(GroupFormat::Evolution),
        },
        Book {
            uid: "CardDAV — jason@example.org".into(),
            name: "CardDAV — jason@example.org".into(),
            groups: Some(GroupFormat::Apple),
        },
    ]
}
