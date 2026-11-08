//! The choices the settings rows offer, in the order the rows list them,
//! and the conversions between a stored value and a row's index.

use super::*;

pub(super) const APP_THEMES: &[(&str, AppTheme)] = &[
    (i18n_noop("Follow the system"), AppTheme::System),
    (i18n_noop("Light"), AppTheme::Light),
    (i18n_noop("Dark"), AppTheme::Dark),
];

/// The Language row's choices: (label, locale code). The system's own
/// first, then English (the source language), then every catalogue in
/// po/LINGUAS by its own name (#179).
pub(crate) fn language_choices() -> Vec<(String, String)> {
    let mut out = vec![
        (i18n("System"), String::new()),
        ("English".to_string(), "en".to_string()),
    ];
    for code in include_str!("../../../po/LINGUAS").lines() {
        let code = code.trim();
        if code.is_empty() || code.starts_with('#') || code == "en" {
            continue;
        }
        out.push((native_language_name(code).to_string(), code.to_string()));
    }
    out
}

/// A language's name in itself, for the Language row.
fn native_language_name(code: &str) -> &str {
    match code {
        "ca" => "Català",
        "fr" => "Français",
        "el" => "Ελληνικά",
        "hu" => "Magyar",
        "ru" => "Русский",
        "de" => "Deutsch",
        "es" => "Español",
        "it" => "Italiano",
        "pt" => "Português",
        "pt_PT" => "Português (Portugal)",
        "pt_BR" => "Português (Brasil)",
        "nl" => "Nederlands",
        "pl" => "Polski",
        "cs" => "Čeština",
        "sv" => "Svenska",
        "da" => "Dansk",
        "nb" | "no" => "Norsk",
        "fi" => "Suomi",
        "tr" => "Türkçe",
        "uk" => "Українська",
        "ja" => "日本語",
        "zh_CN" => "简体中文",
        "zh_TW" => "繁體中文",
        "ko" => "한국어",
        other => other,
    }
}

/// Selectable mail-check intervals (label, seconds). 0 = manual only.
pub(super) const FETCH_INTERVALS: &[(&str, u64)] = &[
    (i18n_noop("Manually"), 0),
    (i18n_noop("Every minute"), 60),
    (i18n_noop("Every 5 minutes"), 300),
    (i18n_noop("Every 15 minutes"), 900),
    (i18n_noop("Every 30 minutes"), 1800),
];

/// The large-attachment policy at row `idx` (Ask, Attach, Cloud).
pub(super) fn files_large_at(idx: u32) -> crate::config::FilesLarge {
    use crate::config::FilesLarge as L;
    match idx {
        1 => L::Attach,
        2 => L::Cloud,
        _ => L::Ask,
    }
}

pub(super) fn files_large_index(l: crate::config::FilesLarge) -> u32 {
    use crate::config::FilesLarge as L;
    match l {
        L::Ask => 0,
        L::Attach => 1,
        L::Cloud => 2,
    }
}

/// The choices of Settings → System → Keep downloaded mail, in days; 0 keeps
/// it (#394).
pub(super) const KEEP_DOWNLOADS_DAYS: [u32; 4] = [0, 365, 90, 30];

/// The row of the read mark combo that shows `value`.
pub(super) fn read_mark_index(value: crate::config::ReadMark) -> u32 {
    use crate::config::ReadMark;
    match value {
        ReadMark::Shown => 0,
        ReadMark::Delay => 1,
        ReadMark::Manual => 2,
    }
}

/// The read mark value at row `idx` of its combo; past the end reads as the first.
pub(super) fn read_mark_at(idx: u32) -> crate::config::ReadMark {
    use crate::config::ReadMark;
    match idx {
        1 => ReadMark::Delay,
        2 => ReadMark::Manual,
        _ => ReadMark::Shown,
    }
}

/// The remote-content row (Never, Ask, Always) for the two stored
/// switches it stands for: loading it always, and warning when it is
/// blocked. Blocked without a warning is Never; blocked with one is Ask.
pub(super) fn remote_content_index(auto_load: bool, warn_blocked: bool) -> u32 {
    match (auto_load, warn_blocked) {
        (true, _) => 2,
        (false, true) => 1,
        (false, false) => 0,
    }
}

/// The two switches a remote-content row stands for: (load always, warn
/// when blocked). Always leaves the warning on, for the day it is turned
/// back to Ask.
pub(super) fn remote_content_at(idx: u32) -> (bool, bool) {
    match idx {
        0 => (false, false),
        2 => (true, true),
        _ => (false, true),
    }
}

/// Whether the photos-and-logos switch shows on: either service in use
/// sends something out, so it reads on until both are off.
pub(super) fn photos_on(gravatar: bool, logos: bool) -> bool {
    gravatar || logos
}

/// The row of the read receipts combo that shows `value` (#520).
pub(super) fn receipts_index(value: crate::config::ReceiptPolicy) -> u32 {
    use crate::config::ReceiptPolicy;
    match value {
        ReceiptPolicy::Ask => 0,
        ReceiptPolicy::Always => 1,
        ReceiptPolicy::Never => 2,
    }
}

/// The row of the returned receipts combo that shows `value` (#520).
/// The read-receipt policy at row `idx` (Ask, Always, Never).
pub(super) fn receipts_at(idx: u32) -> crate::config::ReceiptPolicy {
    use crate::config::ReceiptPolicy;
    match idx {
        1 => ReceiptPolicy::Always,
        2 => ReceiptPolicy::Never,
        _ => ReceiptPolicy::Ask,
    }
}

pub(super) fn returned_receipts_index(value: crate::config::ReturnedReceipts) -> u32 {
    use crate::config::ReturnedReceipts;
    match value {
        ReturnedReceipts::Trash => 0,
        ReturnedReceipts::Archive => 1,
        ReturnedReceipts::Keep => 2,
    }
}

/// The returned receipts value at row `idx` of its combo; past the end
/// moves to Trash.
pub(super) fn returned_receipts_at(idx: u32) -> crate::config::ReturnedReceipts {
    use crate::config::ReturnedReceipts;
    match idx {
        1 => ReturnedReceipts::Archive,
        2 => ReturnedReceipts::Keep,
        _ => ReturnedReceipts::Trash,
    }
}

/// The row of the compose format combo that shows `value`.
pub(super) fn compose_format_index(value: crate::config::ComposeFormat) -> u32 {
    use crate::config::ComposeFormat;
    match value {
        ComposeFormat::Rich => 0,
        ComposeFormat::Plain => 1,
    }
}

/// The compose format value at row `idx` of its combo; past the end reads as the first.
pub(super) fn compose_format_at(idx: u32) -> crate::config::ComposeFormat {
    use crate::config::ComposeFormat;
    match idx {
        1 => ComposeFormat::Plain,
        _ => ComposeFormat::Rich,
    }
}

/// The row of the signature position combo that shows `value`.
pub(super) fn signature_position_index(value: crate::config::SignaturePosition) -> u32 {
    use crate::config::SignaturePosition;
    match value {
        SignaturePosition::AboveQuote => 0,
        SignaturePosition::BelowQuote => 1,
    }
}

/// The signature position value at row `idx` of its combo; past the end reads as the first.
pub(super) fn signature_position_at(idx: u32) -> crate::config::SignaturePosition {
    use crate::config::SignaturePosition;
    match idx {
        1 => SignaturePosition::BelowQuote,
        _ => SignaturePosition::AboveQuote,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn language_choices_start_with_system_and_english_then_linguas() {
        let choices = language_choices();
        assert_eq!(choices[0], ("System".to_string(), String::new()));
        assert_eq!(choices[1], ("English".to_string(), "en".to_string()));
        let codes: Vec<&str> = include_str!("../../../po/LINGUAS")
            .lines()
            .map(str::trim)
            .filter(|c| !c.is_empty() && !c.starts_with('#') && *c != "en")
            .collect();
        let listed: Vec<&str> = choices[2..].iter().map(|(_, c)| c.as_str()).collect();
        assert_eq!(listed, codes);
        for (name, code) in &choices[2..] {
            assert_eq!(name, native_language_name(code));
        }
    }

    #[test]
    fn languages_are_named_in_themselves() {
        assert_eq!(native_language_name("fr"), "Français");
        assert_eq!(native_language_name("pt_BR"), "Português (Brasil)");
        assert_eq!(native_language_name("nb"), "Norsk");
        assert_eq!(native_language_name("no"), "Norsk");
        assert_eq!(native_language_name("xx"), "xx");
    }

    #[test]
    fn files_choices_map_to_combo_rows() {
        use crate::config::FilesLarge;
        for (i, v) in [FilesLarge::Ask, FilesLarge::Attach, FilesLarge::Cloud]
            .into_iter()
            .enumerate()
        {
            assert_eq!(files_large_index(v), i as u32);
            assert_eq!(files_large_at(i as u32), v);
        }
    }

    /// Screen 24 merges the remote-content switches without losing what
    /// was stored: each stored pair reads as one choice, and each choice
    /// writes a pair that reads back as itself.
    #[test]
    fn remote_content_choices_round_trip() {
        assert_eq!(remote_content_index(true, false), 2);
        assert_eq!(remote_content_index(true, true), 2);
        assert_eq!(remote_content_index(false, true), 1);
        assert_eq!(remote_content_index(false, false), 0);
        for idx in 0..3 {
            let (auto, warn) = remote_content_at(idx);
            assert_eq!(remote_content_index(auto, warn), idx);
        }
        assert!(photos_on(true, false) && photos_on(false, true));
        assert!(!photos_on(false, false));
    }

    #[test]
    fn choice_tables_keep_their_order() {
        let themes: Vec<_> = APP_THEMES.iter().map(|(_, v)| *v).collect();
        assert_eq!(themes, [AppTheme::System, AppTheme::Light, AppTheme::Dark]);
        let secs: Vec<u64> = FETCH_INTERVALS.iter().map(|(_, s)| *s).collect();
        assert_eq!(secs, [0, 60, 300, 900, 1800]);
    }

    #[test]
    fn enum_rows_round_trip_and_fall_back_to_the_first() {
        use crate::config::*;
        for v in [ReadMark::Shown, ReadMark::Delay, ReadMark::Manual] {
            assert_eq!(read_mark_at(read_mark_index(v)), v);
        }
        assert_eq!(read_mark_at(9), ReadMark::Shown);
        use crate::config::ReceiptPolicy;
        for v in [
            ReceiptPolicy::Ask,
            ReceiptPolicy::Always,
            ReceiptPolicy::Never,
        ] {
            assert_eq!(receipts_at(receipts_index(v)), v);
        }
        assert_eq!(receipts_at(9), ReceiptPolicy::Ask);
        for v in [
            ReturnedReceipts::Trash,
            ReturnedReceipts::Archive,
            ReturnedReceipts::Keep,
        ] {
            assert_eq!(returned_receipts_at(returned_receipts_index(v)), v);
        }
        assert_eq!(returned_receipts_at(9), ReturnedReceipts::Trash);
        for (i, v) in [ComposeFormat::Rich, ComposeFormat::Plain]
            .into_iter()
            .enumerate()
        {
            assert_eq!(compose_format_index(v), i as u32);
            assert_eq!(compose_format_at(i as u32), v);
        }
        assert_eq!(compose_format_at(9), ComposeFormat::Rich);
        for v in [SignaturePosition::AboveQuote, SignaturePosition::BelowQuote] {
            assert_eq!(signature_position_at(signature_position_index(v)), v);
        }
        assert_eq!(signature_position_at(9), SignaturePosition::AboveQuote);
    }
}
