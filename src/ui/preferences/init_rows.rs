//! Filling the Composing page's receipt rows from the saved values when
//! the dialog is built (the Privacy rows fill themselves).

use super::rows::no_truncate;
use super::*;

impl Preferences {
    /// The receipts asked for, and what becomes of those that come back.
    pub(super) fn init_privacy_rows(&self, widgets: &PreferencesWidgets, init: &PrefInit) {
        let c = &widgets.composing;
        c.request_receipts_row.set_active(init.request_receipts);
        no_truncate(&c.returned_receipts_row);
        c.returned_receipts_row
            .set_selected(super::choices::returned_receipts_index(
                init.returned_receipts,
            ));
    }
}
