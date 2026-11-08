//! The former Tegami Mail address book (`local_contacts.db`), read only.
//!
//! The book is gone (docs/DESIGN.md, "Organize mail and services"): Tegami
//! Mail reads the books GNOME provides. A book that still holds contacts is
//! left on disk untouched, and the app offers to export it as one vCard
//! file, to open in GNOME Contacts (`app::legacy_book`). Nothing here
//! writes to it.

use std::path::{Path, PathBuf};

use rusqlite::{Connection, OpenFlags};

/// The stored cards that are groups, by their text: an Evolution list, or a
/// vCard 4 or Apple group. LIKE ignores ASCII case.
const GROUP_WHERE: &str = "vcard LIKE '%X-EVOLUTION-LIST:TRUE%' OR vcard LIKE '%KIND:group%'";

/// Where the book lived, in the application data dir.
fn db_path() -> Option<PathBuf> {
    crate::config::data_base().map(|d| d.join("tegami").join("local_contacts.db"))
}

/// The book's database opened for reading only, or `None` while there is
/// none, or one with no schema. Reading never writes the file.
fn open_read_at(path: &Path) -> Result<Option<Connection>, String> {
    if !path.is_file() {
        return Ok(None);
    }
    let flags = OpenFlags::SQLITE_OPEN_READ_ONLY
        | OpenFlags::SQLITE_OPEN_NO_MUTEX
        | OpenFlags::SQLITE_OPEN_URI;
    let conn =
        Connection::open_with_flags(path, flags).map_err(|e| format!("{}: {e}", path.display()))?;
    let version: i64 = conn
        .query_row("PRAGMA user_version", [], |r| r.get(0))
        .map_err(|e| format!("{}: {e}", path.display()))?;
    Ok((version > 0).then_some(conn))
}

/// How many people the book at `path` holds: its contacts less its groups.
fn people_at(path: &Path) -> usize {
    let Ok(Some(conn)) = open_read_at(path) else {
        return 0;
    };
    let sql = format!("SELECT count(*) FROM contact WHERE NOT ({GROUP_WHERE})");
    conn.query_row(&sql, [], |r| r.get::<_, i64>(0))
        .map_or(0, |n| n as usize)
}

/// Every card of the book at `path`, groups included, sorted by name.
fn vcards_at(path: &Path) -> Result<Vec<String>, String> {
    let Some(conn) = open_read_at(path)? else {
        return Ok(Vec::new());
    };
    let mut stmt = conn
        .prepare("SELECT vcard FROM contact ORDER BY name COLLATE NOCASE, uid")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |r| r.get::<_, String>(0))
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<_, _>>().map_err(|e| e.to_string())
}

/// How many people the former book holds (0 when there is none).
pub(crate) fn count() -> usize {
    db_path().map_or(0, |p| people_at(&p))
}

/// Write every card of the former book into one `.vcf` file at `dest`,
/// returning how many.
pub(crate) fn export_to(dest: &Path) -> Result<usize, String> {
    let src = db_path().ok_or("no data directory")?;
    export_from(&src, dest)
}

/// Rename the former book once exported, so the offer stops; the file
/// stays, should the export be wanted again.
pub(crate) fn set_aside() {
    if let Some(db) = db_path().filter(|p| p.is_file()) {
        let _ = std::fs::rename(&db, db.with_file_name("local_contacts-exported.db"));
    }
}

fn export_from(src: &Path, dest: &Path) -> Result<usize, String> {
    let vcards = vcards_at(src)?;
    let mut text = String::new();
    for v in &vcards {
        text.push_str(v.trim_end_matches(['\r', '\n']));
        text.push_str("\r\n");
    }
    crate::config::write_private(dest, &text).map_err(|e| format!("{}: {e}", dest.display()))?;
    Ok(vcards.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A book as the former one stored it: two people and a group.
    fn sample(path: &Path) {
        let conn = Connection::open(path).unwrap();
        conn.execute_batch(
            "CREATE TABLE contact (uid TEXT PRIMARY KEY, vcard TEXT NOT NULL,
                 name TEXT NOT NULL, updated INTEGER NOT NULL);
             PRAGMA user_version = 1;",
        )
        .unwrap();
        for (uid, name, extra) in [
            ("b", "Bob", ""),
            ("a", "Ada", ""),
            ("g", "Team", "X-EVOLUTION-LIST:TRUE\r\n"),
        ] {
            let card = format!("BEGIN:VCARD\r\nVERSION:3.0\r\nFN:{name}\r\n{extra}END:VCARD\r\n");
            conn.execute(
                "INSERT INTO contact VALUES (?1, ?2, ?3, 0)",
                rusqlite::params![uid, card, name],
            )
            .unwrap();
        }
    }

    /// The book is counted without its groups, exported whole and in name
    /// order, and left untouched by both.
    #[test]
    fn the_former_book_is_counted_and_exported_without_being_changed() {
        let dir = std::env::temp_dir().join(format!("tegami-legacy-book-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let db = dir.join("local_contacts.db");
        let _ = std::fs::remove_file(&db);
        assert_eq!(people_at(&db), 0);
        sample(&db);
        let before = std::fs::read(&db).unwrap();
        assert_eq!(people_at(&db), 2);
        let out = dir.join("contacts.vcf");
        assert_eq!(export_from(&db, &out).unwrap(), 3);
        let text = std::fs::read_to_string(&out).unwrap();
        let ada = text.find("FN:Ada").unwrap();
        let bob = text.find("FN:Bob").unwrap();
        assert!(ada < bob, "{text}");
        assert_eq!(std::fs::read(&db).unwrap(), before);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
