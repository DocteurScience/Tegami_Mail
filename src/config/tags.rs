//! Tags (#71): the user's tags, their colors, and tags.toml.

use std::path::PathBuf;

use super::{config_file, read_toml, save_private_toml};

/// A tag (#71): a name and a color for one IMAP keyword. Keywords are the
/// standard's own per-message user flags, kept on the server beside `\Seen`
/// and `\Flagged`, so a tag set here is the same tag Thunderbird, Apple Mail
/// or a webmail shows — and theirs show here once a tag names their keyword
/// (Thunderbird's built-in five are `$label1`…`$label5`). Microsoft 365
/// stores them as categories; POP3 and servers that refuse custom keywords
/// keep them in Tegami Mail's own cache instead.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub(crate) struct Tag {
    /// What the user sees.
    pub(crate) name: String,
    /// The keyword the tag is stored as: an IMAP atom (no spaces, quotes,
    /// parentheses, brackets, backslashes or non-ASCII), case-insensitive.
    pub(crate) keyword: String,
    /// `#rrggbb`.
    pub(crate) color: String,
}

/// The tag palette, GNOME's color set at its middle strength.
pub(crate) const TAG_COLORS: &[&str] = &[
    "#1c71d8", "#2ec27e", "#f5c211", "#e66100", "#c01c28", "#813d9c", "#865e3c", "#77767b",
];

/// A chip's background and text for each palette colour: a light tint of
/// the colour with a dark shade of it for the name (docs/DESIGN.md, "Tags
/// and favourite"). A solid fill with white text fell below 4.5:1 on the
/// yellow, green and orange; these pairs are 7.8:1 or better. In the same
/// order as [`TAG_COLORS`].
pub(crate) const CHIP_COLORS: &[(&str, &str)] = &[
    ("#dce9fb", "#0f3a78"),
    ("#d5f3e4", "#0e5233"),
    ("#fcefbd", "#5a4300"),
    ("#fde0cb", "#6b2d00"),
    ("#f8d7da", "#7a1119"),
    ("#ecdcf3", "#4b2159"),
    ("#eddfd2", "#4d3320"),
    ("#e6e6e8", "#3d3d40"),
];

/// The chip background and text for a tag colour, when it is one of the
/// palette's. A colour picked outside the palette has no tested pair, so
/// the caller keeps the solid fill and picks a readable text for it.
pub(crate) fn chip_colors(color: &str) -> Option<(&'static str, &'static str)> {
    TAG_COLORS
        .iter()
        .position(|c| c.eq_ignore_ascii_case(color))
        .map(|i| CHIP_COLORS[i])
}

impl Tag {
    /// The characters an IMAP keyword (an atom) may hold.
    fn is_keyword_char(c: char) -> bool {
        c.is_ascii_graphic()
            && !matches!(
                c,
                '(' | ')' | '{' | '}' | '"' | '\\' | '%' | '*' | ']' | '['
            )
    }

    /// Whether `s` can be sent to a server as a keyword.
    pub(crate) fn valid_keyword(s: &str) -> bool {
        !s.is_empty() && s.chars().all(Self::is_keyword_char)
    }

    /// The keyword a tag named `name` gets unless the user picks one: the
    /// name with spaces turned to underscores and anything an atom cannot
    /// carry dropped ("To Do" → `To_Do`, "Réunion" → `Runion`).
    pub(crate) fn keyword_for(name: &str) -> String {
        name.trim()
            .chars()
            .map(|c| if c == ' ' { '_' } else { c })
            .filter(|c| Self::is_keyword_char(*c))
            .collect()
    }

    /// The CSS class carrying this tag's color (see the app's tag stylesheet):
    /// the keyword reduced to what a class name may hold.
    pub(crate) fn css_class(&self) -> String {
        tag_css_class(&self.keyword)
    }

    /// The name to offer for a keyword found on a server (the tag finder):
    /// Thunderbird's five built-ins get Thunderbird's names; anything else
    /// sheds a leading `$`, turns `_`/`-` into spaces and capitalises each
    /// word (`travel_plans` → "Travel Plans", `$Receipts` → "Receipts").
    pub(crate) fn name_for_keyword(keyword: &str) -> String {
        match keyword.to_ascii_lowercase().as_str() {
            "$label1" => return "Important".to_string(),
            "$label2" => return "Work".to_string(),
            "$label3" => return "Personal".to_string(),
            "$label4" => return "To Do".to_string(),
            "$label5" => return "Later".to_string(),
            _ => {}
        }
        let bare = keyword.trim_start_matches('$');
        let name: String = bare
            .split(['_', '-'])
            .filter(|w| !w.is_empty())
            .map(|w| {
                let mut cs = w.chars();
                match cs.next() {
                    Some(first) => first.to_uppercase().chain(cs).collect::<String>(),
                    None => String::new(),
                }
            })
            .collect::<Vec<_>>()
            .join(" ");
        if name.is_empty() {
            keyword.to_string()
        } else {
            name
        }
    }

    /// The color to offer for a found keyword: Thunderbird's built-ins get
    /// their Thunderbird colors (as the palette has them); the rest take
    /// the first palette color not in `taken`, cycling once every color is.
    pub(crate) fn color_for_keyword(keyword: &str, taken: &[String]) -> String {
        let fixed = match keyword.to_ascii_lowercase().as_str() {
            "$label1" => Some(TAG_COLORS[4]), // red
            "$label2" => Some(TAG_COLORS[3]), // orange
            "$label3" => Some(TAG_COLORS[1]), // green
            "$label4" => Some(TAG_COLORS[0]), // blue
            "$label5" => Some(TAG_COLORS[5]), // purple
            _ => None,
        };
        if let Some(c) = fixed {
            return c.to_string();
        }
        TAG_COLORS
            .iter()
            .find(|c| !taken.iter().any(|t| t.eq_ignore_ascii_case(c)))
            .copied()
            .unwrap_or(TAG_COLORS[taken.len() % TAG_COLORS.len()])
            .to_string()
    }
}

/// The CSS class for a keyword's color: `tag-` plus the keyword lowercased,
/// with every character a class name can't carry turned into its code so two
/// keywords never share a class.
pub(super) fn tag_css_class(keyword: &str) -> String {
    let mut out = String::from("tag-");
    for c in keyword.to_ascii_lowercase().chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c);
        } else {
            out.push_str(&format!("_{:x}", c as u32));
        }
    }
    out
}

#[derive(Default, serde::Serialize, serde::Deserialize)]
pub(super) struct TagsFile {
    #[serde(default)]
    tags: Vec<Tag>,
}

fn tags_path() -> Option<PathBuf> {
    config_file("tags.toml")
}

pub(crate) fn load_tags() -> Vec<Tag> {
    read_toml::<TagsFile>(tags_path())
        .map(|f| f.tags)
        .unwrap_or_default()
}

pub(crate) fn save_tags(tags: &[Tag]) {
    let file = TagsFile {
        tags: tags.to_vec(),
    };
    save_private_toml(tags_path(), &file, "tags", false);
}
