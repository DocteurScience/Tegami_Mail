//! The icon rail: collapsing the sidebar to icons and back.

use super::*;

impl Sidebar {
    pub(super) fn on_set_collapsed(
        &mut self,
        collapsed: bool,
        widgets: &SidebarWidgets,
        sender: ComponentSender<Self>,
    ) {
        // Driven by the app's narrow-window breakpoint: same visual
        // change as the user's own toggle, but no CollapsedChanged
        // output — automatic switches must not overwrite the user's
        // persisted preference.
        if self.collapsed != collapsed {
            self.collapsed = collapsed;
            self.rail_open.clear();
            self.rail_open_accounts.clear();
            self.rebuild(widgets, &sender);
        }
    }

    pub(super) fn on_toggle_collapsed(
        &mut self,
        widgets: &SidebarWidgets,
        sender: ComponentSender<Self>,
    ) {
        self.collapsed = !self.collapsed;
        self.rail_open.clear();
        self.rail_open_accounts.clear();
        self.rebuild(widgets, &sender);
        let _ = sender.output(SidebarOutput::CollapsedChanged(self.collapsed));
    }
}
