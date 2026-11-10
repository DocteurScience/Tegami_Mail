//! Moving through the wizard: forward with its buttons, back with the
//! header bar's, and out.

use super::*;

impl Welcome {
    pub(super) fn on_start(&mut self, widgets: &WelcomeWidgets) {
        widgets.nav.push(&widgets.accounts.page);
    }

    pub(super) fn on_open_form(&mut self, widgets: &WelcomeWidgets) {
        widgets.nav.push(&widgets.form.page);
    }

    /// "Continue": the last page names an account, so with none added the
    /// wizard simply closes, on the main window's empty state.
    pub(super) fn on_continue(&mut self, widgets: &WelcomeWidgets, root: &adw::Window) {
        if self.added.is_empty() {
            root.close();
        } else {
            widgets.done_text.set_text(&i18n_f(
                "Tegami Mail is fetching the mail of {email}. It will appear in a moment. You can adjust the rest in the preferences.",
                &[("email", &self.added[0])],
            ));
            widgets.nav.push(&widgets.done);
        }
    }
}
