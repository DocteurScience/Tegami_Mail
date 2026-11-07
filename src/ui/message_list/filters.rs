//! The quick filters (#392 and before) and Gmail's check filters: which
//! messages and conversations they let through.

use crate::i18n::i18n;
use crate::models::Message;

/// One of the message list's quick filters (#97, #441): with it on, the
/// list shows only the mail that has the property. Together they narrow.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum QuickFilter {
    Unread,
    Starred,
    Attachment,
    ToMe,
    Contacts,
    Important,
}

impl QuickFilter {
    pub(crate) const ALL: [QuickFilter; 6] = [
        QuickFilter::Unread,
        QuickFilter::Starred,
        QuickFilter::Attachment,
        QuickFilter::ToMe,
        QuickFilter::Contacts,
        QuickFilter::Important,
    ];

    /// The Filters menu's Messages section (docs/DESIGN.md, "Sorting,
    /// display and filters"): every one but Unread and Starred, which have
    /// the All / Unread / Starred switch above the list, so that each
    /// filter is set in one place. High importance is the owner's choice
    /// to keep (2026-11-07).
    pub(crate) const IN_MENU: [QuickFilter; 4] = [
        QuickFilter::Attachment,
        QuickFilter::ToMe,
        QuickFilter::Contacts,
        QuickFilter::Important,
    ];

    pub(crate) fn label(self) -> String {
        use crate::i18n::i18n_noop;
        i18n(match self {
            QuickFilter::Unread => i18n_noop("Unread"),
            QuickFilter::Starred => i18n_noop("Starred"),
            QuickFilter::Attachment => i18n_noop("Has attachments"),
            QuickFilter::ToMe => i18n_noop("Addressed to me"),
            QuickFilter::Contacts => i18n_noop("From contacts"),
            QuickFilter::Important => i18n_noop("High importance"),
        })
    }

    fn bit(self) -> u16 {
        1 << self as u16
    }
}

/// The Filters menu's Period: one choice, each counted back from today on
/// this computer's clock, or a custom range of days (the custom-period
/// dialog), mail from the start of `from` to the end of `to`, either end
/// left open. It replaced the old popover's Today check and its calendar
/// (owner's decision, 2026-11-07).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) enum Period {
    #[default]
    Any,
    Today,
    Last7Days,
    Last30Days,
    ThisYear,
    Custom {
        from: Option<chrono::NaiveDate>,
        to: Option<chrono::NaiveDate>,
    },
}

impl Period {
    /// The menu's radio choices, in order; Custom is set by its dialog.
    pub(crate) const PRESETS: [Period; 5] = [
        Period::Any,
        Period::Today,
        Period::Last7Days,
        Period::Last30Days,
        Period::ThisYear,
    ];

    /// The name a preset goes by in the menu and on its chip; a custom
    /// range is named by its days ([`Period::range_label`]).
    pub(crate) fn label(self) -> String {
        use crate::i18n::i18n_noop;
        i18n(match self {
            Period::Any => i18n_noop("Any date"),
            Period::Today => i18n_noop("Today"),
            Period::Last7Days => i18n_noop("Last 7 days"),
            Period::Last30Days => i18n_noop("Last 30 days"),
            Period::ThisYear => i18n_noop("This year"),
            Period::Custom { .. } => return self.range_label(),
        })
    }

    /// A custom range in words: "2 Nov 2026 – 7 Nov 2026", "Since …",
    /// "Until …"; days as the list writes dates.
    fn range_label(self) -> String {
        let Period::Custom { from, to } = self else {
            return String::new();
        };
        let (from, to) = (from.map(day_label), to.map(day_label));
        match (from, to) {
            (Some(from), Some(to)) => {
                crate::i18n::i18n_f("{from} to {to}", &[("from", &from), ("to", &to)])
            }
            (Some(date), None) => crate::i18n::i18n_f("Since {date}", &[("date", &date)]),
            (None, Some(date)) => crate::i18n::i18n_f("Until {date}", &[("date", &date)]),
            (None, None) => Period::Any.label(),
        }
    }

    /// The days the period covers, counted from `today`: the first and the
    /// last, either open. A custom range given the wrong way round is read
    /// in order rather than matching nothing.
    pub(crate) fn days(
        self,
        today: chrono::NaiveDate,
    ) -> (Option<chrono::NaiveDate>, Option<chrono::NaiveDate>) {
        use chrono::Datelike;
        let back = |n: u64| today.checked_sub_days(chrono::Days::new(n));
        match self {
            Period::Any => (None, None),
            Period::Today => (Some(today), None),
            Period::Last7Days => (back(6), None),
            Period::Last30Days => (back(29), None),
            Period::ThisYear => (chrono::NaiveDate::from_ymd_opt(today.year(), 1, 1), None),
            Period::Custom {
                from: Some(a),
                to: Some(b),
            } if a > b => (Some(b), Some(a)),
            Period::Custom { from, to } => (from, to),
        }
    }
}

/// A day as the list writes dates, at its noon so no clock change moves it.
pub(crate) fn day_label(day: chrono::NaiveDate) -> String {
    use chrono::TimeZone;
    day.and_hms_opt(12, 0, 0)
        .and_then(|t| chrono::Local.from_local_datetime(&t).earliest())
        .map(|t| crate::datefmt::day_month_year(t.timestamp()))
        .unwrap_or_default()
}

/// The quick filters that are on, and the period the list is narrowed to.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct QuickFilters {
    bits: u16,
    period: Period,
}

impl QuickFilters {
    pub(crate) fn has(self, f: QuickFilter) -> bool {
        self.bits & f.bit() != 0
    }

    pub(crate) fn with(self, f: QuickFilter, on: bool) -> Self {
        Self {
            bits: if on {
                self.bits | f.bit()
            } else {
                self.bits & !f.bit()
            },
            ..self
        }
    }

    pub(crate) fn period(self) -> Period {
        self.period
    }

    pub(crate) fn with_period(self, period: Period) -> Self {
        // A custom range with both ends open is no range.
        let period = match period {
            Period::Custom {
                from: None,
                to: None,
            } => Period::Any,
            p => p,
        };
        Self { period, ..self }
    }

    /// The period as Unix seconds counted from `today`, the end excluded.
    pub(super) fn span(self, today: chrono::NaiveDate) -> (i64, i64) {
        let (from, to) = self.period.days(today);
        (
            from.map_or(i64::MIN, start_of_day),
            to.and_then(|d| d.succ_opt()).map_or(i64::MAX, start_of_day),
        )
    }

    pub(crate) fn is_empty(self) -> bool {
        self.bits == 0 && self.period == Period::Any
    }

    /// How many filters the Filters menu has on: its checks and a period.
    /// Unread and Starred, on the switch above the list, are not counted.
    pub(crate) fn menu_count(self) -> usize {
        QuickFilter::IN_MENU
            .iter()
            .filter(|f| self.has(**f))
            .count()
            + usize::from(self.period != Period::Any)
    }

    /// The filters with Unread and Starred kept and the menu's cleared.
    pub(crate) fn menu_cleared(self) -> Self {
        Self {
            bits: self.bits & (QuickFilter::Unread.bit() | QuickFilter::Starred.bit()),
            period: Period::Any,
        }
    }
}

/// Local midnight of `day`, Unix seconds; an hour on where a clock change
/// skips midnight.
pub(super) fn start_of_day(day: chrono::NaiveDate) -> i64 {
    use chrono::TimeZone;
    let midnight = day.and_time(chrono::NaiveTime::MIN);
    [midnight, midnight + chrono::Duration::hours(1)]
        .iter()
        .find_map(|t| chrono::Local.from_local_datetime(t).earliest())
        .map_or_else(|| midnight.and_utc().timestamp(), |t| t.timestamp())
}

/// What the quick filters need to judge a message, gathered once a pass.
pub(super) struct QuickFilterContext {
    pub(super) filters: QuickFilters,
    /// Every address you send from, lower case ("Addressed to Me").
    pub(super) own: std::collections::HashSet<String>,
    /// Your contacts' addresses; none read yet lets nothing through.
    pub(super) contacts: Option<std::sync::Arc<std::collections::HashSet<String>>>,
    /// The period as Unix seconds, the end excluded.
    pub(super) span: (i64, i64),
}

impl QuickFilterContext {
    pub(super) fn passes(&self, m: &Message) -> bool {
        (self.span.0..self.span.1).contains(&m.timestamp) && self.passes_checks(m)
    }

    fn passes_checks(&self, m: &Message) -> bool {
        QuickFilter::ALL
            .iter()
            .filter(|f| self.filters.has(**f))
            .all(|f| match f {
                QuickFilter::Unread => m.unread,
                QuickFilter::Starred => m.starred,
                QuickFilter::Attachment => m.has_attachment,
                QuickFilter::ToMe => [m.to.as_str(), m.cc.as_str()]
                    .iter()
                    .flat_map(|list| list.split(','))
                    .any(|part| self.own.contains(&bare_address(part))),
                QuickFilter::Contacts => self
                    .contacts
                    .as_ref()
                    .is_some_and(|c| c.contains(&m.from_addr.trim().to_lowercase())),
                QuickFilter::Important => m.importance == crate::models::Importance::High,
            })
    }
}

/// `Name <addr>` or a bare `addr`, as the address alone in lower case.
pub(super) fn bare_address(part: &str) -> String {
    let part = part.trim();
    let addr = match part.rsplit_once('<') {
        Some((_, rest)) => rest.trim_end_matches('>'),
        None => part,
    };
    addr.trim().to_lowercase()
}

/// Today on this computer's clock (the demo's, in the demo: its mail is
/// dated in October 2025).
pub(crate) fn local_today() -> chrono::NaiveDate {
    use chrono::TimeZone;
    chrono::Local
        .timestamp_opt(crate::datefmt::now(), 0)
        .single()
        .map_or_else(|| chrono::Local::now().date_naive(), |t| t.date_naive())
}

/// Which conversations Gmail's * r, * u, * s and * t select (#392).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CheckFilter {
    Read,
    Unread,
    Starred,
    Unstarred,
}

impl CheckFilter {
    /// Whether a conversation, all its members given, is one to select. A
    /// conversation counts as unread, or starred, when any member is, as
    /// its row shows it.
    pub(super) fn matches(self, members: &[Message]) -> bool {
        let unread = members.iter().any(|m| m.unread);
        let starred = members.iter().any(|m| m.starred);
        match self {
            CheckFilter::Read => !unread,
            CheckFilter::Unread => unread,
            CheckFilter::Starred => starred,
            CheckFilter::Unstarred => !starred,
        }
    }
}
