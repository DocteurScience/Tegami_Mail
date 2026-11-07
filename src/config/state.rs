//! state.toml: what the app remembers between runs that is not a
//! preference (pane sizes, the drawer, the last view, fetch
//! modes, recent fonts, one-time flags).

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use super::{config_file, read_toml, save_plain_toml};

// ---------------------------------------------------------------------------
// Keyring health check + one-time setup-help flag
// ---------------------------------------------------------------------------

#[derive(Debug, Default, Deserialize, Serialize)]
pub(super) struct StateFile {
    /// Set once the user dismisses the Linux Mint keyring setup tip.
    #[serde(default)]
    mint_keyring_help_dismissed: bool,
    /// In-message attachment drawer: collapsed (showing only its header).
    #[serde(default)]
    drawer_collapsed: bool,
    /// The way of asking for message summaries that worked for an account
    /// whose server rejects the default (#226), by email: "headers",
    /// "no-previews" or "headers-no-previews". Absent = the default. Written
    /// only once a stepped-to mode has actually loaded a folder, so a
    /// launch starts there instead of failing its way down the ladder.
    #[serde(default, skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    fetch_modes: std::collections::BTreeMap<String, String>,
    /// Expanded attachment-drawer height in px (the dragged split).
    #[serde(default = "default_drawer_height")]
    drawer_height: i32,
    /// Attachment drawer shows an alphabetical list instead of the thumbnail grid.
    #[serde(default)]
    drawer_list_view: bool,
    /// The drawer's list view sorts Z→A instead of A→Z.
    #[serde(default)]
    drawer_sort_desc: bool,
    /// Message-list pane width in px (#28 — it reset every launch).
    #[serde(default = "default_list_pane_width")]
    pub(super) list_pane_width: i32,
    /// The list's height with the reader below it (#475).
    #[serde(default = "default_list_pane_height")]
    list_pane_height: i32,
    /// The About window's height, remembering the user's vertical resize.
    /// (The settings window's is fixed since its two-pane layout, #141; an
    /// old `prefs_height` key in the file is ignored.)
    #[serde(default = "default_about_height")]
    about_height: i32,
    /// The welcome wizard has been completed once (Start Reading pressed),
    /// so an install with no accounts is not greeted with it again — a
    /// restart right after the wizard, for the app icon, must not loop.
    #[serde(default)]
    wizard_completed: bool,
    /// The Flatpak runtime commit under which WebKit hung on the host's
    /// fonts and was given the runtime's own instead (#296). Kept per
    /// commit, so a runtime update gets another try with every font.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    web_font_fallback: Option<String>,
    /// The mail view open last (#256): an account's folder; an empty
    /// `email` (a file from when All Inboxes existed) reopens nothing. Accounts are named by address, which stays
    /// true across launches where an account's number may not.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    last_view: Option<LastView>,
    /// The account whose mail was open last, kept through a visit to All
    /// Inboxes so "the last account's inbox" still has an account to mean.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    last_account: String,
    /// Each folder's unread count as of the last time it was looked at
    /// (#343), keyed "address␟folder path".
    #[serde(default, skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    unread_seen: std::collections::BTreeMap<String, u32>,
    /// The fonts last picked with the composer's Other Font… (#358), the
    /// newest first, for the font menu to offer again.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    recent_fonts: Vec<RecentFont>,
}

/// A font family picked in the composer (#358), and whether it is
/// monospaced, which decides the generic family it falls back to.
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub(crate) struct RecentFont {
    pub(crate) name: String,
    #[serde(default)]
    pub(crate) mono: bool,
}

/// How many picked fonts the font menu keeps.
const RECENT_FONTS: usize = 5;

pub(crate) fn load_recent_fonts() -> Vec<RecentFont> {
    load_state().recent_fonts
}

/// Put `font` at the head of the recent fonts, once, and keep the newest
/// few.
pub(crate) fn note_recent_font(font: RecentFont) {
    update_state(|s| {
        s.recent_fonts
            .retain(|f| !f.name.eq_ignore_ascii_case(&font.name));
        s.recent_fonts.insert(0, font);
        s.recent_fonts.truncate(RECENT_FONTS);
    });
}

fn default_about_height() -> i32 {
    740
}

pub(super) fn default_list_pane_width() -> i32 {
    350
}

fn default_list_pane_height() -> i32 {
    320
}

fn default_drawer_height() -> i32 {
    160
}

pub(super) fn state_path() -> Option<PathBuf> {
    config_file("state.toml")
}

/// Read the remembered UI state. A missing or unreadable file falls back to
/// deserializing an empty document, not to `StateFile::default()`: the derived
/// Default hands every field its type's zero, which is not what the
/// `#[serde(default = …)]` on each one says a first run should get (a pane
/// width of 0). Every field carries a serde
/// default, so the empty document always parses.
fn load_state() -> StateFile {
    read_toml::<StateFile>(state_path())
        .or_else(|| toml::from_str::<StateFile>("").ok())
        .unwrap_or_default()
}

/// Each folder's unread count when it was last looked at (#343).
pub(crate) fn load_unread_seen() -> std::collections::BTreeMap<String, u32> {
    load_state().unread_seen
}

pub(crate) fn save_unread_seen(seen: &std::collections::BTreeMap<String, u32>) {
    update_state(|s| s.unread_seen = seen.clone());
}

/// Change one thing in state.toml.
fn update_state(change: impl FnOnce(&mut StateFile)) {
    let mut s = load_state();
    change(&mut s);
    save_state(&s);
}

fn save_state(state: &StateFile) {
    save_plain_toml(state_path(), state);
}

/// A mail view to reopen at launch (#256).
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub(crate) struct LastView {
    /// The account's address; empty in files from when All Inboxes existed.
    #[serde(default)]
    pub(crate) email: String,
    /// The folder's path on the server.
    #[serde(default)]
    pub(crate) path: String,
}

/// The view open last and the account whose mail was open last.
pub(crate) fn load_last_view() -> (Option<LastView>, String) {
    let s = load_state();
    (s.last_view, s.last_account)
}

/// Record the view just opened, unless it is the one already recorded:
/// this runs on every folder switch.
pub(crate) fn save_last_view(view: LastView) {
    let mut s = load_state();
    let account = if view.email.is_empty() {
        s.last_account.clone()
    } else {
        view.email.clone()
    };
    if s.last_view.as_ref() == Some(&view) && s.last_account == account {
        return;
    }
    s.last_view = Some(view);
    s.last_account = account;
    save_state(&s);
}

/// Whether the welcome wizard has been completed before.
pub(crate) fn wizard_completed() -> bool {
    load_state().wizard_completed
}

pub(crate) fn mark_wizard_completed() {
    let mut s = load_state();
    if !s.wizard_completed {
        s.wizard_completed = true;
        save_state(&s);
    }
}

/// The runtime commit WebKit's font fallback was settled under (#296).
pub(crate) fn load_web_font_fallback() -> Option<String> {
    load_state().web_font_fallback.filter(|s| !s.is_empty())
}

pub(crate) fn save_web_font_fallback(runtime: Option<&str>) {
    let mut s = load_state();
    let runtime = runtime.map(str::to_string);
    if s.web_font_fallback != runtime {
        s.web_font_fallback = runtime;
        save_state(&s);
    }
}

/// Whether the one-time Mint keyring setup tip has already been dismissed.
pub(crate) fn mint_keyring_help_dismissed() -> bool {
    load_state().mint_keyring_help_dismissed
}

/// Persist that the user dismissed the Mint keyring setup tip ("Don't show again").
pub(crate) fn dismiss_mint_keyring_help() {
    let mut state = load_state();
    state.mint_keyring_help_dismissed = true;
    save_state(&state);
}

/// Persisted state of the in-message attachment drawer.
#[derive(Debug, Clone, Copy)]
pub(crate) struct DrawerState {
    /// Expanded content height in px.
    pub(crate) height: i32,
    /// Whether the drawer is collapsed to just its header.
    pub(crate) collapsed: bool,
    /// Thumbnail edge in px.
    pub(crate) thumb: i32,
    /// Show an alphabetical list instead of the thumbnail grid.
    pub(crate) list_view: bool,
    /// Sort the list Z→A instead of A→Z.
    pub(crate) sort_desc: bool,
}

impl Default for DrawerState {
    fn default() -> Self {
        Self {
            height: 160,
            collapsed: false,
            thumb: 56,
            list_view: false,
            sort_desc: false,
        }
    }
}

/// Load the attachment drawer's remembered state. The collapsed flag, view
/// settings and dragged height are persisted; thumbnail size always starts at
/// its default.
pub(crate) fn load_drawer_state() -> DrawerState {
    let s = load_state();
    DrawerState {
        collapsed: s.drawer_collapsed,
        list_view: s.drawer_list_view,
        sort_desc: s.drawer_sort_desc,
        height: s.drawer_height.clamp(96, 4000),
        ..DrawerState::default()
    }
}

/// The remembered summary-fetch mode for an account (#226), as
/// (use ENVELOPE, preview items rejected). The default when nothing is known.
pub(crate) fn load_fetch_mode(email: &str) -> (bool, bool) {
    fetch_mode_from_name(
        load_state()
            .fetch_modes
            .get(&email.to_ascii_lowercase())
            .map(String::as_str),
    )
}

/// A fetch mode as state.toml names it, read back; the default for no name
/// or one this build does not know.
pub(super) fn fetch_mode_from_name(name: Option<&str>) -> (bool, bool) {
    match name {
        Some("headers") => (false, false),
        Some("no-previews") => (true, true),
        Some("headers-no-previews") => (false, true),
        _ => (true, false),
    }
}

/// The name state.toml keeps a fetch mode under; `None` for the default,
/// which is stored as no entry at all.
pub(super) fn fetch_mode_name(use_envelope: bool, previews_rejected: bool) -> Option<&'static str> {
    match (use_envelope, previews_rejected) {
        (true, false) => None,
        (false, false) => Some("headers"),
        (true, true) => Some("no-previews"),
        (false, true) => Some("headers-no-previews"),
    }
}

/// Remember the summary-fetch mode that worked for an account (#226); the
/// default is remembered by forgetting.
pub(crate) fn save_fetch_mode(email: &str, use_envelope: bool, previews_rejected: bool) {
    let mode = fetch_mode_name(use_envelope, previews_rejected);
    let key = email.to_ascii_lowercase();
    let mut s = load_state();
    let current = s.fetch_modes.get(&key).map(String::as_str);
    if current == mode {
        return;
    }
    match mode {
        Some(m) => {
            s.fetch_modes.insert(key, m.to_string());
        }
        None => {
            s.fetch_modes.remove(&key);
        }
    }
    save_state(&s);
}

/// Persist whether the attachment drawer is collapsed.
pub(crate) fn save_drawer_collapsed(collapsed: bool) {
    update_state(|s| s.drawer_collapsed = collapsed);
}

/// Persist the attachment drawer's expanded (dragged) height.
pub(crate) fn save_drawer_height(height: i32) {
    update_state(|s| s.drawer_height = height.clamp(96, 4000));
}

/// Persist the attachment drawer's view mode (list vs. thumbnail grid).
pub(crate) fn save_drawer_list_view(list_view: bool) {
    update_state(|s| s.drawer_list_view = list_view);
}

/// Persist the attachment drawer's list sort direction.
pub(crate) fn save_drawer_sort_desc(desc: bool) {
    update_state(|s| s.drawer_sort_desc = desc);
}

/// The About window's height: tall by default, remembering the user's own
/// vertical resize across restarts.
pub(crate) fn load_about_height() -> i32 {
    load_state().about_height.clamp(400, 4000)
}

pub(crate) fn save_about_height(height: i32) {
    update_state(|s| s.about_height = height.clamp(400, 4000));
}

/// The message-list pane's remembered width (clamped to something sane).
pub(crate) fn load_list_pane_width() -> i32 {
    load_state().list_pane_width.clamp(324, 4000)
}

/// Persist the message-list pane's width (#28).
pub(crate) fn save_list_pane_width(width: i32) {
    update_state(|s| s.list_pane_width = width.clamp(324, 4000));
}

/// The list's height with the reader below it (#475).
pub(crate) fn load_list_pane_height() -> i32 {
    load_state().list_pane_height.clamp(120, 4000)
}

pub(crate) fn save_list_pane_height(height: i32) {
    update_state(|s| s.list_pane_height = height.clamp(120, 4000));
}
