//! How the list looks: the row look, columns, sorting, the bars, swipes
//! and the palette.

use super::layout::schedule_midnight_refresh;
use super::sort::SortOrder;
use super::MessageList;
use adw::prelude::*;
use relm4::prelude::*;

impl MessageList {
    pub(super) fn on_set_sender_logos(&mut self, on: bool) {
        if self.sender_logos != on {
            self.sender_logos = on;
            self.face_gen += 1;
            self.sync_look();
        }
    }

    pub(super) fn on_set_single_line(&mut self, on: bool) {
        if self.single_line != on {
            self.single_line = on;
            // The date headings line up with a single line's text.
            if on {
                self.list_view.add_css_class("single-line");
            } else {
                self.list_view.remove_css_class("single-line");
            }
            self.sync_look();
        }
    }

    pub(super) fn on_set_look(&mut self, avatars: bool, preview_lines: u32) {
        let preview_lines = preview_lines.min(3);
        if self.avatars != avatars || self.preview_lines != preview_lines {
            self.avatars = avatars;
            self.preview_lines = preview_lines;
            self.sync_look();
        }
    }

    pub(super) fn on_set_avatars(&mut self, on: bool) {
        if self.avatars != on {
            self.avatars = on;
            self.sync_look();
        }
    }

    pub(super) fn on_set_gravatar(&mut self, on: bool) {
        if self.gravatar != on {
            self.gravatar = on;
            self.face_gen += 1;
            self.sync_look();
        }
    }

    pub(super) fn on_set_show_recipient(&mut self, on: bool) {
        if self.show_recipient != on {
            self.show_recipient = on;
            self.sync_look();
        }
    }

    pub(super) fn on_set_in_junk(&mut self, on: bool) {
        if self.in_junk != on {
            self.in_junk = on;
            self.sync_look();
        }
    }

    pub(super) fn on_set_in_drafts(&mut self, on: bool) {
        if self.in_drafts != on {
            self.in_drafts = on;
            self.sync_look();
        }
    }

    pub(super) fn on_contact_photos_changed(&mut self) {
        // The circles on screen look again; the rest do when shown.
        self.face_gen += 1;
        self.sync_look();
    }

    pub(super) fn on_contact_names_changed(&mut self) {
        // Rows showing the same message are kept as they are unless the
        // look is stale, and every name on them may be another now.
        self.look_stale.set(true);
        self.rebuild();
    }

    pub(super) fn on_set_colorize(&mut self, on: bool) {
        if self.colorize != on {
            self.colorize = on;
            self.sync_look();
        }
    }

    pub(super) fn on_day_changed(&mut self, sender: &ComponentSender<Self>) {
        // Re-render so relative labels like "Today" reflect the new date.
        self.rebuild();
        schedule_midnight_refresh(sender);
    }

    pub(super) fn on_set_account_colors(&mut self, colors: std::collections::HashMap<u32, String>) {
        self.account_colors = colors;
        self.refresh_tint_css();
        self.sync_look();
    }

    pub(super) fn on_set_sort(&mut self, order: SortOrder) {
        if self.sort != order {
            self.sort = order;
            self.rebuild();
            self.scroll_top();
        }
    }

    pub(super) fn on_pane_width(&mut self, width: i32) {
        if self.pane_width != width {
            let before = self.single_line.then(|| self.fitted_widths());
            self.pane_width = width;
            if before.is_some_and(|b| b != self.fitted_widths()) {
                self.sync_look();
            }
        }
    }

    pub(super) fn on_set_swipe_enabled(&mut self, on: bool) {
        self.shared.swipe_enabled.set(on);
        // The rows on screen switch their trackers; the rest do so
        // when shown.
        self.shared.refresh_all();
    }
}
