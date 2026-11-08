//! What the settings window asks of the panel: closing, leaving and saving
//! the page that is open.

use super::*;

impl AccountsWindow {
    pub(super) fn on_close_editor(&mut self, widgets: &AccountsWindowWidgets) {
        if widgets
            .nav
            .visible_page()
            .and_then(|p| p.tag())
            .is_some_and(|t| t == "editor" || t == "filter")
        {
            widgets.nav.pop();
        }
    }
}
