//! Opening GNOME Contacts.

use adw::prelude::*;

use crate::i18n::i18n;

/// Launch the GNOME Contacts application via the desktop session.
pub(crate) fn launch_gnome_contacts() {
    // Inside a Flatpak sandbox `gnome-contacts` isn't on the path, so a plain
    // exec fails. Reach the host app by D-Bus-activating it (it's a
    // DBusActivatable GApplication).
    if crate::desktop::platform::is_flatpak() {
        if let Err(e) = activate_host_contacts() {
            // No `flatpak-spawn --host` fallback: reaching it needs
            // --talk-name=org.freedesktop.Flatpak, which is a general-purpose
            // "run anything on the host" permission and not worth holding for a
            // button. D-Bus activation is the supported path.
            tracing::warn!("could not activate GNOME Contacts: {e}");
        }
        return;
    }
    if let Ok(app) = gtk::gio::AppInfo::create_from_commandline(
        "gnome-contacts",
        Some(i18n("Contacts").as_str()),
        gtk::gio::AppInfoCreateFlags::NONE,
    ) {
        let _ = app.launch(&[], gtk::gio::AppLaunchContext::NONE);
    }
}

/// Activate the host `org.gnome.Contacts` service over the session bus (the
/// standard `org.freedesktop.Application.Activate` a GApplication exposes).
fn activate_host_contacts() -> Result<(), Box<dyn std::error::Error>> {
    let conn = zbus::blocking::Connection::session()?;
    let platform_data: std::collections::HashMap<String, zbus::zvariant::Value> =
        std::collections::HashMap::new();
    conn.call_method(
        Some("org.gnome.Contacts"),
        "/org/gnome/Contacts",
        Some("org.freedesktop.Application"),
        "Activate",
        &(platform_data,),
    )?;
    Ok(())
}
