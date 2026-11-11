# Tegami Mail

An email client for the GNOME desktop, written in Rust with GTK 4 and
libadwaita. It is a redesign of [Hylki](https://github.com/hyprlab/hylki):
the same mail engine, with a calmer, simpler interface.

![Tegami Mail, with a conversation open beside the message list](data/screenshots/main.png)

## Features

- **Your accounts:** IMAP, POP3, JMAP and Microsoft 365, with Microsoft
  sign-in built in, Google through GNOME Online Accounts, or any account
  already set up in GNOME; folders kept on this computer, and mail imported
  from Thunderbird or Evolution.
- **Reading:** each conversation as a column of cards; the reading pane
  beside the list, below it, or off; meeting invitations answered in place,
  read receipts, one-click unsubscribe, translation.
- **Writing:** rich or plain text, signatures, templates, inline images,
  spell and grammar checking, Undo Send, and large files shared through your
  own Nextcloud, ownCloud, OpenCloud, Seafile, OneDrive or Dropbox.
- **Organising:** folders, filters, tags stored on the server as IMAP
  keywords, smart folders and favorites, Snooze.
- **Privacy:** no telemetry, remote content blocked until you allow it,
  passwords kept in the system keyring; OpenPGP signing and encryption, and
  DKIM, SPF and DMARC checked on arrival.
- **GNOME-native:** follows the system's style, accent colour, text size,
  language and clock; GNOME Contacts, CardDAV and LDAP address books;
  notifications with actions and mail that keeps arriving with the window
  closed.
- **Translated** into Catalan, French, German, Greek, Hungarian, Italian,
  Polish, Portuguese (Brazil and Portugal), Russian and Spanish.

See [docs/FEATURES.md](docs/FEATURES.md) for the full list and
[docs/DOCUMENTATION.md](docs/DOCUMENTATION.md) for how each part works.

## Install

Download the latest release:

- **[Flatpak](https://github.com/DocteurScience/Tegami_Mail/releases/latest/download/tegami-mail.flatpak)**,
  then install it with
  `flatpak install --user tegami-mail.flatpak` (the GNOME runtime comes
  from Flathub, which `flatpak install` asks for).
- **[AppImage](https://github.com/DocteurScience/Tegami_Mail/releases/latest/download/Tegami.Mail-x86_64.AppImage)**
  for x86_64: make it executable (`chmod +x Tegami.Mail-x86_64.AppImage`)
  and run it.
- **Snap:** the `.snap` file of the release, once attached, installed with
  `sudo snap install --dangerous tegami-mail_*.snap`. A snap installed this
  way, outside the Snap Store, gets only the permissions snapd grants
  everyone; connect the others it needs (the keyring, GNOME Online
  Accounts and GNOME Contacts, OpenPGP, USB drives):

  ```bash
  for p in password-manager-service online-accounts-service contacts-service evolution-files desktop-launch gpg-keys gpg-public-keys removable-media; do sudo snap connect tegami-mail:$p; done
  ```

Every release, with its notes, is on the
[releases page](https://github.com/DocteurScience/Tegami_Mail/releases).
To build it yourself, see [docs/BUILDING.md](docs/BUILDING.md).

## Contributing

Bug reports and ideas are welcome in the
[issue tracker](https://github.com/DocteurScience/Tegami_Mail/issues). See
[docs/CONTRIBUTING.md](docs/CONTRIBUTING.md) before sending changes, and
[po/README.md](po/README.md) to help with a translation.

## Licence

AGPL-3.0-or-later, like Hylki. Where the code comes from, its copyrights and
the third-party components are listed in [NOTICE](NOTICE); Hylki's own README
is kept in [UPSTREAM_README.md](UPSTREAM_README.md).
