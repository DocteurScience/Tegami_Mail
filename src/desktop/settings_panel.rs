//! Opening a panel of GNOME Settings (Online Accounts, Notifications).

use gtk::glib::prelude::*;

/// Open the GNOME Settings panel `panel` ("notifications",
/// "online-accounts").
///
/// Asked over D-Bus first, through GNOME Settings' `launch-panel` action:
/// that works in a Flatpak (with `--talk-name=org.gnome.Settings`) and from
/// an AppImage, where starting `gnome-control-center` either is not
/// possible or runs it with the bundle's libraries and fails — the
/// Notifications page's link did nothing in the 1.0.0 packages. The command
/// is the fallback for desktops whose Settings answers under no such name.
pub(crate) fn open_settings_panel(panel: &str) {
    if let Err(e) = launch_panel(panel) {
        tracing::debug!("GNOME Settings did not answer on D-Bus: {e}");
        if let Err(e) = std::process::Command::new("gnome-control-center")
            .arg(panel)
            .spawn()
        {
            tracing::warn!("could not open the {panel} settings: {e}");
        }
    }
}

/// The arguments of `org.freedesktop.Application.ActivateAction` for
/// GNOME Settings' `launch-panel`: the action, its parameter `(sav)` (the
/// panel and its extra arguments) wrapped in `av`, and no platform data.
fn launch_panel_args(panel: &str) -> gtk::glib::Variant {
    let parameter = (panel, Vec::<gtk::glib::Variant>::new()).to_variant();
    let platform: std::collections::HashMap<String, gtk::glib::Variant> =
        std::collections::HashMap::new();
    ("launch-panel", vec![parameter], platform).to_variant()
}

fn launch_panel(panel: &str) -> Result<(), gtk::glib::Error> {
    let conn = gtk::gio::bus_get_sync(gtk::gio::BusType::Session, gtk::gio::Cancellable::NONE)?;
    conn.call_sync(
        Some("org.gnome.Settings"),
        "/org/gnome/Settings",
        "org.freedesktop.Application",
        "ActivateAction",
        Some(&launch_panel_args(panel)),
        None,
        gtk::gio::DBusCallFlags::NONE,
        -1,
        gtk::gio::Cancellable::NONE,
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn launch_panel_args_have_the_shape_settings_expects() {
        let args = launch_panel_args("notifications");
        assert_eq!(args.type_().as_str(), "(sava{sv})");
        assert_eq!(
            args.to_string(),
            "('launch-panel', [<('notifications', @av [])>], @a{sv} {})"
        );
    }
}
