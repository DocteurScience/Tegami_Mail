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
| Sort and view | the list's end | Sort by: date newest first, date oldest first, sender, subject, size (radio). Show: group into conversations, message preview, avatars (check). |
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

- Whether a dark version of every screen is needed before the first release
  (the colours are named, so it should follow without drawing).
- A tag's server keyword when two accented names collapse to the same atom.
- The icons: the mockups use simple stroke icons. The application should use
  libadwaita's symbolic icons of the same meaning.
- The application icon is a separate piece of work: see [ICON.md](ICON.md).

## Screens

All in [`data/repo/design/`](../data/repo/design/): `01-list.png`,
`02-conversation.png`, `03-conversation-expanded.png`, `04-compose.png`,
`05-reply-with-history.png`, `06-menus.png`, `07-tags-panel.png`,
`08-tags-filter.png`, `09-tags-dialogs.png`, `10-attachments-received.png`,
`11-attachments-sending.png`, `12-attachments-drop.png`,
`13-attachment-forgotten.png`, `14-narrow-1024x600.png`,
`15-phone-width.png`, `16-high-contrast.png`, `17-large-text.png`.
