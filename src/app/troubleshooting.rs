//! Troubleshooting, from the main menu (docs/DESIGN.md, "Preferences":
//! troubleshooting tools are not preferences). The status bar, where it
//! sits, console mode and the console, and exporting the log: what left
//! the former System page.

use adw::prelude::*;
use relm4::prelude::*;

use crate::i18n::i18n;
use crate::ui::preferences::PrefOutput;

use super::model::AppModel;
use super::msg::AppMsg;

impl AppModel {
    /// Open the Troubleshooting dialog over the main window, each row
    /// holding the value it has now.
    pub(super) fn open_troubleshooting(&self, sender: &ComponentSender<Self>) {
        let page = adw::PreferencesPage::new();

        let bar = adw::PreferencesGroup::new();
        bar.set_title(&i18n("Status bar"));
        bar.set_description(Some(&i18n(
            "It shows what Tegami Mail is doing, and opens by itself for errors.",
        )));
        let shown = adw::SwitchRow::new();
        shown.set_title(&i18n("Show the status bar"));
        shown.set_active(self.status_bar_shown);
        {
            // The bar's own toggle: it may have been shown or hidden by
            // other means since, so only a change from what it is now acts.
            let s = sender.clone();
            shown.connect_active_notify(move |_| s.input(AppMsg::ToggleNotifications));
        }
        let top = adw::SwitchRow::new();
        top.set_title(&i18n("At the top of the window"));
        top.set_active(self.status_bar_top);
        {
            let s = sender.clone();
            top.connect_active_notify(move |row| {
                s.input(AppMsg::Pref(PrefOutput::SetStatusBarTop(row.is_active())))
            });
        }
        bar.add(&shown);
        bar.add(&top);
        page.add(&bar);

        let diagnostics = adw::PreferencesGroup::new();
        diagnostics.set_title(&i18n("Diagnostics"));
        let console_mode = adw::SwitchRow::new();
        console_mode.set_title(&i18n("Console mode"));
        console_mode.set_subtitle(&i18n(
            "Offers a detailed activity console in the status bar.",
        ));
        console_mode.set_active(self.console_mode);
        let open_console = adw::ActionRow::new();
        open_console.set_title(&i18n("Open the console"));
        open_console.set_subtitle(&i18n("Ctrl+Shift+C"));
        open_console.set_activatable(true);
        open_console.set_sensitive(self.console_mode);
        {
            let s = sender.clone();
            let open_console = open_console.clone();
            console_mode.connect_active_notify(move |row| {
                open_console.set_sensitive(row.is_active());
                s.input(AppMsg::Pref(PrefOutput::SetConsoleMode(row.is_active())));
            });
        }
        {
            let s = sender.clone();
            open_console.connect_activated(move |_| s.input(AppMsg::OpenConsole));
        }
        let export = adw::ActionRow::new();
        export.set_title(&i18n("Export the log…"));
        export.set_subtitle(&i18n(
            "Saves recent activity in a file, to attach to a bug report.",
        ));
        export.set_activatable(true);
        {
            let s = sender.clone();
            export.connect_activated(move |_| s.input(AppMsg::Pref(PrefOutput::ExportLog)));
        }
        diagnostics.add(&console_mode);
        diagnostics.add(&open_console);
        diagnostics.add(&export);
        page.add(&diagnostics);

        let view = adw::ToolbarView::new();
        view.add_top_bar(&adw::HeaderBar::new());
        view.set_content(Some(&page));
        let dialog = adw::Dialog::new();
        dialog.set_title(&i18n("Troubleshooting"));
        dialog.set_content_width(480);
        dialog.set_child(Some(&view));
        dialog.present(Some(&self.window));
    }
}
