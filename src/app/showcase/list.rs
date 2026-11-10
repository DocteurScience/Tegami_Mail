//! Showcase hooks for the list, the reader and the sidebar: opening rows,
//! menus and palettes, swipes, the rail and the peek panel.

use adw::prelude::*;
use relm4::prelude::*;

use crate::app::model::AppModel;
use crate::app::msg::AppMsg;
use crate::ui::message_list::MessageListInput;
use crate::ui::message_view::MessageViewInput;
use crate::ui::showcase::showcase_capture;
use crate::ui::sidebar::SidebarInput;

impl AppModel {
    /// TEGAMI_SHOWCASE_HEIGHTS=1 logs the allocated height of the
    /// sidebar's section headings and their rows just before the
    /// capture, for checking pill geometry without a pointer.
    pub(super) fn stage_heights(&self, delay: u32) {
        if std::env::var("TEGAMI_SHOWCASE_HEIGHTS").is_ok() {
            let sidebar = self.sidebar.widget().clone().upcast::<gtk::Widget>();
            gtk::glib::timeout_add_seconds_local_once(delay.saturating_sub(1).max(1), move || {
                fn walk(w: &gtk::Widget, root: &gtk::Widget) {
                    let classes = w.css_classes();
                    let heading = classes.iter().any(|c| c == "unified-folders-toggle");
                    let child_row = w.is::<gtk::ListBoxRow>()
                        && w.parent().is_some_and(|p| {
                            p.css_classes().iter().any(|c| c == "section-child-list")
                        });
                    if heading || child_row {
                        let y = w.compute_bounds(root).map(|b| b.y() as i32).unwrap_or(-1);
                        tracing::info!(
                            "showcase heights: {} y={y} h={} margins={}/{} classes={:?}",
                            if heading { "heading" } else { "row" },
                            w.height(),
                            w.margin_top(),
                            w.margin_bottom(),
                            classes,
                        );
                    }
                    let mut c = w.first_child();
                    while let Some(ch) = c {
                        walk(&ch, root);
                        c = ch.next_sibling();
                    }
                }
                walk(&sidebar, &sidebar);
            });
        }
    }

    /// Moves the list's selection onto its first row at 3 s, which opens
    /// the demo conversation.
    pub(super) fn stage_select_first_row(&self) {
        let list = self.message_list.sender().clone();
        gtk::glib::timeout_add_seconds_local_once(3, move || {
            let _ = list.send(MessageListInput::MoveSelection(1));
        });
    }

    /// Highlights the conversation's mid-thread card (account 1, message
    /// 2) at 6 s.
    pub(super) fn stage_open_card(&self) {
        let view = self.message_view.sender().clone();
        gtk::glib::timeout_add_seconds_local_once(6, move || {
            let _ = view.send(crate::ui::message_view::MessageViewInput::CardClicked {
                account_id: 1,
                id: 2,
                mode: crate::ui::message_view::SelectMode::Plain,
            });
        });
    }

    /// TEGAMI_SHOWCASE_SWIPE=N[:left|right] swipes row N fully and
    /// releases it at 7s, with TEGAMI_SHOWCASE_BURST=<path> writing
    /// stills through the commit exit — the only way to see the
    /// gesture's animation here (no input injection).
    pub(super) fn stage_swipe(&self, root: &adw::ApplicationWindow) {
        if let Ok(spec) = std::env::var("TEGAMI_SHOWCASE_SWIPE") {
            let (idx, side) = spec.split_once(':').unwrap_or((spec.as_str(), "left"));
            let index: usize = idx.trim().parse().unwrap_or(0);
            let left = side != "right";
            let list = self.message_list.sender().clone();
            let burst = std::env::var("TEGAMI_SHOWCASE_BURST").ok();
            let win = root.clone();
            {
                // An occluded window's frame clock is suspended, so
                // raise it well before the animation runs.
                let win = win.clone();
                gtk::glib::timeout_add_seconds_local_once(5, move || win.present());
            }
            gtk::glib::timeout_add_seconds_local_once(7, move || {
                let _ = list.send(MessageListInput::DebugSwipe { index, left });
                if let Some(base) = burst.clone() {
                    for (i, ms) in [40u64, 120, 200, 300, 500].iter().enumerate() {
                        let win = win.clone();
                        let path = format!("{base}.{i}.png");
                        gtk::glib::timeout_add_local_once(
                            std::time::Duration::from_millis(*ms),
                            move || {
                                showcase_capture(win.upcast_ref::<gtk::Widget>(), &path);
                            },
                        );
                    }
                }
            });
        }
    }

    /// TEGAMI_SHOWCASE_ROW=N selects the list's row N at 4s (with
    /// TEGAMI_SHOWCASE_STAGE=0), for capturing or probing one message.
    pub(super) fn stage_row(&self) {
        if let Some(Ok(n)) = std::env::var("TEGAMI_SHOWCASE_ROW")
            .ok()
            .map(|v| v.parse::<u32>())
        {
            let list = self.message_list.sender().clone();
            gtk::glib::timeout_add_seconds_local_once(4, move || {
                for _ in 0..=n {
                    let _ = list.send(MessageListInput::MoveSelection(1));
                }
            });
        }
    }

    /// TEGAMI_SHOWCASE_UNSUB=ask|go presses the open card's
    /// Unsubscribe button at 7s (after TEGAMI_SHOWCASE_ROW opened
    /// it): `ask` stops at the confirmation, `go` skips it.
    pub(super) fn stage_unsubscribe(&self) {
        if std::env::var("TEGAMI_SHOWCASE_UNSUB").is_ok() {
            let view = self.message_view.sender().clone();
            gtk::glib::timeout_add_seconds_local_once(7, move || {
                let _ = view.send(MessageViewInput::Unsubscribe {
                    account_id: 1,
                    id: 10,
                });
            });
        }
    }

    /// TEGAMI_SHOWCASE_INVITE=accept|maybe|decline|calendar presses
    /// that button on the open card's invitation banner at 7s
    /// (after TEGAMI_SHOWCASE_ROW=8 opened the meeting request).
    pub(super) fn stage_invite(&self) {
        if let Ok(which) = std::env::var("TEGAMI_SHOWCASE_INVITE") {
            use crate::models::Rsvp;
            use crate::ui::message_view::InviteAction;
            let action = match which.as_str() {
                "maybe" => InviteAction::Answer(Rsvp::Tentative),
                "decline" => InviteAction::Answer(Rsvp::Declined),
                "calendar" => InviteAction::AddToCalendar,
                _ => InviteAction::Answer(Rsvp::Accepted),
            };
            let view = self.message_view.sender().clone();
            gtk::glib::timeout_add_seconds_local_once(7, move || {
                let _ = view.send(MessageViewInput::InviteAction {
                    account_id: 1,
                    id: 13,
                    action,
                });
            });
        }
    }

    /// TEGAMI_SHOWCASE_UNIFIED=favorites|tags|smart opens that merged
    /// row at 3s and again at 9s, so the timing logs show a cold and a
    /// warm open. TEGAMI_SHOWCASE_UNIFIED_AT=<s> starts that at s seconds
    /// rather than 3, once a real profile's accounts have loaded.
    pub(super) fn stage_unified(&self) {
        if let Ok(which) = std::env::var("TEGAMI_SHOWCASE_UNIFIED") {
            let at: u32 = std::env::var("TEGAMI_SHOWCASE_UNIFIED_AT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(3);
            let pick = move || match which.as_str() {
                "tags" => SidebarInput::UnifiedTagsSelected,
                "smart" => SidebarInput::SmartRowSelected(0),
                _ => SidebarInput::UnifiedFavoritesSelected,
            };
            let sb = self.sidebar.sender().clone();
            let p = pick.clone();
            gtk::glib::timeout_add_seconds_local_once(at, move || {
                let _ = sb.send(p());
            });
            let sb = self.sidebar.sender().clone();
            gtk::glib::timeout_add_seconds_local_once(at + 6, move || {
                let _ = sb.send(pick());
            });
        }
    }

    /// TEGAMI_SHOWCASE_ROW_MENU=1 opens the first row's context
    /// menu at 5s (pair with TEGAMI_SHOWCASE_MENU to capture it).
    pub(super) fn stage_row_menu(&self) {
        if std::env::var("TEGAMI_SHOWCASE_ROW_MENU").is_ok() {
            let ml = self.message_list.sender().clone();
            gtk::glib::timeout_add_seconds_local_once(5, move || {
                let _ = ml.send(MessageListInput::DebugRowMenu);
            });
        }
    }
}

/// TEGAMI_SHOWCASE_APPLY_FILTERS=inboxes|<account>:<folder>
/// starts a manual filter run at 5s (#198), the way Apply Now
/// and the folder menu's Apply Filters do, and the run's
/// report lands in the status bar a beat later.
pub(super) fn stage_apply_filters(sender: &ComponentSender<AppModel>) {
    if let Ok(spec) = std::env::var("TEGAMI_SHOWCASE_APPLY_FILTERS") {
        let s = sender.clone();
        gtk::glib::timeout_add_seconds_local_once(5, move || {
            let targets = spec
                .split_once(':')
                .and_then(|(a, f)| Some(vec![(a.parse().ok()?, f.parse().ok()?)]))
                .unwrap_or_default();
            s.input(AppMsg::ApplyFilters(targets));
        });
    }
}

/// TEGAMI_SHOWCASE_READER_MENU=1 opens the reader header's ⋯
/// overflow menu at 5s (pair with TEGAMI_SHOWCASE_MENU to
/// capture it; the window must be narrow enough to collapse).
pub(super) fn stage_reader_menu(sender: &ComponentSender<AppModel>) {
    if std::env::var("TEGAMI_SHOWCASE_READER_MENU").is_ok() {
        let s = sender.input_sender().clone();
        gtk::glib::timeout_add_seconds_local_once(5, move || {
            let _ = s.send(AppMsg::ReaderOverflowMenu);
        });
    }
}

/// TEGAMI_SHOWCASE_TAGS=panel opens the Tags panel at 5s, =new the New
/// Tag dialog; each captures itself to `<TEGAMI_SHOWCASE>.tags.png`, as a
/// popover and a dialog are not part of the window's snapshot.
pub(super) fn stage_tags(sender: &ComponentSender<AppModel>) {
    if let Ok(which) = std::env::var("TEGAMI_SHOWCASE_TAGS") {
        let s = sender.input_sender().clone();
        gtk::glib::timeout_add_seconds_local_once(5, move || {
            let _ = s.send(if which == "new" {
                AppMsg::NewTagDialog { apply: false }
            } else {
                AppMsg::ReaderTagMenu
            });
        });
    }
}

/// TEGAMI_SHOWCASE_LIST=sort opens the Sort and display popover at 5s
/// (mockup 18), =filters the search field and its Filters menu (19),
/// =active the search field with two filters set (20), =period the
/// custom-period dialog on a range (21).
pub(super) fn stage_list_filters(sender: &ComponentSender<AppModel>) {
    if let Ok(which) = std::env::var("TEGAMI_SHOWCASE_LIST") {
        let s = sender.input_sender().clone();
        gtk::glib::timeout_add_seconds_local_once(5, move || {
            let _ = s.send(AppMsg::ShowcaseListFilters(which));
        });
    }
}

/// TEGAMI_SHOWCASE_PRINT=1 opens the print preview of the open
/// message at 6s (#359); TEGAMI_SHOWCASE_TOP captures it.
pub(super) fn stage_print(sender: &ComponentSender<AppModel>) {
    if std::env::var("TEGAMI_SHOWCASE_PRINT").is_ok() {
        let s = sender.input_sender().clone();
        gtk::glib::timeout_add_seconds_local_once(6, move || {
            let _ = s.send(AppMsg::PrintPreview);
        });
    }
}

/// TEGAMI_SHOWCASE_POPOUT=1 opens the open message in a window
/// of its own at 6s; TEGAMI_SHOWCASE_TOP captures it.
pub(super) fn stage_popout(sender: &ComponentSender<AppModel>) {
    if std::env::var("TEGAMI_SHOWCASE_POPOUT").is_ok() {
        let s = sender.input_sender().clone();
        gtk::glib::timeout_add_seconds_local_once(6, move || {
            let _ = s.send(AppMsg::ShowcasePopout);
        });
    }
}

/// TEGAMI_SHOWCASE_EDIT_AS_NEW=1 copies the open message into
/// the composer at 6s (#232), as its menu entry does.
pub(super) fn stage_edit_as_new(sender: &ComponentSender<AppModel>) {
    if std::env::var("TEGAMI_SHOWCASE_EDIT_AS_NEW").is_ok() {
        let s = sender.input_sender().clone();
        gtk::glib::timeout_add_seconds_local_once(6, move || {
            let _ = s.send(AppMsg::EditAsNewCurrent);
        });
    }
}

/// TEGAMI_SHOWCASE_MOVE=1 opens the Move To… picker at 5s
/// (it captures itself a second later).
pub(super) fn stage_move(sender: &ComponentSender<AppModel>) {
    if std::env::var("TEGAMI_SHOWCASE_MOVE").is_ok() {
        let s = sender.input_sender().clone();
        gtk::glib::timeout_add_seconds_local_once(5, move || {
            let _ = s.send(AppMsg::MoveToMenu);
        });
    }
}

impl AppModel {
    /// TEGAMI_SHOWCASE_RAIL=1 collapses the sidebar to the rail
    /// at 3s (the user's own toggle, fold-ups and all).
    pub(super) fn stage_rail(&self) {
        if std::env::var("TEGAMI_SHOWCASE_RAIL").is_ok() {
            let sb = self.sidebar.sender().clone();
            gtk::glib::timeout_add_seconds_local_once(3, move || {
                let _ = sb.send(SidebarInput::ToggleCollapsed);
            });
        }
    }
}

/// TEGAMI_SHOWCASE_PEEK=<seconds>[,<seconds>…] works the header's
/// sidebar toggle at each of those moments (1 or anything
/// unparseable means 8 s), with the window raised first so the
/// slide really animates; TEGAMI_SHOWCASE_BURST writes stills
/// through the first one. In a narrow window (a window.toml
/// under the 1094px breakpoint) that is the floating peek; at
/// full width it is the rail
/// expanding and folding again.
pub(super) fn stage_peek(root: &adw::ApplicationWindow, sender: &ComponentSender<AppModel>) {
    if let Ok(spec) = std::env::var("TEGAMI_SHOWCASE_PEEK") {
        let mut at: Vec<f64> = spec
            .split(',')
            .filter_map(|p| p.trim().parse::<f64>().ok())
            .filter(|s| *s > 1.5)
            .collect();
        if at.is_empty() {
            at.push(8.0);
        }
        let burst = std::env::var("TEGAMI_SHOWCASE_BURST").ok();
        let win = root.clone();
        {
            let win = win.clone();
            let first = at[0];
            gtk::glib::timeout_add_local_once(
                std::time::Duration::from_millis(((first - 1.5) * 1000.0) as u64),
                move || win.present(),
            );
        }
        for (n, secs) in at.into_iter().enumerate() {
            let s = sender.clone();
            let win = win.clone();
            let burst = (n == 0).then(|| burst.clone()).flatten();
            gtk::glib::timeout_add_local_once(
                std::time::Duration::from_millis((secs * 1000.0) as u64),
                move || {
                    s.input(AppMsg::ToggleSidebar);
                    if let Some(base) = burst {
                        for (i, ms) in [60u64, 120, 200, 400].iter().enumerate() {
                            let win = win.clone();
                            let path = format!("{base}.{i}.png");
                            gtk::glib::timeout_add_local_once(
                                std::time::Duration::from_millis(*ms),
                                move || {
                                    showcase_capture(win.upcast_ref::<gtk::Widget>(), &path);
                                },
                            );
                        }
                    }
                },
            );
        }
    }
}
