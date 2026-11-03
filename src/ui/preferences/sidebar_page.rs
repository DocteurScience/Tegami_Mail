//! Settings → Sidebar: the placed sections, the icon
//! rail's fold-ups and Focus Mode.

use super::choices::{section_style_at, start_view_at};
use super::*;

impl Preferences {
    pub(super) fn on_toggle_focus(
        &mut self,
        part: crate::config::FocusPart,
        on: bool,
        sender: ComponentSender<Self>,
    ) {
        if self.focus.get(part) != on {
            self.focus.set(part, on);
            let _ = sender.output(PrefOutput::SetFocusMode(self.focus));
        }
    }

    pub(super) fn on_change_section_style(&mut self, idx: u32, sender: ComponentSender<Self>) {
        let style = section_style_at(idx);
        let _ = sender.output(PrefOutput::SetSectionStyle(style));
    }

    pub(super) fn on_change_start_view(&mut self, idx: u32, sender: ComponentSender<Self>) {
        let view = start_view_at(idx);
        let _ = sender.output(PrefOutput::SetStartView(view));
    }

    pub(super) fn on_change_folder_sort(&mut self, idx: u32, sender: ComponentSender<Self>) {
        let sort = crate::config::FolderSort::ALL
            .get(idx as usize)
            .copied()
            .unwrap_or_default();
        let _ = sender.output(PrefOutput::SetFolderSort(sort));
    }
}

impl Preferences {
    /// Flip one of the icon rail's fold-up switches and hand the app the whole
    /// set.
    pub(super) fn set_rail_fold(
        &mut self,
        switch: fn(&mut crate::config::RailFold) -> &mut bool,
        on: bool,
        sender: ComponentSender<Self>,
    ) {
        *switch(&mut self.rail_fold) = on;
        let _ = sender.output(PrefOutput::SetRailFold(self.rail_fold));
    }
}
