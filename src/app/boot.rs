//! The startup steps that run before the root component's model exists:
//! process-wide state (network, CSS, icons, theme, web fonts), the sidebar
//! layout, the accounts reconciled against GNOME Online
//! Accounts, and the main menu and its two stateful actions.

use adw::prelude::*;
use relm4::ComponentSender;

use crate::config::{self, AccountConfig};
use crate::i18n::i18n;
use crate::ui::sidebar::{SidebarInit, StartTarget};

use super::accounts::{forget_account_secrets, reconcile_goa, GoaReconcile};
use super::demo::demo_mode;
use super::palette::keyed_item;
use super::theme::install_scheme_css;
use super::{AppModel, AppMsg};
use crate::desktop::icon_theme::register_icons;

/// What the steps before the model read from disk and the environment.
pub(super) struct Boot {
    /// Every setting in privacy.toml, read once.
    pub(super) prefs: config::PrivacyFile,
    /// The sidebar layout, already pruned of accounts GOA removed and reset
    /// when "Remember the sidebar layout" is off.
    pub(super) sidebar_state: config::SidebarState,
    /// The user's own icon-rail choice, the one that is saved.
    pub(super) icon_only: bool,
    /// Whether the sidebar starts as the rail.
    pub(super) rail_now: bool,
    pub(super) config: Vec<AccountConfig>,
    /// Whether this run serves the built-in sample data (see spawn_workers).
    pub(super) demo_data: bool,
}

impl Boot {
    /// Runs the steps in the order they depend on each other.
    pub(super) fn read(root: &adw::ApplicationWindow) -> Self {
        let prefs = install_globals(root);
        let (mut sidebar_state, icon_only) = load_sidebar_layout(&prefs);
        let rail_now = icon_only;
        let config = load_accounts(&mut sidebar_state);
        // Decided after the GOA reconcile, which can add accounts.
        let demo_data = demo_mode() && config.is_empty();
        Boot {
            prefs,
            sidebar_state,
            icon_only,
            rail_now,
            config,
            demo_data,
        }
    }
}

/// Reads privacy.toml and puts the process-wide state in place.
fn install_globals(root: &adw::ApplicationWindow) -> config::PrivacyFile {
    let prefs = config::load_privacy();
    // Before the workers start, so their first connection already knows
    // whether there is a network to make it on (#442).
    crate::desktop::power::read_network(prefs.metered_save_data);
    relm4::set_global_css(include_str!("../styles.css"));
    register_icons(prefs.desktop_icons);
    // Before install_scheme_css and before the reader exists: both read
    // the theme's colors back, and both listen for the light/dark flip
    // that swaps a theme's two palettes — GTK runs those handlers in
    // connection order, so the palette has to be in place first.
    crate::desktop::theme::install(&config::load_theme());
    install_scheme_css(root);
    // Before any web view loads: a font fallback WebKit needed under
    // this runtime has to be in place for its first web process (#296).
    crate::desktop::web_fonts::startup(root);
    prefs
}

/// The saved sidebar layout, as this launch starts it: `(layout,
/// icon_only)`.
fn load_sidebar_layout(prefs: &config::PrivacyFile) -> (config::SidebarState, bool) {
    let mut sidebar_state = config::load_sidebar_state();
    // Settings → Sidebar: "Remember the icon rail" off starts every
    // launch with the full sidebar; "Remember the sidebar layout" off
    // starts with every section folded up — and every account, folded as
    // it arrives (SetAccount) — keeping only the account order. The two
    // are independent.
    let icon_only = prefs.remember_rail && sidebar_state.icon_only;
    if !prefs.remember_sidebar {
        // The layout starts fresh; the order things were put in stays.
        sidebar_state = config::SidebarState {
            order: sidebar_state.order,
            folder_order: sidebar_state.folder_order,
            ..Default::default()
        };
    }
    (sidebar_state, icon_only)
}

/// Load accounts, then reconcile against GNOME Online Accounts: drop any
/// imported account GOA no longer has, pause any whose Mail service is
/// switched off there. Reconciliation is skipped when GOA is unreachable,
/// so a momentary outage never wipes imported accounts. Live changes are
/// handled by the GOA watcher.
fn load_accounts(sidebar_state: &mut config::SidebarState) -> Vec<AccountConfig> {
    let mut config = config::load().unwrap_or_default();
    // Snapshot before reconciling: a removed account's alias-password
    // keyring entries are keyed by addresses only its config knows.
    let config_before_goa = config.clone();
    let goa_outcome = match crate::services::goa::live_state() {
        Some(live) => reconcile_goa(&mut config, &live),
        None => GoaReconcile::default(),
    };
    let goa_removed = goa_outcome.removed;
    if !goa_removed.is_empty() {
        forget_account_secrets(&goa_removed, &config_before_goa);
        sidebar_state.order.retain(|e| !goa_removed.contains(e));
        sidebar_state.collapsed.retain(|e| !goa_removed.contains(e));
        sidebar_state
            .folders_expanded
            .retain(|e| !goa_removed.contains(e));
        sidebar_state
            .tags_expanded_accounts
            .retain(|e| !goa_removed.contains(e));
        config::save_sidebar_state(sidebar_state);
    }
    if !goa_removed.is_empty() || goa_outcome.paused_changed || goa_outcome.servers_changed {
        let _ = config::save(&config);
    }
    config
}

/// The view the sidebar picks first instead of the first inbox (#256), from
/// Settings → General's start view and the last view left.
pub(super) fn start_target(start_view: config::StartView) -> Option<StartTarget> {
    let (last, last_account) = config::load_last_view();
    match start_view {
        config::StartView::AllInboxes => None,
        config::StartView::AccountInbox => Some(last_account)
            .filter(|e| !e.is_empty())
            .map(StartTarget::Inbox),
        config::StartView::LastFolder => last
            .filter(|v| !v.email.is_empty())
            .map(|v| StartTarget::Folder(v.email, v.path)),
    }
}

/// The launch settings of a sidebar instance: the sections open as the
/// saved layout left them. The main sidebar and the peek panel's mirror
/// differ only in `collapsed`, `mirror` and `start`.
pub(super) fn sidebar_init(
    state: &config::SidebarState,
    collapsed: bool,
    mirror: bool,
    start: Option<StartTarget>,
) -> SidebarInit {
    SidebarInit {
        collapsed,
        mirror,
        tags_expanded: state.tags_expanded,
        favorites_expanded: state.favorites_expanded,
        smart_expanded: state.smart_expanded,
        start,
    }
}

/// The primary menu, the two menus filled in later (help by
/// rebuild_help_menu, undo/redo by refresh_undo_menu) and the actions of
/// its two check items.
pub(super) struct MainMenu {
    pub(super) menu: gtk::gio::Menu,
    pub(super) help: gtk::gio::Menu,
    pub(super) undo: gtk::gio::Menu,
    pub(super) reading_pane_action: gtk::gio::SimpleAction,
}

/// Sectioned: sync / accounts / settings / printing / help / quit.
pub(super) fn build_main_menu(
    reading_pane: config::ReadingPane,
    sender: &ComponentSender<AppModel>,
) -> MainMenu {
    let menu = gtk::gio::Menu::new();
    let help_menu = gtk::gio::Menu::new();
    let model_undo_menu = gtk::gio::Menu::new();
    {
        // The design's main menu (docs/DESIGN.md, "Menus"), with Print and
        // Quit kept (owner's decision, docs/SUIVI.md, 2026-11-02). Undo,
        // Reading Pane and Fullscreen are no longer here; their keys still
        // work (Ctrl+Z, Ctrl+Alt+1-3, F11), and
        // refresh_undo_menu still fills `model_undo_menu`, unshown, as it
        // also enables the Undo and Redo actions.
        let sync = gtk::gio::Menu::new();
        sync.append_item(&keyed_item(&i18n("Sync now"), "win.sync-now", "F5"));
        menu.append_section(None, &sync);

        let accounts = gtk::gio::Menu::new();
        accounts.append(Some(i18n("Add account…").as_str()), Some("win.add-account"));
        accounts.append(
            Some(i18n("GNOME Online Accounts…").as_str()),
            Some("win.online-accounts"),
        );
        menu.append_section(None, &accounts);

        let settings = gtk::gio::Menu::new();
        settings.append(Some(i18n("Settings").as_str()), Some("win.accounts"));
        menu.append_section(None, &settings);

        let printing = gtk::gio::Menu::new();
        printing.append(
            Some(i18n("Print preview…").as_str()),
            Some("win.print-preview"),
        );
        printing.append(Some(i18n("Print message…").as_str()), Some("win.print"));
        menu.append_section(None, &printing);

        menu.append_section(None, &help_menu);

        // Last, where a Quit item belongs.
        let quit = gtk::gio::Menu::new();
        quit.append(Some(i18n("Quit").as_str()), Some("app.quit"));
        menu.append_section(None, &quit);
    }
    MainMenu {
        menu,
        help: help_menu,
        undo: model_undo_menu,
        reading_pane_action: reading_pane_action(reading_pane, sender),
    }
}

/// The "reading-pane" radio items' action, its state the pane's key.
fn reading_pane_action(
    pane: config::ReadingPane,
    sender: &ComponentSender<AppModel>,
) -> gtk::gio::SimpleAction {
    let action = gtk::gio::SimpleAction::new_stateful(
        "reading-pane",
        Some(gtk::glib::VariantTy::STRING),
        &pane.key().to_variant(),
    );
    let s = sender.clone();
    action.connect_change_state(move |action, value| {
        let pane = value
            .and_then(|v| v.get::<String>())
            .and_then(|k| config::ReadingPane::from_key(&k));
        if let Some(pane) = pane {
            action.set_state(&pane.key().to_variant());
            s.input(AppMsg::SetReadingPane(pane));
        }
    });
    action
}
