//! What a vCard says (its fields, a group's members) and the members of
//! contact groups.

use super::vcard::{
    property_name, split_unescaped, split_vcard_line, unescape_vcard_text, unfold_vcard,
};

// ---------------------------------------------------------------------------
// Full contact details (the in-app Contacts view)
// ---------------------------------------------------------------------------

/// A postal address in the parts of a vCard ADR.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct Postal {
    pub(crate) label: String,
    pub(crate) pobox: String,
    pub(crate) extended: String,
    pub(crate) street: String,
    pub(crate) locality: String,
    pub(crate) region: String,
    pub(crate) code: String,
    pub(crate) country: String,
}

impl Postal {
    fn from_adr(label: String, raw: &str) -> Postal {
        let mut parts = split_unescaped(raw, ';')
            .into_iter()
            .map(|p| unescape_vcard_text(p.trim()));
        let mut next = || parts.next().unwrap_or_default();
        Postal {
            label,
            pobox: next(),
            extended: next(),
            street: next(),
            locality: next(),
            region: next(),
            code: next(),
            country: next(),
        }
    }
}

/// One labelled value on a contact (a "Home" email, a "Mobile" phone…). The
/// label may be empty when the vCard carries no usable TYPE.
#[derive(Debug, Clone)]
pub(crate) struct Labeled {
    pub(crate) label: String,
    pub(crate) value: String,
}

/// A fully parsed address-book entry, everything the in-app Contacts view
/// shows. Read straight from the EDS vCards, so it matches GNOME Contacts.
#[derive(Debug, Clone, Default)]
pub(crate) struct ContactDetails {
    pub(crate) name: String,
    pub(crate) nickname: String,
    pub(crate) org: String,
    pub(crate) title: String,
    pub(crate) birthday: String,
    pub(crate) note: String,
    pub(crate) emails: Vec<Labeled>,
    pub(crate) phones: Vec<Labeled>,
    pub(crate) addresses: Vec<Labeled>,
    /// The same addresses in their parts, for the editor.
    pub(crate) postal: Vec<Postal>,
    /// BDAY as stored ("1985-04-12", "--04-12"…), for the editor;
    /// `birthday` is it in words.
    pub(crate) birthday_raw: String,
    pub(crate) urls: Vec<String>,
    pub(crate) photo: Option<Vec<u8>>,
    /// [`photo_key`] of `photo`, set by [`read_contact_details`]; 0 when
    /// there is none or it was not worked out.
    pub(crate) photo_key: u64,
    /// EDS source UID of the book this contact came from — where edits go.
    pub(crate) book_uid: String,
    /// That book's display name ("On This Computer", the CardDAV account…).
    pub(crate) book_name: String,
    /// The contact's own EDS UID (for deletion and deep links).
    pub(crate) eds_uid: String,
    /// The stored vCard, verbatim; edits patch this so unknown properties
    /// (PHOTO, UID, X-…) survive the round trip.
    pub(crate) raw_vcard: String,
    /// Set for a contact group, with how its vCard keeps the members. A
    /// group has no `emails` of its own: its addresses are the members'.
    pub(crate) group: Option<GroupFormat>,
    pub(crate) members: Vec<Member>,
    /// Members named by the UID of a contact in the same book
    /// (`urn:uuid:`), with the line naming them, until [`resolve_members`]
    /// looks them up.
    pub(crate) member_refs: Vec<(String, String)>,
    /// The lines of members [`resolve_members`] found no address for (a
    /// person with only a phone, a card not synced yet). They are not
    /// shown, and an edit keeps them.
    pub(crate) hidden_members: Vec<String>,
    /// Mail to the group puts its members in Bcc, so they do not see each
    /// other's addresses: Evolution's "Hide addresses when sending mail to
    /// this list".
    pub(crate) hide_addresses: bool,
}

impl ContactDetails {
    /// The address used for avatars and as the compose default.
    pub(crate) fn primary_email(&self) -> Option<&str> {
        self.emails.first().map(|e| e.value.as_str())
    }

    pub(crate) fn is_group(&self) -> bool {
        self.group.is_some()
    }

    /// The members with an address to send to: an Apple group may hold
    /// people known only by a phone number.
    pub(crate) fn reachable(&self) -> impl Iterator<Item = &Member> {
        self.members.iter().filter(|m| !m.email.is_empty())
    }
}

/// How a group's vCard lists its members.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum GroupFormat {
    /// `X-EVOLUTION-LIST:TRUE`, one `EMAIL` per member: what Evolution
    /// makes, and what Tegami Mail writes, so an exported group imports there.
    Evolution,
    /// `X-ADDRESSBOOKSERVER-KIND:group`, one `X-ADDRESSBOOKSERVER-MEMBER`
    /// per member naming a contact of the same book by UID: Apple's, what
    /// iCloud and most CardDAV clients keep.
    Apple,
    /// `KIND:group` (vCard 4), with `MEMBER` lines.
    Kind,
}

/// One person in a contact group.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct Member {
    pub(crate) name: String,
    pub(crate) email: String,
    /// The vCard line the member was read from, unfolded. An edit writes it
    /// back as it was, so what Evolution keeps about the member (the
    /// contact it was picked from, HTML mail) survives. Empty for a member
    /// added here.
    pub(crate) line: String,
    /// The UID of the member's own card in the group's book, for an Apple
    /// group, which names its members by it. Empty when not known yet.
    pub(crate) uid: String,
    /// The card of a contact of another book, copied into an Apple group's
    /// book when they join it ([`moved_vcard`]); empty for someone added by
    /// address, who gets a card with their name and address.
    pub(crate) card: String,
    /// The contact, by book and UID, a member was picked as in the group's
    /// form; the save finds their card by it.
    pub(crate) source: Option<(String, String)>,
}

impl Member {}

/// The human label for a property's TYPE parameters ("" when none applies).
fn vcard_type_label(params: &[&str]) -> String {
    let joined = params.join(";").to_uppercase();
    for (needle, label) in [
        ("CELL", "Mobile"),
        ("HOME", "Home"),
        ("WORK", "Work"),
        ("FAX", "Fax"),
    ] {
        if joined.contains(needle) {
            return label.to_string();
        }
    }
    String::new()
}

/// "1985-04-12" / "19850412" → "Apr 12, 1985"; "--04-12" → "Apr 12", in the
/// locale's words and the user's date order, as mail dates are written;
/// anything else passes through unchanged.
pub(super) fn pretty_birthday(raw: &str) -> String {
    let head = raw.split('T').next().unwrap_or(raw);
    let digits: String = head.chars().filter(|c| c.is_ascii_digit()).collect();
    let (year, month, day) = if head.starts_with("--") && digits.len() == 4 {
        (None, &digits[0..2], &digits[2..4])
    } else if digits.len() == 8 {
        (Some(&digits[0..4]), &digits[4..6], &digits[6..8])
    } else {
        return raw.to_string();
    };
    let (Ok(m), Ok(d)) = (month.parse::<i32>(), day.parse::<i32>()) else {
        return raw.to_string();
    };
    // A birthday with no year is placed in a leap year, so 29 February is a
    // date too. Noon keeps the day the same in every time zone.
    let y = year.and_then(|y| y.parse::<i32>().ok()).unwrap_or(2000);
    let Ok(date) = gtk::glib::DateTime::from_local(y, m, d, 12, 0, 0.0) else {
        return raw.to_string();
    };
    match year {
        Some(_) => crate::datefmt::day_month_year(date.to_unix()),
        None => crate::datefmt::day_month(date.to_unix()),
    }
}

/// [`parse_vcard_details`] without the photo, which costs a base64 decode or
/// a file read: for imports and lookups that never show it.
pub(crate) fn parse_vcard_fields(vcard: &str) -> Option<ContactDetails> {
    let mut c = ContactDetails::default();
    // A list's EMAIL lines are its members, and the line saying so may come
    // after them, so they are kept aside until the card is read.
    let mut email_lines: Vec<(String, Vec<String>, String)> = Vec::new();
    let mut show_addresses = None;
    for line in unfold_vcard(vcard).lines() {
        let Some((prop, raw_value)) = split_vcard_line(line) else {
            continue;
        };
        let name = property_name(prop).to_ascii_uppercase();
        let params: Vec<&str> = prop.split(';').skip(1).collect();
        let value = unescape_vcard_text(raw_value.trim());
        if value.is_empty() {
            continue;
        }
        match name.as_str() {
            "X-EVOLUTION-LIST" if value.eq_ignore_ascii_case("true") => {
                c.group = Some(GroupFormat::Evolution)
            }
            "X-EVOLUTION-LIST-SHOW-ADDRESSES" => {
                show_addresses = Some(value.eq_ignore_ascii_case("true"))
            }
            "X-ADDRESSBOOKSERVER-KIND" if value.eq_ignore_ascii_case("group") => {
                c.group.get_or_insert(GroupFormat::Apple);
            }
            "KIND" if value.eq_ignore_ascii_case("group") => {
                c.group.get_or_insert(GroupFormat::Kind);
            }
            "MEMBER" | "X-ADDRESSBOOKSERVER-MEMBER" => read_member(&mut c, &value, line),
            "UID" if c.eds_uid.is_empty() => c.eds_uid = value,
            "FN" => c.name = value,
            "NICKNAME" if c.nickname.is_empty() => c.nickname = value,
            "ORG" => {
                // ORG is org;department;… — the org alone reads best.
                c.org = value.split(';').next().unwrap_or("").trim().to_string();
            }
            "TITLE" if c.title.is_empty() => c.title = value,
            "BDAY" => {
                c.birthday = pretty_birthday(&value);
                c.birthday_raw = value;
            }
            "NOTE" if c.note.is_empty() => c.note = value,
            "EMAIL" => {
                email_lines.push((
                    line.to_string(),
                    params.iter().map(|p| p.to_string()).collect(),
                    value.clone(),
                ));
                if value.contains('@') && !c.emails.iter().any(|e| e.value == value) {
                    c.emails.push(Labeled {
                        label: vcard_type_label(&params),
                        value,
                    });
                }
            }
            "TEL" => {
                if !c.phones.iter().any(|p| p.value == value) {
                    c.phones.push(Labeled {
                        label: vcard_type_label(&params),
                        value,
                    });
                }
            }
            "URL" => {
                if !c.urls.contains(&value) {
                    c.urls.push(value);
                }
            }
            "ADR" => read_address(&mut c, raw_value, &params),
            _ => {}
        }
    }
    if c.group.is_some() {
        return finish_group(c, show_addresses, email_lines);
    }
    // A card with no name goes by what else it has, as an address book
    // lists it; one with nothing to go by is skipped.
    if c.name.trim().is_empty() {
        c.name = fallback_name(&c)?;
    }
    Some(c)
}

/// A group member line (`MEMBER`, `X-ADDRESSBOOKSERVER-MEMBER`): an
/// address, or a reference to a card by UID to be looked up later.
fn read_member(c: &mut ContactDetails, value: &str, line: &str) {
    let lower = value.to_ascii_lowercase();
    if lower.starts_with("mailto:") {
        let email = value[7..].trim().to_string();
        if email.contains('@')
            && !c
                .members
                .iter()
                .any(|m| m.email.eq_ignore_ascii_case(&email))
        {
            c.members.push(Member {
                email,
                line: line.to_string(),
                ..Member::default()
            });
        }
    } else {
        let uid = if lower.starts_with("urn:uuid:") {
            &value[9..]
        } else {
            value
        };
        c.member_refs
            .push((uid.trim().to_string(), line.to_string()));
    }
}

/// An `ADR` line, as one line of text and in its parts, unless the card
/// has it already.
fn read_address(c: &mut ContactDetails, raw_value: &str, params: &[&str]) {
    // pobox;extended;street;locality;region;postal-code;country.
    let formatted = raw_value
        .split(';')
        .map(|p| unescape_vcard_text(p.trim()))
        .filter(|p| !p.is_empty())
        .collect::<Vec<_>>()
        .join(", ");
    if !formatted.is_empty() && !c.addresses.iter().any(|a| a.value == formatted) {
        c.addresses.push(Labeled {
            label: vcard_type_label(params),
            value: formatted,
        });
        c.postal
            .push(Postal::from_adr(vcard_type_label(params), raw_value.trim()));
    }
}

/// The rest of a group's card once every line is read: its members from
/// Evolution's `EMAIL` lines, and whether mail to it goes in Bcc. A group
/// with no name is no group.
fn finish_group(
    mut c: ContactDetails,
    show_addresses: Option<bool>,
    email_lines: Vec<(String, Vec<String>, String)>,
) -> Option<ContactDetails> {
    // A group is known by its name: it has no address to fall back on.
    if c.name.trim().is_empty() {
        return None;
    }
    if c.group == Some(GroupFormat::Evolution) {
        // Evolution's own default: a list with no say hides its addresses.
        c.hide_addresses = show_addresses != Some(true);
        for (line, params, value) in email_lines {
            let Some((name, email)) = evolution_member(&params, &value) else {
                continue;
            };
            if !c
                .members
                .iter()
                .any(|m| m.email.eq_ignore_ascii_case(&email))
            {
                c.members.push(Member {
                    name,
                    email,
                    line,
                    ..Member::default()
                });
            }
        }
    } else {
        // Other groups have no say of their own; Tegami Mail writes
        // Evolution's line into an Apple group to keep Send as Bcc.
        c.hide_addresses = show_addresses == Some(false);
    }
    c.emails.clear();
    Some(c)
}

/// What a card with no name is listed as: its address, nickname, company
/// or phone number.
fn fallback_name(c: &ContactDetails) -> Option<String> {
    c.primary_email()
        .map(str::to_string)
        .or_else(|| (!c.nickname.is_empty()).then(|| c.nickname.clone()))
        .or_else(|| (!c.org.is_empty()).then(|| c.org.clone()))
        .or_else(|| c.phones.first().map(|p| p.value.clone()))
}

/// The (name, address) of one member of an Evolution list. The value is a
/// recipient ("Ann Lee <ann@x.org>"); Evolution also keeps the parts as
/// parameters, which are the fallback when the value does not parse.
fn evolution_member(params: &[String], value: &str) -> Option<(String, String)> {
    if let Some((name, email)) = crate::worker::parse_recipients(value).into_iter().next() {
        if email.contains('@') {
            return Some((name, email));
        }
    }
    let param = |key: &str| {
        params.iter().find_map(|p| {
            let (k, v) = p.split_once('=')?;
            k.trim()
                .eq_ignore_ascii_case(key)
                .then(|| unescape_vcard_text(v.trim().trim_matches('"')))
        })
    };
    let email = param("X-EVOLUTION-DEST-EMAIL").filter(|e| e.contains('@'))?;
    Some((param("X-EVOLUTION-DEST-NAME").unwrap_or_default(), email))
}

/// Fill in the group members named by contact UID, from `lookup` (UID →
/// that contact's name and first address, empty when they have none, in the
/// group's own book). A member whose card is not there is not shown, but is
/// kept.
pub(super) fn resolve_members(
    c: &mut ContactDetails,
    lookup: impl Fn(&str) -> Option<(String, String)>,
) {
    for (uid, line) in std::mem::take(&mut c.member_refs) {
        let found = lookup(&uid).filter(|(_, email)| {
            email.is_empty()
                || !c
                    .members
                    .iter()
                    .any(|m| m.email.eq_ignore_ascii_case(email))
        });
        match found {
            Some((name, email)) if !c.members.iter().any(|m| m.uid == uid) => {
                c.members.push(Member {
                    name,
                    email,
                    line,
                    uid,
                    ..Member::default()
                });
            }
            _ => c.hidden_members.push(line),
        }
    }
}

/// The (name, first address or "") of a person's vCard, for
/// [`resolve_members`].
pub(super) fn person_address(vcard: &str) -> Option<(String, String)> {
    let c = parse_vcard_fields(vcard).filter(|c| !c.is_group())?;
    let email = c.primary_email().unwrap_or_default().to_string();
    Some((c.name, email))
}
