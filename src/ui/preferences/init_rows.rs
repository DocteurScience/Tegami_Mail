//! Filling the Privacy rows, and the Composing page's receipt rows, from
//! the saved values when the dialog is built.

use super::rows::no_truncate;
use super::*;

impl Preferences {
    /// Settings → Privacy, and the receipts asked for in Composing.
    pub(super) fn init_privacy_rows(&self, widgets: &PreferencesWidgets, init: &PrefInit) {
        let p = &widgets.privacy;
        p.auto_remote_content_row
            .set_active(init.auto_remote_content);
        p.remote_ahead_row.set_active(init.remote_ahead);
        p.show_remote_banner_row.set_active(init.show_remote_banner);
        p.show_spoof_banner_row.set_active(init.show_spoof_banner);
        no_truncate(&p.receipts_row);
        p.receipts_row
            .set_selected(super::choices::receipts_index(init.receipts));
        p.gravatar_row.set_active(init.gravatar);
        p.sender_logos_row.set_active(init.sender_logos);
        let c = &widgets.composing;
        c.request_receipts_row.set_active(init.request_receipts);
        no_truncate(&c.returned_receipts_row);
        c.returned_receipts_row
            .set_selected(super::choices::returned_receipts_index(
                init.returned_receipts,
            ));
    }
}
