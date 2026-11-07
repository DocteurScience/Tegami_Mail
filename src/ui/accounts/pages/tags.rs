//! Settings → Tags, as mockup 09's Tags dialog draws it: the tags in a
//! boxed list ending on "New tag…", and the tag finder on a row of its own
//! below.

use adw::prelude::*;
use relm4::prelude::*;

use super::super::{AccountsInput, AccountsWindow};
use crate::i18n::i18n;

/// The page, and the widgets on it that the panel reaches later.
#[derive(Debug)]
pub(in crate::ui::accounts) struct TagsPage {
    pub(in crate::ui::accounts) page: adw::PreferencesPage,
    pub(in crate::ui::accounts) find_tags_spinner: gtk::Spinner,
    pub(in crate::ui::accounts) find_tags_row: adw::ActionRow,
    pub(in crate::ui::accounts) tags_list: gtk::ListBox,
}

impl TagsPage {
    pub(in crate::ui::accounts) fn build(sender: &ComponentSender<AccountsWindow>) -> Self {
        relm4::view! {
            page = adw::PreferencesPage {
                // Tags (#71): a name and color per keyword. No title: the
                // page's header already says Tags, as the mockup's does.
                add = &adw::PreferencesGroup {
                    set_description: Some(
                        i18n("Tags are stored on the server as IMAP keywords, so Thunderbird and other clients show them too. The first nine are mapped to the 1–9 keys.").as_str()
                    ),
                    // Filled by `rebuild_tag_rows`: a row per tag, then
                    // the New Tag row.
                    #[name = "tags_list"]
                    gtk::ListBox {
                        add_css_class: "boxed-list",
                        set_selection_mode: gtk::SelectionMode::None,
                    },
                },
                // The tag finder: read every mailbox for keywords already
                // in use and offer them as tags.
                add = &adw::PreferencesGroup {
                    gtk::ListBox {
                        add_css_class: "boxed-list",
                        set_selection_mode: gtk::SelectionMode::None,
                        #[name = "find_tags_row"]
                        adw::ActionRow {
                            set_title: &i18n("Import tags from the servers…"),
                            set_tooltip_text: Some(i18n("Look through every mailbox for tags other clients have set").as_str()),
                            set_activatable: true,
                            connect_activated => AccountsInput::FindTags,
                            // Turning while the mailboxes are read.
                            #[name = "find_tags_spinner"]
                            add_suffix = &gtk::Spinner {
                                set_visible: false,
                            },
                            add_suffix = &gtk::Image {
                                set_icon_name: Some("go-next-symbolic"),
                            },
                        },
                    },
                },
            }
        }
        Self {
            page,
            find_tags_spinner,
            find_tags_row,
            tags_list,
        }
    }
}
