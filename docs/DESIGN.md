# Design

The reference for how Tegami Mail should look and behave. It collects the
decisions taken on the mockups in [`data/repo/design/`](../data/repo/design/),
checked against the [GNOME Human Interface Guidelines](https://developer.gnome.org/hig/).
It describes what to change in the existing interface, not a new application:
the engine, the settings and most widgets stay as they are.

The mockups are in French because they were drawn and reviewed in French. The
text in the code is English, and the French is a translation
([DESIGN_STRINGS.md](DESIGN_STRINGS.md)). Where a mockup shows a keyboard
shortcut that differs from [KEYBOARD_SHORTCUTS.md](KEYBOARD_SHORTCUTS.md), the
mockup is a placeholder and the existing scheme wins.

## Principles

1. **Follow the HIG, and use libadwaita's own widgets, colours and style
   classes.** Custom CSS is kept to the few places listed below, because it
   has to be kept up to date and tested in light, dark, high contrast and
   large text.
2. **One way to do each thing.** An action lives in one place. Where the same
   result can be reached twice (a toolbar button and a menu entry), both do
   exactly the same thing.
3. **Information never rests on colour alone.** A colour is always paired with
   text, an icon or a shape.
4. **Nothing is hidden behind hover.** Every action has a visible control, and
   every icon-only control has a tooltip and an accessible name.
5. **Every string is translatable, and the code can be read by someone who
   did not write it** (see [Code and translations](#code-and-translations)).

## Decisions

Adopted:

- The message list is a column of **cards**, with space between them.
- The accent colour is **the system's accent colour**, not a fixed brand colour.
- Replying always opens **the composer window**, with the conversation history
  below the reply.
- The reply actions live **in the reading pane's header bar only**.
- Messages of a conversation are listed **oldest first, newest last**, each one
  a card.
- **Favourite is a special tag**, the first row of the tags panel. It is the
  IMAP `\Flagged` flag, as before.
- Tags get a header bar button, a sidebar section, and two dialogs.
- Attachments are shown as **cards with a preview** when received, and as a
  list with the total size when sent.
- A message that mentions an attachment but has none asks before it is sent.

Rejected, so that nobody builds them by mistake:

- A quick-reply field under the message. It gave two different ways to reply.
- A reply composer inside the conversation. It cost reading space.
- A standard-row list as an alternative to cards.
- A star button in the header bar, now covered by the tags panel.
- A gauge of the account's size limit (see [Attachments](#attachments)).
- Anything that appears only on hover.

## Window and adaptive layout

Three panes, built with nested `AdwNavigationSplitView`:

| Pane | Width | Holds |
| --- | --- | --- |
| Sidebar | about 220 px | account, folders, tags |
| List | about 380 px | search, filter, message cards |
| Reading pane | the rest | the message or conversation |

- At **1024 x 600**, the smallest desktop size in the HIG, the sidebar folds
  behind a button at the start of the list's header bar.
- At **phone width** (360 px), the panes become a navigation stack: folders,
  then list, then message, each with a back button.
- Widgets that appear or disappear while resizing are animated.
- Lists and the reading text have a maximum width (about 560 px for the text)
  so that large windows do not stretch lines.

Screens: `14-narrow-1024x600.png`, `15-phone-width.png`.

## Header bars

Controls sit at the start, the centre (title) and the end, as the HIG asks. No
button in a main window's header bar uses the suggested or destructive style,
and no button is label-only: buttons are flat, with an icon, and with a label
only when the label earns its place (Reply).

| Pane | Start | Centre | End |
| --- | --- | --- | --- |
| Sidebar | none | Mail | main menu |
| List | New message (icon) | folder name | sort and view menu, search toggle |
| Reading pane | Reply (icon and label), Reply to all, Forward, Archive, Delete | none, left free for dragging | Tags, More actions |

Every button has a tooltip. A tooltip gives the action and, where one exists,
its key.

Search is a search bar that opens under the list's header bar, from the search
button, from the search key, or by typing. It searches all folders.

The filter above the list (All, Unread, Starred) is an `AdwToggleGroup`.

## Message list

One card per message or conversation, in a `GtkListView`.

- Sender (bold when unread), date, an unread dot, subject, an attachment
  icon, a one-line preview.
- Tags as chips below the preview: two at most, then a `+n` chip.
- A conversation shows the participants' names, a messages icon with the
  count, and the same card layout.
- The selected card has a 2 px outline in the accent colour on a ground
  tinted with 10 % of the accent (mockup 01). Under high contrast, a 3 px
  black outline on the card's own ground.
- The unread state is carried by bold text and the dot, not by colour.

Custom CSS for this view is limited to the card (radius 12 px, 1 px border,
8 px between cards) and the tag chip. A starting point, to be checked against
the installed libadwaita:

```css
/* A message card: libadwaita's card colours, a border, and room around it. */
.message-card {
  background-color: @card_bg_color;
  color: @card_fg_color;
  border-radius: 12px;
  border: 1px solid alpha(currentColor, 0.18);
  padding: 12px 14px;
  margin: 0 12px 8px 12px;
}

/* Selected: an outline in the system accent on a lightly tinted ground. */
.message-card:selected {
  background-color: mix(@card_bg_color, @accent_bg_color, 0.1);
  box-shadow: 0 0 0 2px @accent_bg_color;
}

/* Secondary text. libadwaita's .dim-label (55 %) is below 4.5:1 on white;
   70 % of the text colour is about 5:1. */
.secondary-text {
  color: alpha(currentColor, 0.7);
}
```

## Sorting, display and filters

The old sort popover held about twenty controls of three types, with a full
calendar, and covered over half of the window. The HIG asks for the opposite:
popovers "should always be small" (about a third of the window at most) and
"low in complexity", should not mix many control types, and menus hold
three to twelve items, without nested submenus, with actions, check boxes
and radio buttons kept apart. The job is therefore split into three small
places, one purpose each (screens 18 to 21).

| Place | What it holds |
| --- | --- |
| Quick filter, under the search field | The toggle group All, Unread, Starred. It replaces the sorts "Unread first" and "Starred first". |
| Sort and display, the list's end (18) | A single-column popover, about 290 px wide, 7 items. Sort by (radio): date newest first, date oldest first, sender A–Z, subject A–Z. Show (check): group into conversations, message preview, avatars. |
| Filters, a button beside the search field (19) | A menu about 300 px wide. Messages (check): has attachments, addressed to me, from contacts. Period (radio): any date, today, last 7 days, last 30 days, this year. "Custom…" opens a small dialog (21) with two date fields, each with a calendar button. "Clear filters" is disabled when no filter is set. |

Active filters (20): each one is a removable chip under the search field
(its remove button has an accessible name: "Remove filter: …"), "Clear all"
removes them together, the Filters button shows the count, and a status line
under the chips says how many messages match. Nothing relies on colour
alone: radios are circles, check boxes are rounded squares.

Removed from the old popover, as proposed to the owner on 2026-11-07:

- "Unread first" and "Starred first": the quick filter does the job.
- "Tagged": the sidebar already filters by tag.
- The inline calendar: replaced by the custom-period dialog.
- The separate "Today" entry in the dates area: it is now in Period.

The owner approved each removal on 2026-11-07, and kept "High importance"
as a fourth check box in Messages.

## Preferences

The window held about 450 settings on 20 pages: roughly 260 for the
application on 11 pages and 190 for accounts, filters, tags and services.
The GNOME HIG has no page on what preferences hold, and the libadwaita
documentation for `AdwPreferencesDialog` only describes its structure (pages,
groups, search, sub-pages). The rules below come from the way GNOME
applications are built, not from a written rule:

1. A setting exists only if people really differ and no default serves both.
2. Follow the system rather than copy it: style, accent, fonts, text size,
   language, date and time, sounds, browser. Icons are the exception: the
   application ships its own Adwaita symbolic set (see "Removed or moved").
3. No layout settings: this document decides the layout.
4. Troubleshooting tools are not preferences.
5. One action in one place: nothing repeats a control that a menu of the
   list or the reader already has.

The window is an `AdwPreferencesDialog` with the search button on (the
header's search icon), five pages in a view switcher, groups with a title,
rows with a short sentence-case subtitle, and sub-pages for anything longer
than one screen. Screens 22 to 48. The owner decided every removal below on
2026-11-07 and 2026-11-08.

| Page | What it holds | Screens |
| --- | --- | --- |
| Accounts | The accounts, then Organize mail (tags, filters, smart folders, senders) and Services (OpenPGP keys, cloud storage, translation, address books, backup and restore). | 26 to 48 |
| Messages | Reading (mark as read, newest first, confirm deleting a conversation, swipe), composing (send from, format, undo send, spell check). | 22 |
| Notifications | Desktop notifications, show sender and subject, how often to check, a link to the system's notification settings. | 23 |
| Privacy | Remote content (never, ask, always), sender photos and logos, answering read-receipt requests. | 24 |
| Application | Style (follow the system, light, dark), keep running in the background, start at login, single-key shortcuts (with the Gmail variant as an option of this row), keep downloaded mail, large attachments. | 25 |

### Accounts

- The list shows each account with its address, a chevron, and "Add account…"
  that opens a small menu (27): e-mail account, GNOME Online Account, local
  mail on this computer, import from Thunderbird or Evolution (Maildir and
  mbox, moved here from the account page).
- An account page (28) is short: display name, address, colour, show in
  combined views, then one row per sub-page (servers and connection,
  synchronisation, folders, send-as aliases, signatures, OpenPGP), and a
  destructive "Remove account" row at the end. The account's circle in the
  list is always computed from its initials and colour; the picture, emoji and
  Gravatar choices are gone.
- "Servers and connection" (29): receiving, sending, test connection. The
  rare fields (API token, custom OAuth app registration, certificate for
  another name, separate SMTP sign-in) sit under "Advanced". A connection
  test reports its result with an icon and a sentence.
- Synchronisation (30): push, automatic emptying of Junk and Trash, and how long
  downloaded mail is kept ("As the application" by default, set in
  Application). "Move list mail to Trash" is gone. "Include attachments" and
  "Copy mail to On This Computer" stay here (owner, 2026-11-08).
- Folders (31): the five special folders by default on "Automatic"; templates,
  snoozed, reply copies and "server saves its own copy" under "Advanced";
  folder order; hidden folders.
- Aliases (32), signatures (33), OpenPGP key for this account (34): as drawn.
- The Maildir/mbox import rows leave the account page for the Add account
  menu.
- Kept on the account page by the owner (2026-11-08): the nickname, the
  storage in use, the Save button in the header (sub-pages have Back only);
  the provider is under "Advanced" in Servers and connection, over Incoming
  while an account is being added.
- Microsoft 365: keep "Search the organization directory", only for a Microsoft
  account. "Show sensitivity label names" is removed.

### Organize mail and services

- Tags (35, 36): name, colour, order; the first nine have the keys 1 to 9. The
  IMAP keyword and a custom colour are under "Advanced" in the editor.
- Filters (37, 38) stay in full: they are the user's own rules, not preferences.
  The editor moves the rare conditions (body, Reply-To) and "Count unread mail"
  under "Advanced". "Apply now" stays here only.
- Smart folders (39, 40): the sidebar gets one collapsible section "Smart
  folders" right after the unified rows and before the tags, shown only when at
  least one exists. The preview text and the From-without-Reply-To option are
  under "Advanced" in the editor.
- Senders (41): allowed (their remote content loads) and blocked (to Trash).
- OpenPGP keys (42): your keys, correspondents' keys, the GnuPG engine as a
  note at the bottom. The account's key choice is on the account page (34).
- Cloud storage (43, 44): Dropbox stays. Two-factor code and "behind
  Cloudflare" are under "Advanced". Expiry and password for links are set per
  account; the composer reuses them.
- Translation (45): service, key, language to translate into. Server and
  region are under "Advanced". The "Offer to translate" switch is gone: the
  button shows whenever a service is set.
- Address books (46, 47): the books GNOME provides (Evolution Data Server,
  GNOME Online Accounts, CardDAV, LDAP). The Tegami Mail address book is gone,
  with its import, export and "delete all contacts": the application reads
  GNOME Contacts and the provider's contacts. "Show names from Contacts" is
  always on, and has no setting.
- Backup and restore (48): export, and import with a confirmation first. The
  import row says: "Replaces your settings. Keep your accounts in place:
  removing them first deletes their passwords."
- Grammar (in Messages, under spell check, only when a checker exists): one
  switch for the local checker; LanguageTool (server, user name, API key)
  under "Advanced".

### Removed or moved

All of these are features the app has today; the owner agreed to each.

- Appearance: coloured themes (only light, dark, follow the system), the icon
  gallery, the customizable toolbar, "This window opens to", text size (it
  follows the system) and icon set. The icons are the application's own
  Adwaita symbolic set, bundled, whatever icon theme the desktop has: they
  match the mockups, include the application's own glyphs, and cannot be
  mixed with a third-party theme's (owner, 2026-11-10).
- Reading: own font and colours (the system's document font, text size and high
  contrast apply), message cards and header options, find bar position,
  printing options, the actions palette in the cards.
- Message list: columns, layout, preview lines, grouping by date, search and
  selection bar positions, swipe direction and sensitivity, the actions palette
  on hover (it hid actions behind hover).
- Sidebar: the whole page. The design fixes order, sections, chevron, icon rail,
  and the tags above the accounts.
- Conversations and Date and time: the pages. Defaults, or the system.
- System: language (GNOME has none per application), renderer, start
  fullscreen, tray icon, GNOME Files integration, "Send with" default, size
  limit, link opener. Status bar, console mode and "Export log" move to a
  "Troubleshooting" entry of the main menu.
- Notification buttons: always Mark as read, Archive and Delete; Reply,
  Forward and Mark as spam are no longer offered (owner, 2026-11-10).
- Launcher badge: always the unread messages in the inboxes, where a dock
  shows counts (owner, 2026-11-10).
- Start view: the application always opens on the first account's inbox
  (owner, 2026-11-10).
- Existing values: a removed setting keeps its stored value and is simply no
  longer shown, unless the code below it is also removed.

High importance stays in the Filters menu of the list (owner, 2026-11-08,
reversing this list's earlier line).

Open point: the Tegami Mail address book may hold contacts for some users.
Before deleting it, offer once to export them as a vCard file.

## First run

The first-run wizard has six pages in a carousel on a flat saturated blue
(`.welcome-window` in `src/styles.css`): welcome with a language picker, Online
Accounts, e-mail account, privacy, "Make it yours" (preview lines, avatars,
conversation view, icon gallery), done. The blue and the white text on it are
hard to read and ignore the system's colours, light or dark, and accent. Three
of the pages ask for settings that the preferences no longer have, or that
Preferences > Privacy already holds. The owner asked on 2026-11-10 for a new
design: four screens, 49 to 52.

The HIG has no page on first run. Its placeholder pages allow an illustration,
a heading, an optional line of text and a suggested button, with a positive tone
that speaks to the person. The wizard follows that, in the system's colours:

1. **Welcome** (49): the app icon at 128 px, "Welcome to Tegami Mail", one
   sentence, a pill "Get started" (suggested). No language picker: the language
   is the system's.
2. **Add your accounts** (50): a list of the GNOME Online Accounts mail accounts
   found, each with an "Add" button that becomes a check and "Added" once done
   (icon and text, not colour alone). Then "Another account": "Email account…"
   (opens 3) and "Open Online Accounts" (external). One suggested "Continue"
   button. With no account added yet it reads the same; the main window then
   shows its empty state.
3. **Email account** (51): address, password, "Advanced" folded (display name,
   servers, API token, and what the provider needs). The servers are found from
   the address for known providers. While testing, the row shows a status line
   with a spinner and the sentence, and the button is disabled. An error is a
   sentence under the fields, never only a red colour. "Test and add" returns to
   2 on success.
4. **You're all set** (52): the app icon at 96 px, a sentence naming the account
   being synchronised, "Start reading". The issue-tracker link moves to About.

Rules for the build:

- Use `AdwNavigationView` with a back button and a close button in the header
  bar instead of the carousel and its dots. The pages are a sequence, not
  something to browse.
- Use the system colours (`@window_bg_color`, `@card_bg_color`, the accent for the
  suggested button) and delete the `.welcome-*` CSS: nothing in this wizard is
  drawn by hand except the pill shape (`.pill`). The dialog works in light, dark
  and high contrast.
- Remove the language page content, the privacy page and the personalize page.
  The defaults they set (remote content "Ask", no Gravatar or sender logos) are
  those of Preferences > Privacy; Gravatar and sender logos stay off until the
  person turns them on there.
- The wizard opens only when the application has no account, as today, and
  never again after the first account is added or the dialog is closed.
- The text is in the system language only; keep the plural and parameter rules
  of `src/i18n.rs`.

## Reading pane and conversations

- A single message: subject (`title-2`), sender with avatar (`AdwAvatar`), date
  line (`caption`), body, attachments.
- A conversation: the subject and a line such as "3 messages, 3
  participants", then one card per message, oldest first. The newest unread
  message is open and the view is scrolled to it when the conversation opens.
  Older messages are one-line cards (avatar, sender, first words, date) that
  open when activated. A second screen shows every message open.
- An open message card has its own More actions menu (reply to this message,
  reply to all, forward) and nothing else. The reading pane's header bar acts
  on the conversation's latest message.

Screens: `02-conversation.png`, `03-conversation-expanded.png`.

## Replying and composing

Reply, Reply to all, Forward and New message open the **composer window**,
the only place a message is written: the reading pane holds no composer
(owner's decision, 2026-11-03).

The composer window:

- Header bar: Cancel at the start (text, flat); title, with the draft status
  as a caption ("Draft saved at 18:10"); More actions, then **Send** as the
  window's only suggested button, at the end. There is no second Send button
  at the bottom.
- Fields, one per row, label then value: From (a button opening the account
  list), To (recipient chips, each with a remove button at least 32 px square,
  and an autocomplete popover), Cc / Bcc (a button that adds the rows),
  Subject.
- A formatting toolbar, then the body.
- The attachments section (below).
- For a reply, the history sits under the reply, in a section that can be
  hidden. Quoted messages carry a vertical bar at the start and a line such as
  "Camille wrote, on 29 October 2026 at 09:42:". Earlier messages are nested
  one level deeper, and a button shows the previous message.

Screens: `04-compose.png`, `05-reply-with-history.png`.

## Menus

Entries use sentence case, an ellipsis when the entry opens a dialog or needs
more input, and show the key the application already defines (single key when
the single-key scheme is on, otherwise the Ctrl key where there is one). The
keys in the mockups are placeholders.

| Menu | Opened from | Entries |
| --- | --- | --- |
| Main menu | the sidebar's menu button | Sync now. Add account, GNOME Online Accounts. Preferences, Keyboard shortcuts, Help, About. |
| More actions, message | the reading pane's end | Forward as attachment, Mark as unread, Mark as spam, Move to (submenu). Show remote images, View source, Print, Save as. |
| Message context menu | a right click on a card | Reply, Reply to all, Forward. Mark as read, Star, Tags (submenu). Archive, Move to (submenu), Mark as spam. Delete. |
| Sort and display | the list's end | Sort by: date newest first, date oldest first, sender, subject (radio). Show: group into conversations, message preview, avatars (check). See [Sorting, display and filters](#sorting-display-and-filters). |
| Folder context menu | a right click on a folder | Mark all as read, Sync this folder. New subfolder, Rename. Delete folder (destructive style). |
| More actions, composer | the composer's end | Save draft. Format: rich text, plain text (radio). Sign with OpenPGP, Encrypt with OpenPGP, Request a read receipt (check). Check spelling (check), Spelling language (submenu). Delete draft (destructive style). |
| Sending account | the From field | Send from: one row per account with its address, the current one checked. Manage accounts. |
| Attachment | the more button on an attachment | Open, Open with (submenu). Save as, Save all attachments. |

Radio and check entries share one leading column in their menu. Only the
destructive entries use the destructive style.

Screen: `06-menus.png`.

## Tags and favourite

Tags already exist in the engine (`src/config/tags.rs`): a name and a colour
per IMAP keyword, Microsoft 365 categories, Gmail labels, a finder for the
keywords a server already holds. This section changes how they are shown.

- **Favourite** is the first row of the tags panel, with a star, above the
  user's tags. It is the `\Flagged` flag. The "Favourites" folder stays in the
  sidebar.
- **Panel**: opened by the Tags button in the reading pane's header bar. A
  search field that also creates ("Search or create"), one check row per tag
  with its colour dot, then "New tag" and "Manage tags".
- **Chips**: the tag's name always written out, on a tint of its colour.
- **Sidebar**: a "Tags" section, one for every account, with a colour dot,
  the name, the unread count, and a button to add a tag. It sits above the
  accounts by default (owner's decision, 2026-11-04: in sight however many
  accounts there are; mockup 07 draws a single account), and Settings →
  Sidebar can put it below them. Selecting a tag filters the list and
  shows the tag in the list's title.
- **Dialogs**: "New tag" (name, colour with its name under each swatch, a
  preview chip, Cancel and Create) and "Tags" (a boxed list with edit and
  delete buttons, a button row to add one, and a row to import tags from the
  servers).

The eight colours are the engine's palette. Chip colours below were chosen
for a text contrast of 7.8:1 or better:

| Colour | Dot | Chip background | Chip text |
| --- | --- | --- | --- |
| Blue | `#1c71d8` | `#dce9fb` | `#0f3a78` |
| Green | `#2ec27e` | `#d5f3e4` | `#0e5233` |
| Yellow | `#f5c211` | `#fcefbd` | `#5a4300` |
| Orange | `#e66100` | `#fde0cb` | `#6b2d00` |
| Red | `#c01c28` | `#f8d7da` | `#7a1119` |
| Purple | `#813d9c` | `#ecdcf3` | `#4b2159` |
| Brown | `#865e3c` | `#eddfd2` | `#4d3320` |
| Gray | `#77767b` | `#e6e6e8` | `#3d3d40` |

Under high contrast, chips are white with a black border and the name.

Screens: `07-tags-panel.png`, `08-tags-filter.png`, `09-tags-dialogs.png`.

## Attachments

**Received.** Under the body, a section "3 attachments" with the total size and
a "Save all" button. Each attachment is a card: a preview (the image, the
first page of a PDF, a video or file icon), the name, the type and size, a
Save as button and a More actions button, both always visible. Activating the
preview opens the lightbox that already exists. An executable file (for
example a script or an installer) shows a written warning with an icon.

**Sent.** In the composer, a section "Attachments" with the file count and the
total size, then one card per file with a preview and a remove button. A file
over about 10 MB shows "Large: some servers will refuse it" and a button that
uploads it to the cloud account and sends a link instead. The drop surfaces
that already exist (attach, insert in the text, share a link) stay.

**Size limit: not shown.** `lettre` does not expose the SIZE value a server
announces, and reading it would mean a second, hand-written connection with
TLS. The value is also only an upper bound: quotas and filters can refuse a
smaller message. So the interface shows the total and a generic warning, and
does not draw a limit.

**Refused for size.** The failure path already exists: a permanent SMTP
refusal (`smtp_refused`) returns the message to the composer. Today it shows
"Send failed:" followed by the server's own text. Recognise a 552 reply, or
the enhanced code 5.3.4, and show a plain sentence ("The server refused the
message because it is too large") with the server's text as detail.

**Forgotten attachment.** Before sending, if the body (excluding quoted
history and the signature) contains a word such as "attached", "attachment",
"enclosed" and the message has no attachment, an `AdwAlertDialog` asks "Send
without an attachment?" with Cancel, Send Anyway and Attach a File (the
suggested response). The words are a translatable list, one per language, as
Thunderbird does. Quoted text must be skipped, or a reply to a message that
did have an attachment would trigger it.

Screens: `10-attachments-received.png`, `11-attachments-sending.png`,
`12-attachments-drop.png`, `13-attachment-forgotten.png`.

## Colour, type and spacing

Use libadwaita's named colours (`@window_bg_color`, `@view_bg_color`,
`@card_bg_color`, `@headerbar_bg_color`, `@sidebar_bg_color`,
`@accent_bg_color`) so that dark, high contrast and the accent choice follow
the system. The values in the mockups are the light theme, approximately.

Type uses the standard styles, never a fixed size: `body` for text, `heading`
for the window title and group headings, `caption` for dates, sizes and hints,
`title-2` for a message subject, `title-3` or `title-4` for dialog headings.
Two weights only, regular and bold. No italics, no capitals.

Spacing and shape: buttons and entries 6 px radius, cards and popovers 12 px,
controls 34 px high, header bars at libadwaita's default height, 8 px or
12 px between related items and 16 px or 24 px around content.

Contrast, computed on the light theme:

| Pair | Ratio | Note |
| --- | --- | --- |
| Text on white | 12.6:1 | passes |
| Secondary text at 70 % | 4.5 to 5.4:1, by background | passes |
| Secondary text at libadwaita's default 55 % | 3.2:1 | fails 4.5:1 |
| White on the blue, teal or green accent | about 3.8:1 | fails 4.5:1 for small text |
| White on the purple accent | 5.9:1 | passes |

The accent follows the system, so the white-on-accent ratio is libadwaita's to
fix. Keep text on accent buttons bold, and do not put small text on an accent
fill.

## Accessibility

Test each screen with the checklist from the HIG:

- High contrast (`16-high-contrast.png`): borders in black, selection
  inverted, no shadows.
- Large text at 150 % (`17-large-text.png`): control heights grow with the
  text and nothing is cut off. Long names truncate with an ellipsis.
- Keyboard only: every control reachable, with the keys of the existing
  scheme.
- Screen reader: each icon-only button has an accessible name, each dot has a
  text alternative ("Unread", "Attachment").
- On-screen keyboard: every entry works with it.

## Code and translations

- Every user-visible string is English in the code and goes through the
  existing gettext functions, as described in [po/README.md](../po/README.md):
  `i18n`, `i18n_f` with named placeholders, `ni18n` for plurals. A count such as
  "3 attachments" always uses the plural form.
- Reuse an existing string when one fits. [DESIGN_STRINGS.md](DESIGN_STRINGS.md)
  lists the strings the design needs, with a proposed French text and whether
  the catalogue already has them.
- After adding strings, run `tools/update-pot.sh`, then update `po/fr.po`. The
  other languages fall back to English until their translators update their
  files; the template marks the new strings.
- The French catalogue translates "Tags" as "Tags" today. The design uses
  "Étiquettes", which is also Thunderbird's French term.
- Comments explain why, not what. Each module starts with a `//!` paragraph
  saying what it owns and what it leaves to others, as `src/i18n.rs` does. A
  function whose behaviour is not obvious from its name gets a `///` comment
  with the reason for the choices in it. Cite the issue number for a
  workaround. State the invariant next to any `unsafe` block. Add a test
  beside a bug fix.

## Small changes the design depends on

- **Tag keyword from an accented name.** `Tag::keyword_for` drops any
  character an IMAP atom cannot carry, so "Réunion" becomes `Runion` on the
  server. Replace accented letters by their base letter first ("Reunion").
  The displayed name is not affected.
- **Size refusal text.** As described under Attachments.
- **Favourite in the tags panel.** The panel shows the `\Flagged` state as its
  first row and toggles it.
- **Forgotten attachment check.** New, local to the composer's send path.

## Open questions

- A dark version of the screens is not needed (owner, 2026-11-11): the
  colours are named, so dark follows without drawing.
- The application icon is a separate piece of work: see [ICON.md](ICON.md).

## Screens

All in [`data/repo/design/`](../data/repo/design/): `01-list.png`,
`02-conversation.png`, `03-conversation-expanded.png`, `04-compose.png`,
`05-reply-with-history.png`, `06-menus.png`, `07-tags-panel.png`,
`08-tags-filter.png`, `09-tags-dialogs.png`, `10-attachments-received.png`,
`11-attachments-sending.png`, `12-attachments-drop.png`,
`13-attachment-forgotten.png`, `14-narrow-1024x600.png`,
`15-phone-width.png`, `16-high-contrast.png`, `17-large-text.png`,
`18-sort-display.png`, `19-filters-menu.png`, `20-filters-active.png`,
`21-custom-period.png`, then the preferences, `22-prefs-messages.png` to
`48-prefs-backup.png`: the five pages and every sub-page and editor of Accounts
(see Preferences above for what each one is); then the first-run wizard,
`49-welcome.png` to `52-welcome-done.png` (see First run).
