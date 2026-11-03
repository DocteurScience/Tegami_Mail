//! Settings → Sidebar: the sections and the icon
//! rail.

use adw::prelude::*;

use super::super::{PrefInput, PrefOutput, Preferences, ViewSender};
use crate::i18n::i18n;

/// The page, and the rows on it that the window reaches later.
#[derive(Debug)]
pub(in crate::ui::preferences) struct SidebarPage {
    pub(in crate::ui::preferences) page: adw::PreferencesPage,
    pub(in crate::ui::preferences) start_view_row: adw::ComboRow,
    pub(in crate::ui::preferences) folder_sort_row: adw::ComboRow,
    pub(in crate::ui::preferences) chevron_side_row: adw::ComboRow,
    pub(in crate::ui::preferences) account_click_row: adw::SwitchRow,
    pub(in crate::ui::preferences) sidebar_hover_expand_row: adw::SwitchRow,
    pub(in crate::ui::preferences) remember_sidebar_row: adw::SwitchRow,
    pub(in crate::ui::preferences) remember_rail_row: adw::SwitchRow,
    pub(in crate::ui::preferences) seen_counts_row: adw::SwitchRow,
    pub(in crate::ui::preferences) section_style_row: adw::ComboRow,
    pub(in crate::ui::preferences) tags_placement_row: adw::ComboRow,
    pub(in crate::ui::preferences) favorites_placement_row: adw::ComboRow,
    pub(in crate::ui::preferences) rail_dots_row: adw::SwitchRow,
    pub(in crate::ui::preferences) rail_fold_row: adw::ExpanderRow,
    pub(in crate::ui::preferences) rail_fold_accounts_row: adw::SwitchRow,
    pub(in crate::ui::preferences) rail_fold_tags_row: adw::SwitchRow,
}

impl SidebarPage {
    pub(in crate::ui::preferences) fn build(_model: &Preferences, sender: &ViewSender) -> Self {
        relm4::view! {
            page = adw::PreferencesPage {
                add = &adw::PreferencesGroup {
                    set_title: &i18n("Sidebar"),

                    #[name = "start_view_row"]
                    adw::ComboRow {
                        set_title: &i18n("Open at startup"),
                        connect_selected_notify[sender] => move |row| {
                            sender.input(PrefInput::ChangeStartView(row.selected()));
                        },
                    },

                    #[name = "folder_sort_row"]
                    adw::ComboRow {
                        set_title: &i18n("Folder order"),
                        set_subtitle: &i18n("Dragging a folder switches to Custom Order."),
                        connect_selected_notify[sender] => move |row| {
                            sender.input(PrefInput::ChangeFolderSort(row.selected()));
                        },
                    },

                    #[name = "chevron_side_row"]
                    adw::ComboRow {
                        set_title: &i18n("Chevron placement"),
                        connect_selected_notify[sender] => move |row| {
                            sender.input(PrefInput::ChangeChevronSide(row.selected()));
                        },
                    },

                    #[name = "account_click_row"]
                    adw::SwitchRow {
                        set_title: &i18n("Fold accounts with one click"),
                        set_subtitle: &i18n("When off, only its chevron or a double click activates the fold."),
                        connect_active_notify[sender] => move |row| {
                            let _ = sender.output(PrefOutput::SetAccountClickFolds(row.is_active()));
                        },
                    },

                    #[name = "sidebar_hover_expand_row"]
                    adw::SwitchRow {
                        set_title: &i18n("Expand the sidebar on hover"),
                        set_subtitle: &i18n("Hovering the icon rail floats the full sidebar out."),
                        connect_active_notify[sender] => move |row| {
                            let _ = sender.output(PrefOutput::SetSidebarHoverExpand(row.is_active()));
                        },
                    },

                    #[name = "remember_sidebar_row"]
                    adw::SwitchRow {
                        set_title: &i18n("Remember the sidebar layout"),
                        set_subtitle: &i18n("Off starts every launch folded up."),
                        connect_active_notify[sender] => move |row| {
                            let _ = sender.output(PrefOutput::SetRememberSidebar(row.is_active()));
                        },
                    },

                    #[name = "remember_rail_row"]
                    adw::SwitchRow {
                        set_title: &i18n("Remember icon rail state"),
                        connect_active_notify[sender] => move |row| {
                            let _ = sender.output(PrefOutput::SetRememberRail(row.is_active()));
                        },
                    },

                    #[name = "seen_counts_row"]
                    adw::SwitchRow {
                        set_title: &i18n("Highlight only new unread mail"),
                        set_subtitle: &i18n("A count turns grey once you have looked at the folder."),
                        connect_active_notify[sender] => move |row| {
                            let _ = sender.output(PrefOutput::SetSeenCounts(row.is_active()));
                        },
                    },
                },

                add = &adw::PreferencesGroup {
                    set_title: &i18n("Sections"),

                    #[name = "section_style_row"]
                    adw::ComboRow {
                        set_title: &i18n("Section style"),
                        connect_selected_notify[sender] => move |row| {
                            sender.input(PrefInput::ChangeSectionStyle(row.selected()));
                        },
                    },

                    #[name = "tags_placement_row"]
                    adw::ComboRow {
                        set_title: &i18n("Tags placement"),
                        connect_selected_notify[sender] => move |row| {
                            sender.input(PrefInput::ChangeTagsPlacement(row.selected()));
                        },
                    },

                    #[name = "favorites_placement_row"]
                    adw::ComboRow {
                        set_title: &i18n("Favorites placement"),
                        connect_selected_notify[sender] => move |row| {
                            sender.input(PrefInput::ChangeFavoritesPlacement(row.selected()));
                        },
                    },

                },

                add = &adw::PreferencesGroup {
                    set_title: &i18n("Icon rail"),

                    #[name = "rail_dots_row"]
                    adw::SwitchRow {
                        set_title: &i18n("Unread dots instead of counts"),
                        set_subtitle: &i18n("The count stays in the tooltip."),
                        connect_active_notify[sender] => move |row| {
                            let _ = sender.output(PrefOutput::SetRailDots(row.is_active()));
                        },
                    },

                    #[name = "rail_fold_row"]
                    adw::ExpanderRow {
                        set_title: &i18n("Fold up expanded items"),
                        set_subtitle: &i18n("The items below start folded up in the rail. Long-press to open one."),
                        set_show_enable_switch: true,
                        set_expanded: false,
                        connect_enable_expansion_notify[sender] => move |row| {
                            sender.input(PrefInput::ToggleRailFoldEnabled(row.enables_expansion()));
                        },

                        #[name = "rail_fold_accounts_row"]
                        add_row = &adw::SwitchRow {
                            set_title: &i18n("All accounts"),
                            connect_active_notify[sender] => move |row| {
                                sender.input(PrefInput::ToggleRailFoldAccounts(row.is_active()));
                            },
                        },
                        #[name = "rail_fold_tags_row"]
                        add_row = &adw::SwitchRow {
                            set_title: &i18n("Tags"),
                            connect_active_notify[sender] => move |row| {
                                sender.input(PrefInput::ToggleRailFoldTags(row.is_active()));
                            },
                        },
                    },
                },
            }
        }
        Self {
            page,
            start_view_row,
            folder_sort_row,
            chevron_side_row,
            account_click_row,
            sidebar_hover_expand_row,
            remember_sidebar_row,
            remember_rail_row,
            seen_counts_row,
            section_style_row,
            tags_placement_row,
            favorites_placement_row,
            rail_dots_row,
            rail_fold_row,
            rail_fold_accounts_row,
            rail_fold_tags_row,
        }
    }
}
