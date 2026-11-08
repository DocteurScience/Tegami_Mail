//! The rows the preferences dialog is assembled from (`dialog`), one
//! builder each: the Composing rows (Messages and its Advanced sub-page),
//! Privacy, Backup, and the rows kept from the former pages.
//!
//! Each builder is a standalone `relm4::view!` or plain code run from
//! `Preferences::init` before the dialog's own view; the rows the dialog
//! reaches later are kept in the builder's struct (`widgets.kept.fetch_row`).

mod backup;
mod composing;
mod kept;
mod privacy;

pub(super) use backup::BackupPage;
pub(super) use composing::ComposingPage;
pub(super) use kept::KeptRows;
pub(super) use privacy::PrivacyPage;
