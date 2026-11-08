//! Tests of reading, parsing and editing contacts.

#[test]
fn registry_reply_keeps_sources_and_skips_unrelated_objects() {
    use gtk::glib::{variant::ObjectPath, variant::ToVariant};
    use std::collections::HashMap;
    let path = ObjectPath::try_from("/org/gnome/evolution/dataserver/Source/test").unwrap();
    let data = "[Data Source]\nDisplayName=Test\n[Address Book]\nBackendName=local\n";
    let objects = HashMap::from([
        (
            path.clone(),
            HashMap::from([(
                "org.gnome.evolution.dataserver.Source",
                HashMap::from([("UID", "test".to_variant()), ("Data", data.to_variant())]),
            )]),
        ),
        (ObjectPath::try_from("/unrelated").unwrap(), HashMap::new()),
    ]);
    assert_eq!(
        super::sources_from_reply(&(objects,).to_variant()),
        Some(vec![(path.to_string(), "test".into(), data.into()),])
    );
    assert!(super::sources_from_reply(&"wrong reply type".to_variant()).is_none());
}

use super::{prefer_book, vcard_display_name, vcard_photo, vcard_photo_with, Book};

pub(super) const ONE_PIXEL_PNG: &str =
    "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mNk+A8AAQUBAScY42YAAAAASUVORK5CYII=";

// iCloud's CardDAV photos: a quoted https URL parameter precedes the real
// file value — the split must not stop at the colon inside the quotes.
#[test]
fn quoted_params_do_not_swallow_the_value() {
    let line = "PHOTO;X-EVOLUTION-WEBDAV-IMG-URL=\"https://gateway.icloud.com/x/y\";\
                VALUE=uri:file:///tmp/photo.jpg";
    let (prop, value) = super::split_vcard_line(line).unwrap();
    assert!(prop.ends_with("VALUE=uri"));
    assert_eq!(value, "file:///tmp/photo.jpg");
}

#[test]
fn fn_preserves_case() {
    let v = "BEGIN:VCARD\r\nVERSION:3.0\r\nN:Arnwine;Aaron;;;\r\nFN:Aaron Arnwine\r\nEND:VCARD\r\n";
    assert_eq!(vcard_display_name(v).as_deref(), Some("Aaron Arnwine"));
}

#[test]
fn fn_with_parameters() {
    let v = "FN;PID=1.1:Jane O'Brien\r\n";
    assert_eq!(vcard_display_name(v).as_deref(), Some("Jane O'Brien"));
}

#[test]
fn fn_line_folded() {
    // A folded FN value continues on the next line after a leading space.
    let v = "FN:Reallylongfirst\r\n Lastname\r\n";
    assert_eq!(
        vcard_display_name(v).as_deref(),
        Some("ReallylongfirstLastname")
    );
}

#[test]
fn fn_escaped_comma() {
    let v = "FN:Smith\\, John\r\n";
    assert_eq!(vcard_display_name(v).as_deref(), Some("Smith, John"));
}

#[test]
fn empty_or_missing_fn() {
    assert_eq!(vcard_display_name("FN:\r\nN:x;;;;\r\n"), None);
    assert_eq!(vcard_display_name("N:Doe;John;;;\r\n"), None);
}

#[test]
fn photo_decodes_nextcloud_data_uri() {
    let vcard = format!("item1.PHOTO:data:image/image/png\\;base64\\,{ONE_PIXEL_PNG}\r\n");
    let photo = vcard_photo(&vcard).unwrap();
    assert_eq!(&photo[..8], b"\x89PNG\r\n\x1a\n");
}

#[test]
fn photo_decodes_folded_encoding_b() {
    let split = ONE_PIXEL_PNG.len() / 2;
    let vcard = format!(
        "PHOTO;ENCODING=b;TYPE=PNG:{}\r\n {}\r\n",
        &ONE_PIXEL_PNG[..split],
        &ONE_PIXEL_PNG[split..]
    );
    assert!(vcard_photo(&vcard).is_some());
}

#[test]
fn photo_does_not_fetch_remote_uri() {
    assert!(vcard_photo("PHOTO;VALUE=uri:https://example.com/tracker.png\r\n").is_none());
}

/// A scratch directory tree for exercising [`super::confine_to_roots`]:
/// `<tmp>/vireo-confine-<pid>-<n>/{root,outside}` with one file in each,
/// removed on drop.
struct ConfineFixture {
    base: std::path::PathBuf,
}

impl ConfineFixture {
    fn new(n: u32) -> Self {
        let base = std::env::temp_dir().join(format!("tegami-confine-{}-{n}", std::process::id()));
        std::fs::create_dir_all(base.join("root")).unwrap();
        std::fs::create_dir_all(base.join("outside")).unwrap();
        std::fs::write(base.join("root/photo.png"), b"in").unwrap();
        std::fs::write(base.join("outside/secret"), b"out").unwrap();
        Self { base }
    }

    fn root(&self) -> std::path::PathBuf {
        self.base.join("root")
    }
}

impl Drop for ConfineFixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.base);
    }
}

#[test]
fn photo_files_inside_a_root_are_admitted() {
    let fx = ConfineFixture::new(1);
    let roots = vec![fx.root()];
    let admitted = super::confine_to_roots(&fx.root().join("photo.png"), &roots);
    assert!(admitted.is_some());
}

#[test]
fn dotdot_traversal_cannot_leave_a_root() {
    let fx = ConfineFixture::new(2);
    let roots = vec![fx.root()];
    let escape = fx.root().join("../outside/secret");
    assert_eq!(super::confine_to_roots(&escape, &roots), None);
}

#[test]
fn a_symlink_planted_inside_a_root_cannot_reach_outside() {
    let fx = ConfineFixture::new(3);
    let roots = vec![fx.root()];
    let link = fx.root().join("link.png");
    std::os::unix::fs::symlink(fx.base.join("outside/secret"), &link).unwrap();
    assert_eq!(super::confine_to_roots(&link, &roots), None);
}

#[test]
fn a_lookalike_sibling_directory_is_not_a_root() {
    let fx = ConfineFixture::new(4);
    let sibling = fx.base.join("root-evil");
    std::fs::create_dir_all(&sibling).unwrap();
    std::fs::write(sibling.join("photo.png"), b"no").unwrap();
    let roots = vec![fx.root()];
    assert_eq!(
        super::confine_to_roots(&sibling.join("photo.png"), &roots),
        None
    );
}

#[test]
fn no_roots_admits_nothing() {
    let fx = ConfineFixture::new(5);
    assert_eq!(
        super::confine_to_roots(&fx.root().join("photo.png"), &[]),
        None
    );
    // A root that doesn't resolve is the same as no root at all.
    let ghost = vec![fx.base.join("missing")];
    assert_eq!(
        super::confine_to_roots(&fx.root().join("photo.png"), &ghost),
        None
    );
}

#[test]
fn a_host_bearing_file_uri_is_refused() {
    assert_eq!(
        super::read_eds_photo_file("file://evil-host/etc/passwd", 1024),
        None
    );
}

/// A birthday is written as mail dates are, so the words and the order
/// are the locale's; a vCard value that is no date passes through.
#[test]
fn birthdays_are_written_as_dates() {
    let full = super::pretty_birthday("1985-04-12");
    assert!(full.contains("1985") && full.contains("12"), "{full}");
    assert_eq!(super::pretty_birthday("19850412"), full);
    let leap = super::pretty_birthday("--02-29");
    assert!(leap.contains("29") && !leap.contains("2000"), "{leap}");
    assert_eq!(super::pretty_birthday("1985-02-30"), "1985-02-30");
    assert_eq!(super::pretty_birthday("sometime"), "sometime");
}

/// EDS `.source` files are key files: a book's name comes from its
/// account collection, and only `[Data Source]`'s Enabled switches it off.
#[test]
fn eds_sources_are_read_as_key_files() {
    let book = "[Data Source]\nDisplayName=Contacts\nEnabled=true\nParent=acct\n\n\
                [Address Book]\nBackendName=carddav\n\n[Refresh]\nEnabled=false\n";
    let acct = "[Data Source]\nDisplayName=Google\n\n[Collection]\nBackendName=google\n\
                Identity=someone@gmail.com\n";
    let sources: std::collections::HashMap<String, String> =
        [("acct".to_string(), acct.to_string())].into();
    assert_eq!(
        super::book_display_name(book, &sources),
        "Google — someone@gmail.com · Contacts"
    );
    assert!(
        !super::data_source_disabled(book),
        "[Refresh] Enabled=false is not the switch"
    );
    // Nextcloud: several books under one account get distinct names…
    let nc = "[Data Source]\nDisplayName=alice@cloud.example\n\n[Collection]\nBackendName=webdav\n\
              Identity=alice\n";
    let sources: std::collections::HashMap<String, String> =
        [("acct".to_string(), nc.to_string())].into();
    let recent = book.replace("DisplayName=Contacts", "DisplayName=Recently contacted");
    assert_eq!(
        super::book_display_name(book, &sources),
        "CardDAV — alice · Contacts"
    );
    assert_eq!(
        super::book_display_name(&recent, &sources),
        "CardDAV — alice · Recently contacted"
    );
    // …while a standalone book (no parent) isn't named twice.
    let lone = "[Data Source]\nDisplayName=Work\n\n[Address Book]\nBackendName=carddav\n";
    assert_eq!(super::book_display_name(lone, &sources), "CardDAV — Work");
    assert!(super::data_source_disabled(
        &book.replace("Enabled=true", "Enabled=false")
    ));
    assert_eq!(
        crate::desktop::platform::keyfile_value(book, "Address Book", "Missing"),
        None
    );
}

#[test]
fn books_that_fill_themselves_from_mail_are_spotted() {
    assert!(super::is_auto_collected("contact-collected", ""));
    assert!(super::is_auto_collected("x1", "Collected Addresses"));
    assert!(super::is_auto_collected(
        "x2",
        "CardDAV \u{2014} alice · Recently contacted"
    ));
    assert!(super::is_auto_collected(
        "x3",
        "Google \u{2014} me@gmail.com · Other contacts"
    ));
    assert!(!super::is_auto_collected(
        "system-address-book",
        "On This Computer"
    ));
    assert!(!super::is_auto_collected(
        "x4",
        "CardDAV \u{2014} alice · Contacts"
    ));
}

#[test]
fn the_chosen_book_comes_first() {
    let book = |uid: &str| Book {
        uid: uid.into(),
        name: uid.into(),
        groups: None,
    };
    let mut books = vec![book("a"), book("b"), book("c")];
    prefer_book(&mut books, "c");
    assert_eq!(
        books.iter().map(|b| b.uid.as_str()).collect::<Vec<_>>(),
        ["c", "a", "b"]
    );
    prefer_book(&mut books, "gone");
    prefer_book(&mut books, "");
    assert_eq!(books[0].uid, "c");
}

#[test]
fn local_vcards_never_read_file_photos() {
    let v = "BEGIN:VCARD\r\nFN:Ann\r\nPHOTO;VALUE=uri:file:///etc/hostname\r\nEND:VCARD\r\n";
    assert!(vcard_photo_with(v, false).is_none());
}

#[test]
fn an_account_named_in_tegami_names_its_books_account() {
    let work = "Work".to_string();
    assert_eq!(
        super::with_account_name("CardDAV — a@b.org · card", Some(&work)),
        "Work · card"
    );
    assert_eq!(
        super::with_account_name("CardDAV — a@b.org", Some(&work)),
        "Work · a@b.org"
    );
    assert_eq!(
        super::with_account_name("On This Computer · Personal", Some(&work)),
        "Work · Personal"
    );
    assert_eq!(
        super::with_account_name("CardDAV — a@b.org · card", None),
        "CardDAV — a@b.org · card"
    );
    // And a book named too.
    let family = "Family".to_string();
    assert_eq!(
        super::with_given_name(
            &super::with_account_name("CardDAV — a@b.org · card", Some(&work)),
            Some(&family)
        ),
        "Work · Family"
    );
}

#[test]
fn a_book_named_in_tegami_keeps_its_account() {
    let given = "Family".to_string();
    assert_eq!(
        super::with_given_name("CardDAV — a@b.org · card", Some(&given)),
        "CardDAV — a@b.org · Family"
    );
    assert_eq!(
        super::with_given_name("CardDAV — Work", Some(&given)),
        "CardDAV — Work · Family"
    );
    assert_eq!(
        super::with_given_name("Tegami Mail Address Book", Some(&given)),
        "Family"
    );
    assert_eq!(
        super::with_given_name("CardDAV — a@b.org · card", None),
        "CardDAV — a@b.org · card"
    );
}
