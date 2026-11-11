//! Opening GNOME Contacts.

use adw::prelude::*;

use crate::i18n::i18n;

/// One way of starting GNOME Contacts, tried in [`launch_gnome_contacts`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Way {
    /// D-Bus activation of `org.gnome.Contacts`, a DBusActivatable
    /// GApplication: from a Flatpak (no `gnome-contacts` on its path), an
    /// AppImage (whose libraries a child would inherit) and a native build.
    Activate,
    /// snapd's desktop launcher (`desktop-launch` plug), which opens a
    /// desktop entry of the host: a strictly confined snap may neither talk
    /// to `org.gnome.Contacts` nor run a host program.
    SnapLauncher,
    /// Running `gnome-contacts`, where it is on the path.
    Command,
}

/// The ways to try, in order. No `flatpak-spawn --host` in a Flatpak:
/// reaching it needs `--talk-name=org.freedesktop.Flatpak`, a "run anything
/// on the host" permission not worth holding for a button.
fn ways(flatpak: bool, snap: bool) -> &'static [Way] {
    match (flatpak, snap) {
        (true, _) => &[Way::Activate],
        (false, true) => &[Way::SnapLauncher, Way::Activate],
        (false, false) => &[Way::Activate, Way::Command],
    }
}

/// Launch the GNOME Contacts application of the desktop.
pub(crate) fn launch_gnome_contacts() {
    let snap = std::env::var_os("SNAP").is_some_and(|v| !v.is_empty());
    for way in ways(crate::desktop::platform::is_flatpak(), snap) {
        let tried = match way {
            Way::Activate => activate_host_contacts(),
            Way::SnapLauncher => open_desktop_entry_from_snap(),
            Way::Command => run_gnome_contacts(),
        };
        match tried {
            Ok(()) => return,
            Err(e) => tracing::debug!("GNOME Contacts, {way:?}: {e}"),
        }
    }
    tracing::warn!("could not open GNOME Contacts");
}

fn run_gnome_contacts() -> Result<(), Box<dyn std::error::Error>> {
    let app = gtk::gio::AppInfo::create_from_commandline(
        "gnome-contacts",
        Some(i18n("Contacts").as_str()),
        gtk::gio::AppInfoCreateFlags::NONE,
    )?;
    app.launch(&[], gtk::gio::AppLaunchContext::NONE)?;
    Ok(())
}

/// The desktop entries GNOME Contacts goes by: installed with the system or
/// as a Flatpak, and as a snap.
const CONTACTS_ENTRIES: [&str; 2] = [
    "org.gnome.Contacts.desktop",
    "gnome-contacts_gnome-contacts.desktop",
];

/// Ask snapd's user daemon to open GNOME Contacts' desktop entry
/// (`io.snapcraft.PrivilegedDesktopLauncher.OpenDesktopEntry`, allowed by
/// the `desktop-launch` plug).
fn open_desktop_entry_from_snap() -> Result<(), Box<dyn std::error::Error>> {
    let conn = zbus::blocking::Connection::session()?;
    let mut last: Option<zbus::Error> = None;
    for entry in CONTACTS_ENTRIES {
        match conn.call_method(
            Some("io.snapcraft.Launcher"),
            "/io/snapcraft/PrivilegedDesktopLauncher",
            Some("io.snapcraft.PrivilegedDesktopLauncher"),
            "OpenDesktopEntry",
            &(entry,),
        ) {
            Ok(_) => return Ok(()),
            Err(e) => last = Some(e),
        }
    }
    Err(last.map_or_else(|| "no entry".into(), |e| e.into()))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_package_tries_the_ways_it_is_allowed() {
        assert_eq!(ways(true, false), &[Way::Activate]);
        assert_eq!(ways(false, true), &[Way::SnapLauncher, Way::Activate]);
        assert_eq!(ways(false, false), &[Way::Activate, Way::Command]);
    }
}
