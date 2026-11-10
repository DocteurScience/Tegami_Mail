//! The app icon, and how it reaches the desktop.
//!
//! The app ships one icon (the ink-wash envelope,
//! `data/icons/src/default.svg`; docs/ICON.md); the beta ships its `.Devel`
//! twin, GNOME's development-build styling (the hazard stripe). The gallery
//! of alternative icons is gone (owner's decision, 2026-11-07): at start the
//! files an earlier choice wrote into the user's icon directory
//! (`~/.local/share/icons/hicolor`) are cleared and the launcher is pointed
//! back at the shipped icon, unless somebody set an icon on it outside
//! Tegami Mail (#252).

use std::path::PathBuf;

mod launcher;

pub(crate) use launcher::sync_mime_cache;
use launcher::Launcher;

/// The name the shipped icon is written under where the launcher needs a
/// file of its own (see [`apply`]).
const DEFAULT_ID: &str = "default";
/// The icon this build installs under its app ID.
#[cfg(not(feature = "beta"))]
const DEFAULT_PNG: &[u8] =
    include_bytes!("../../data/icons/hicolor/512x512/apps/io.github.DocteurScience.TegamiMail.png");
#[cfg(feature = "beta")]
const DEFAULT_PNG: &[u8] = include_bytes!(
    "../../data/icons/hicolor/512x512/apps/io.github.DocteurScience.TegamiMail.Beta.png"
);

/// The shipped icon's 512 px artwork.
pub(crate) fn png() -> &'static [u8] {
    DEFAULT_PNG
}

/// The image the tray's App icon option draws: the icon the dock shows.
/// That is the launcher's own icon when somebody set one outside Tegami Mail
/// (#252), a file or a name in the icon theme, and otherwise the shipped
/// one. An icon that cannot be read falls back to the shipped one.
pub(crate) fn tray_image() -> std::borrow::Cow<'static, [u8]> {
    use std::borrow::Cow;
    let Some(icon) = custom_icon() else {
        return Cow::Borrowed(png());
    };
    let path = if std::path::Path::new(&icon).is_absolute() {
        Some(std::path::PathBuf::from(&icon))
    } else {
        // In the desktop's icon folders: Tegami Mail's own search leaves them out.
        crate::desktop::icon_theme::desktop_icon_theme()
            .lookup_icon(
                &icon,
                &[],
                256,
                1,
                gtk::TextDirection::None,
                gtk::IconLookupFlags::empty(),
            )
            .file()
            .and_then(|f| gtk::gio::prelude::FileExt::path(&f))
    };
    match path.and_then(|p| std::fs::read(p).ok()) {
        Some(bytes) => Cow::Owned(bytes),
        None => Cow::Borrowed(png()),
    }
}

/// Bring the desktop in line with the shipped icon at start: the files
/// of an icon picked from the former gallery go, and the launcher points
/// at the shipped icon again, unless it carries an icon set outside Tegami
/// Mail (#252).
pub(crate) fn init_on_startup() {
    if std::env::var("TEGAMI_DEMO").is_ok() {
        return;
    }
    apply();
}

/// The icon the app's launcher names when somebody other than Tegami Mail put it
/// there: a menu editor, a hand edit, an icon theme's name (#252). Tegami Mail
/// only ever writes the app's own icon name or a file of its own (one per
/// former gallery choice), so anything else was the user's doing.
pub(crate) fn custom_icon() -> Option<String> {
    if std::env::var("TEGAMI_DEMO").is_ok() {
        return None;
    }
    Launcher::find().custom_icon()
}

/// Whether a launcher's `Icon=` value is one Tegami Mail puts there: the name the
/// build installs its icon under, or the file a choice is written to.
fn icon_is_ours(value: &str) -> bool {
    if value == crate::APP_ID {
        return true;
    }
    let path = std::path::Path::new(value);
    let prefix = format!("{}-", crate::APP_ID);
    path.is_absolute()
        && path
            .file_name()
            .map(|n| n.to_string_lossy())
            .is_some_and(|n| n.starts_with(&prefix) && n.ends_with(".png"))
}

/// The icon name the shipped icon is written under where the launcher
/// needs a file: `<APP_ID>-default`, as the former gallery named it, so
/// GNOME Shell's per-name cache (it never re-reads a file it has shown)
/// still matches what earlier releases wrote.
fn icon_name() -> String {
    format!("{}-{DEFAULT_ID}", crate::APP_ID)
}

/// The user's share directory as the *desktop* sees it. Inside Flatpak
/// XDG_DATA_HOME is the sandbox's private dir, so the host's default is
/// used (that is where the manifest mounts `xdg-data/icons/hicolor` and
/// `xdg-data/applications`).
fn host_data_home() -> Option<PathBuf> {
    if !crate::desktop::platform::is_flatpak() {
        if let Some(d) = dirs::data_dir() {
            return Some(d);
        }
    }
    Some(crate::desktop::platform::home()?.join(".local/share"))
}

fn hicolor_dir() -> Option<PathBuf> {
    Some(host_data_home()?.join("icons/hicolor"))
}

/// An icon folder only Tegami Mail writes to, which its own icon lookups search in
/// place of the shared ones (`icon_theme::apply_icon_theme`): the chosen app
/// icon is copied here so the window finds it by name.
pub(crate) fn private_icons_root() -> Option<PathBuf> {
    Some(dirs::data_dir()?.join("tegami/icons"))
}

/// Keep the private folder's copy of the icon, and nothing else of ours
/// there.
fn sync_private_icon(name: &str, write: bool) {
    let Some(dir) = private_icons_root().map(|r| r.join("hicolor/512x512/apps")) else {
        return;
    };
    let prefix = format!("{}-", crate::APP_ID);
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let fname = entry.file_name().to_string_lossy().to_string();
            if fname.starts_with(&prefix) && fname != format!("{name}.png") {
                let _ = std::fs::remove_file(entry.path());
            }
        }
    }
    if !write {
        return;
    }
    let path = dir.join(format!("{name}.png"));
    if std::fs::read(&path).is_ok_and(|cur| cur == png()) {
        return;
    }
    if let Err(e) = std::fs::create_dir_all(&dir).and_then(|()| std::fs::write(&path, png())) {
        tracing::warn!("could not keep the app icon in {}: {e}", dir.display());
    }
}

/// The icon sizes the install ships — every one is written under the
/// icon's name, so whichever a desktop asks for resolves to it.
const SIZES: [i32; 2] = [512, 256];

/// Put the icon on the desktop: where the launcher is the user's own copy,
/// the art under its own name in the user's icon directory and the
/// launcher pointed at that file; otherwise the install's launcher
/// restored and our files cleared.
///
/// The launcher names the file by absolute path, never by icon name. The
/// desktop then loads it directly, outside its icon-theme cache — which
/// GNOME Shell only re-scans on its own schedule: a freshly written name
/// stays unknown for seconds, and a removed file keeps resolving to its
/// old path (drawn as a blank) until the next re-scan. A changed path also
/// counts as a changed launcher, so the dock rebuilds the icon at once.
///
/// A launcher whose icon the user set themselves is left as it is, file
/// and all.
fn apply() {
    let name = icon_name();
    let Some(hicolor) = hicolor_dir() else { return };
    let launcher = Launcher::find();
    if let Some(icon) = launcher.custom_icon() {
        tracing::info!("the launcher's icon ({icon}) was set outside Tegami Mail, leaving it");
        // A theme name serves the windows too, where Tegami Mail's icon search
        // finds it; a file path cannot.
        let found = !icon.contains('/')
            && gtk::gdk::Display::default()
                .is_some_and(|d| gtk::IconTheme::for_display(&d).has_icon(&icon));
        gtk::Window::set_default_icon_name(if found { icon.as_str() } else { crate::APP_ID });
        return;
    }
    // No file of its own where the install's launcher can simply be
    // restored (the copy is removed); a launcher the user's own install
    // owns keeps pointing at a file.
    let own_file = matches!(launcher, Launcher::Owned(_));
    sync_icon_files(&hicolor, &name, own_file);
    sync_private_icon(&name, own_file);
    let path = hicolor.join("512x512/apps").join(format!("{name}.png"));
    launcher.point_at(&path.to_string_lossy(), !own_file);
    // Tegami Mail's own windows (X11, panels that draw window icons).
    if own_file {
        gtk::Window::set_default_icon_name(&name);
    } else {
        gtk::Window::set_default_icon_name(crate::APP_ID);
    }
}

/// Write the icon's art (if it is not already there) and clear the art of
/// former gallery choices, then nudge the directories' timestamps so icon
/// caches re-scan them.
fn sync_icon_files(hicolor: &std::path::Path, name: &str, write: bool) {
    let prefix = format!("{}-", crate::APP_ID);
    let mut changed = false;
    for size in SIZES {
        let dir = hicolor.join(format!("{size}x{size}/apps"));
        if let Ok(entries) = std::fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let fname = entry.file_name().to_string_lossy().to_string();
                let stale = fname.starts_with(&prefix)
                    && fname.ends_with(".png")
                    && fname != format!("{name}.png");
                if stale && std::fs::remove_file(entry.path()).is_ok() {
                    changed = true;
                }
            }
        }
        if !write {
            continue;
        }
        let path = dir.join(format!("{name}.png"));
        let png = png();
        let bytes = if size == 512 {
            std::borrow::Cow::Borrowed(png)
        } else {
            match scaled_png(png, size) {
                Some(b) => std::borrow::Cow::Owned(b),
                None => continue,
            }
        };
        if std::fs::read(&path)
            .map(|cur| cur == *bytes)
            .unwrap_or(false)
        {
            continue;
        }
        if let Err(e) = std::fs::create_dir_all(&dir) {
            tracing::warn!("could not create {}: {e}", dir.display());
            continue;
        }
        match std::fs::write(&path, &*bytes) {
            Ok(()) => changed = true,
            Err(e) => tracing::warn!("could not write {}: {e}", path.display()),
        }
    }
    if changed {
        touch(hicolor);
        if let Some(parent) = hicolor.parent() {
            touch(parent);
        }
    }
}

/// The artwork re-encoded at `size`.
fn scaled_png(png: &[u8], size: i32) -> Option<Vec<u8>> {
    use gtk::gdk_pixbuf::{InterpType, PixbufLoader};
    use gtk::prelude::PixbufLoaderExt;
    let loader = PixbufLoader::with_type("png").ok()?;
    loader.write(png).ok()?;
    loader.close().ok()?;
    let pb = loader
        .pixbuf()?
        .scale_simple(size, size, InterpType::Hyper)?;
    pb.save_to_bufferv("png", &[]).ok()
}

fn touch(dir: &std::path::Path) {
    if let Ok(f) = std::fs::File::open(dir) {
        let _ = f.set_modified(std::time::SystemTime::now());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_icons_tegami_writes_count_as_its_own() {
        let id = crate::APP_ID;
        assert!(icon_is_ours(id));
        assert!(icon_is_ours(&format!(
            "/home/u/.local/share/icons/hicolor/512x512/apps/{id}-envelope-wave.png"
        )));
        assert!(icon_is_ours(&format!(
            "/home/u/.local/share/icons/hicolor/512x512/apps/{id}-default.png"
        )));
        // A theme's name, the symbolic, another app's file, a relative path.
        assert!(!icon_is_ours("mail-client"));
        assert!(!icon_is_ours(&format!("{id}-symbolic")));
        assert!(!icon_is_ours("/home/u/icons/tegami-square.svg"));
        assert!(!icon_is_ours(&format!("{id}-default.png")));
    }
}
