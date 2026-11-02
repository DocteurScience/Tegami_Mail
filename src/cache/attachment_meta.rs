//! What a message's attachments are, recorded without their bytes
//! (`attachment_meta`). The attachments gallery that read it is gone (owner's
//! decision, docs/SUIVI.md); the table is still kept in step with moves and
//! deletions until it is dropped with a schema change.

use rusqlite::params;

use super::Cache;

impl Cache {
    /// Record what a message's attachments are, without their bytes: one row
    /// per attachment plus a scan mark, so a message with none is never asked
    /// about again. Replaces any earlier answer for that message.
    pub(crate) fn save_attachment_meta(
        &self,
        account_id: u32,
        folder_path: &str,
        uid: u32,
        metas: &[crate::models::AttachmentMeta],
    ) {
        let _ = self.conn.execute(
            "DELETE FROM attachment_meta WHERE account_id = ?1 AND folder_path = ?2 AND uid = ?3",
            params![account_id, folder_path, uid],
        );
        for m in metas {
            if let Err(e) = self.conn.execute(
                "INSERT OR REPLACE INTO attachment_meta \
                 (account_id, folder_path, uid, idx, name, mime, size, section, ext, bucket, keywords) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                params![
                    account_id,
                    folder_path,
                    uid,
                    m.idx,
                    &m.name,
                    &m.mime,
                    m.size as i64,
                    &m.section,
                    crate::models::ext_of(&m.name),
                    crate::models::type_bucket(&m.name),
                    crate::models::type_keywords(&m.name),
                ],
            ) {
                tracing::warn!("cache save_attachment_meta failed: {e}");
            }
        }
        let _ = self.conn.execute(
            "INSERT OR IGNORE INTO attachment_scan (account_id, folder_path, uid) VALUES (?1, ?2, ?3)",
            params![account_id, folder_path, uid],
        );
    }
}
