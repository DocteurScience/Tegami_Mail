# Tegami Mail documentation

- **[FEATURES.md](FEATURES.md):** everything the app does.
- **[DOCUMENTATION.md](DOCUMENTATION.md):** setting up accounts and OAuth,
  cloud attachments, OpenPGP, the GNOME Files entry, and what Tegami Mail
  keeps on disk.
- **[KEYBOARD_SHORTCUTS.md](KEYBOARD_SHORTCUTS.md):** the single-key scheme.
- **[DESIGN.md](DESIGN.md):** how the interface should look and behave, with
  the screens it refers to; [DESIGN_STRINGS.md](DESIGN_STRINGS.md) lists the
  text it needs and its French translation.
- **[BUILDING.md](BUILDING.md):** building from source.
- **[CONTRIBUTING.md](CONTRIBUTING.md):** how to help.
- **[CREDITS.md](CREDITS.md):** who has.
- **[RELEASE_NOTES.md](RELEASE_NOTES.md):** what every release brought, in
  the words the release page and About use.
- **[SECURITY.md](SECURITY.md):** reporting a vulnerability privately.
- **[LICENSE.md](LICENSE.md):** the licence and third-party notices.

Also at the top of the repository: [CHANGELOG.md](../CHANGELOG.md), and
[po/README.md](../po/README.md) for translators.

`python3 tools/check-docs.py` checks that all of it still hangs together:
links, anchors, this index, and where each kind of thing belongs
([CONTRIBUTING.md](CONTRIBUTING.md#documentation)).
