//! The GNOME Files (Nautilus) right-click extension (#188): "Send with
//! Tegami Mail" on selected files opens a composer with them attached.
//!
//! Nautilus loads its extensions on the host, outside any sandbox, so the
//! copy bundled here is installed into the user's own extension directory
//! on request (Settings → System → GNOME Files) and removed the same way.
//! The extension itself launches Tegami Mail by desktop id, so it does not care
//! whether the app is the Flatpak or a native package.

use std::path::PathBuf;

/// The extension, verbatim: `data/nautilus/tegami-nautilus.py`.
pub(crate) const SOURCE: &str = include_str!("../../data/nautilus/tegami-nautilus.py");
pub(crate) const FILE_NAME: &str = "tegami-nautilus.py";

/// The per-user extension directory as Nautilus sees it. Inside Flatpak
/// XDG_DATA_HOME is the sandbox's private dir, so the host's default is
/// used (where the manifest mounts `xdg-data/nautilus-python/extensions`).
pub(crate) fn dir() -> Option<PathBuf> {
    let base = if crate::desktop::platform::is_flatpak() {
        crate::desktop::platform::home()?.join(".local/share")
    } else {
        dirs::data_dir()?
    };
    Some(base.join("nautilus-python/extensions"))
}

/// Where the extension reads the path of the AppImage to launch (#235).
fn launcher_record() -> Option<PathBuf> {
    crate::config::config_base().map(|b| b.join("tegami").join("launcher"))
}

/// Leave the extension a way to start Tegami Mail when Tegami Mail is an AppImage.
/// Files launches the app by desktop id, and failing that by name on PATH,
/// and a bundle answers to neither: it installs nothing and sits wherever
/// the user put it. So the path is written down on every start, which also
/// follows a bundle that has been moved or replaced, and cleared whenever
/// Tegami Mail runs as anything else, so a later packaged install does not leave
/// the extension chasing a bundle that is gone.
pub(crate) fn record_launcher() {
    let Some(path) = launcher_record() else {
        return;
    };
    let Some(bundle) = crate::desktop::platform::appimage() else {
        let _ = std::fs::remove_file(&path);
        return;
    };
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    if let Err(e) = std::fs::write(&path, bundle.as_os_str().as_encoded_bytes()) {
        tracing::warn!(
            "could not record the AppImage path at {}: {e}",
            path.display()
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_extension_names_both_builds() {
        assert!(SOURCE.contains("\"io.github.DocteurScience.TegamiMail\""));
        assert!(SOURCE.contains("\"io.github.DocteurScience.TegamiMail.Beta\""));
        assert!(SOURCE.contains("Nautilus.MenuProvider"));
    }
}
