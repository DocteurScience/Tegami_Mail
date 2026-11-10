//! The app's look outside the mail: light or dark, the colors that follow
//! the light/dark flip, and reduced motion. The icon theme is in
//! [`crate::desktop::icon_theme`].

use adw::prelude::*;

use crate::config;

/// Force (or release) the app-wide color scheme per the appearance
/// preference — the whole chrome, not just message content, which has its own
/// setting.
pub(super) fn apply_app_theme(theme: config::AppTheme) {
    let scheme = match theme {
        config::AppTheme::System => adw::ColorScheme::Default,
        config::AppTheme::Light => adw::ColorScheme::ForceLight,
        config::AppTheme::Dark => adw::ColorScheme::ForceDark,
    };
    adw::StyleManager::default().set_color_scheme(scheme);
}

/// Styles that branch on the color scheme, which static CSS cannot do: a
/// dedicated provider (above the static stylesheet's priority) carries the
/// scheme-dependent values and reloads whenever the scheme flips.
///
/// - The message list's selection pill: the alpha that reads right on dark is
///   heavy on a light ground, so light mode runs 25% lighter.
/// - The remote-content banner's shield: amber on dark, a deeper orange on
///   light where amber washes out.
/// - The Refresh button while offline (#442): orange, a step deeper on light.
/// - The composer's ground: the reader's deeper page shade, read from the
///   live theme (#148) rather than the stock GNOME values, so a custom theme
///   carries through to the composer like it does to the reader.
///
/// `window` is only where the theme's named colors are read from.
pub(super) fn install_scheme_css(window: &impl IsA<gtk::Widget>) {
    let provider = gtk::CssProvider::new();
    if let Some(display) = gtk::gdk::Display::default() {
        gtk::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION + 1,
        );
    }
    let window = window.clone().upcast::<gtk::Widget>();
    let apply = move |provider: &gtk::CssProvider, dark: bool| {
        // The shield over blocked remote content, in GNOME's warning orange
        // or yellow: the deeper step on light, the lighter one on dark.
        let shield = if dark { "#ffca28" } else { "#ff7800" };
        // GNOME's orange: its lighter step on dark, its deeper one on light.
        let offline = if dark { "#ffa348" } else { "#e66100" };
        // The compose surface sits on the reader's deeper page ground — the
        // same shade the threaded cards float on, as the theme defines it.
        let (_, page, _) = crate::ui::message_view::theme_grounds_for(&window, dark);
        provider.load_from_string(&format!(
            ".remote-alert image {{ color: {shield}; }}\
             .refresh-offline {{ color: {offline}; }}\
             .compose-pane {{ background-color: {page}; }}\
             .compose-windowed .compose-pane {{ background-color: @view_bg_color; }}"
        ));
    };
    let style = adw::StyleManager::default();
    apply(&provider, style.is_dark());
    style.connect_dark_notify(move |sm| {
        // The theme's named colors are re-resolved after this signal, not
        // before it: read them now and the lookup answers for the scheme
        // just left (a light-grey composer in dark mode). Apply once the
        // main loop comes round, as the reader does through its own message.
        let dark = sm.is_dark();
        let provider = provider.clone();
        let apply = apply.clone();
        gtk::glib::idle_add_local_once(move || apply(&provider, dark));
    });
}

/// High contrast (docs/DESIGN.md, "Accessibility"; mockup 16): borders in
/// the text colour, selection inverted, no tint standing for anything. The
/// accent becomes the text colour, so every outline, dot and suggested
/// button that wears it turns black (white on dark) without a rule each,
/// whatever light colour the system accent is.
///
/// Loaded from libadwaita's own high-contrast flag rather than a
/// `prefers-contrast` media query: the flag follows GNOME's setting and
/// `ADW_DEBUG_HIGH_CONTRAST` alike, which the media query does not on
/// every GTK the app runs on. Above the appearance theme's provider, so a
/// theme's accent never wins over it.
pub(crate) const HIGH_CONTRAST_CSS: &str = "\
@define-color accent_bg_color @window_fg_color;\
@define-color accent_color @window_fg_color;\
@define-color accent_fg_color @window_bg_color;\
:root { --accent-bg-color: var(--window-fg-color); --accent-color: var(--window-fg-color);\
  --accent-fg-color: var(--window-bg-color); }\
.message-row { border-color: @window_fg_color; }\
.message-list > row:selected > .message-item:not(.swiping) .message-row {\
  background-color: @card_bg_color; border-color: @window_fg_color;\
  box-shadow: inset 0 0 0 2px @window_fg_color; }\
.navigation-sidebar > row:selected, .navigation-sidebar > row:selected:hover {\
  background-color: @window_fg_color; color: @window_bg_color; }\
toggle-group > toggle:checked { background-color: @window_fg_color; color: @window_bg_color; }\
.tag-chip { background-color: @window_bg_color; color: @window_fg_color;\
  border: 1px solid @window_fg_color; }\
.tag-chip label { color: @window_fg_color; }\
.tag-chip > .tag-dot { background-color: @window_fg_color; }\
.account-circle { background-color: @window_bg_color; color: @window_fg_color;\
  box-shadow: inset 0 0 0 1.5px @window_fg_color; }\
.account-circle, avatar { outline-color: @window_fg_color; }\
";

/// Load [`HIGH_CONTRAST_CSS`] while libadwaita draws high contrast, and
/// follow the setting as it changes.
pub(super) fn install_high_contrast_css() {
    let provider = gtk::CssProvider::new();
    if let Some(display) = gtk::gdk::Display::default() {
        gtk::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION + 4,
        );
    }
    let apply = move |on: bool| provider.load_from_string(if on { HIGH_CONTRAST_CSS } else { "" });
    let style = adw::StyleManager::default();
    apply(style.is_high_contrast());
    style.connect_high_contrast_notify(move |sm| apply(sm.is_high_contrast()));
}

/// GNOME Shell turns animations off in every app while a remote desktop
/// session is on, to spare the connection; that is not the person asking
/// for less motion. Where the app can read GNOME's own Reduce Animation
/// switch (through dconf, which the Flatpak has no access to), Tegami Mail's
/// animations follow that switch instead, now and whenever it changes.
pub(super) fn follow_motion_setting() {
    use gtk::gio;
    if gio::SettingsBackend::default().type_().name() != "DConfSettingsBackend" {
        return;
    }
    let readable = gio::SettingsSchemaSource::default()
        .and_then(|s| s.lookup("org.gnome.desktop.interface", true))
        .is_some_and(|s| s.has_key("enable-animations"));
    let Some(gtk_settings) = gtk::Settings::default().filter(|_| readable) else {
        return;
    };
    // The binding holds the settings object for as long as GTK's lives.
    gio::Settings::new("org.gnome.desktop.interface")
        .bind("enable-animations", &gtk_settings, "gtk-enable-animations")
        .flags(gio::SettingsBindFlags::GET)
        .build();
}
