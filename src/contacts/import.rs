//! Importing `.vcf` files into the Tegami Mail book. Editing contacts and
//! groups went with the Contacts page (owner's decision, docs/SUIVI.md).

use std::path::PathBuf;

use super::photos::local_changed;

/// Import `.vcf` files into the local Tegami Mail book (blocking: call from a
/// background thread).
pub(crate) fn import_vcf_files(
    paths: &[PathBuf],
) -> Result<crate::contacts::local::ImportOutcome, String> {
    let outcome = crate::contacts::local::import_files(paths)?;
    if outcome.added + outcome.updated > 0 {
        local_changed();
    }
    Ok(outcome)
}
