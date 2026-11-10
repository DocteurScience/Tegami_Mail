# Building from source

Tegami Mail needs the Rust toolchain ([rustup](https://rustup.rs)), a C compiler,
and the GTK 4 / libadwaita / WebKitGTK 6 development libraries, plus
OpenSSL, gettext and Wayland headers. At runtime it needs a Secret Service
provider (e.g. gnome-keyring), and Evolution Data Server
(`evolution-data-server`) for address books: saving a contact, CardDAV
accounts and LDAP directories all go through it. It runs without GNOME;
without it, suggestions still come from your mail.

## Dependencies

**Fedora**

```sh
sudo dnf install gcc gcc-c++ gtk4-devel libadwaita-devel webkitgtk6.0-devel \
    poppler-glib-devel openssl-devel gettext-devel wayland-devel
```

**Debian / Ubuntu**

```sh
sudo apt install build-essential pkg-config libgtk-4-dev libadwaita-1-dev \
    libwebkitgtk-6.0-dev libpoppler-glib-dev libssl-dev gettext libwayland-dev
```

## Build and install

```sh
git clone https://github.com/DocteurScience/Tegami_Mail.git
cd tegami
cargo build --release
./install.sh          # installs the binary, icon and .desktop file into ~/.local
./uninstall.sh        # removes them again (--purge also removes settings and the mail cache)
```

## Translations

`tools/build-locale.sh` compiles `po/*.po` so a source-tree run picks them up;
the Flatpak, the RPM, the Debian package and `install.sh` all do it as part of
their own build.
See [po/README.md](../po/README.md).

## Flatpak

The manifest is `io.github.DocteurScience.TegamiMail.yml`, built with `flatpak-builder` (or
`org.flatpak.Builder`). Its Rust dependencies come from `cargo-sources.json`,
which has to be regenerated whenever `Cargo.lock` changes:

```sh
python3 flatpak-cargo-generator.py Cargo.lock -o cargo-sources.json
```

## AppImage

`tools/build-appimage.sh` produces `packaging/out/Tegami Mail-<arch>.AppImage`, or
`Tegami Mail-Beta-<arch>.AppImage` when the version is a beta, which is then built
with the `beta` feature as the Flatpak beta is (#235). It runs inside an Arch
container (the script starts one with podman; in CI it is already in one),
because the bundle carries every library down to the C library and those have
to be current. The library collecting is [pkgforge's
`quick-sharun`](https://github.com/pkgforge-dev/Anylinux-AppImages), fetched
by the script rather than vendored. `TEGAMI_APPIMAGE_SRC` names another
checkout to build with this script, such as an older tag.

The bundle also carries the spell-check dictionaries and a few fonts with a
fontconfig file of their own (`packaging/appimage/fonts-fallback.conf`), which
the message view falls back to when the system's fonts hang it (#296).

`.github/workflows/build-appimage.yml` builds it for x86_64 and aarch64 on
every tag. Nothing is published from CI: the release attaches the two
AppImages, and Flatout on example.org serves each channel's newest one
with the `.zsync` file AppImage updaters read.

## Your own OAuth client

Google and Dropbox clients can be compiled in at build time; see
[Configuration](DOCUMENTATION.md#oauth-google--microsoft).
