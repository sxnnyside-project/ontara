use serde::{Deserialize, Serialize};

use crate::calendar::{Calendar, CalendarDate};
use crate::error::DomainError;
use crate::identifier::CalendarId;

/// How precisely an instant is known.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Precision {
    Day,
    Month,
    Year,
    Decade,
    Century,
    Millennium,
}

impl Precision {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Day => "day",
            Self::Month => "month",
            Self::Year => "year",
            Self::Decade => "decade",
            Self::Century => "century",
            Self::Millennium => "millennium",
        }
    }
}

impl std::str::FromStr for Precision {
    type Err = DomainError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        match raw {
            "day" => Ok(Self::Day),
            "month" => Ok(Self::Month),
            "year" => Ok(Self::Year),
            "decade" => Ok(Self::Decade),
            "century" => Ok(Self::Century),
            "millennium" => Ok(Self::Millennium),
            other => Err(DomainError::UnknownValue("precision", other.to_owned())),
        }
    }
}

/// A moment the author tells.
///
/// It always says something: a `label` (the author's own wording, "the winter the river froze"),
/// a `day`, or both. A `day` is the position on the shared timeline and the ordering key; it is
/// optional, so a moment can be told long before anyone can place it. `earliest` and `latest`
/// bound an uncertain day.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Instant {
    pub day: Option<i64>,
    pub precision: Precision,
    pub earliest: Option<i64>,
    pub latest: Option<i64>,
    pub calendar: Option<CalendarId>,
    pub label: Option<String>,
}

impl Instant {
    /// An exact day on the shared timeline.
    #[must_use]
    pub const fn at_day(day: i64) -> Self {
        Self {
            day: Some(day),
            precision: Precision::Day,
            earliest: None,
            latest: None,
            calendar: None,
            label: None,
        }
    }

    /// The instant of a date in a calendar.
    pub fn from_date(calendar: &Calendar, date: CalendarDate) -> Result<Self, DomainError> {
        Ok(Self {
            calendar: Some(calendar.id),
            ..Self::at_day(calendar.universe_day(date)?)
        })
    }

    /// A moment known only by what the author says about it.
    #[must_use]
    pub fn told(label: impl Into<String>) -> Self {
        Self {
            day: None,
            precision: Precision::Day,
            earliest: None,
            latest: None,
            calendar: None,
            label: Some(label.into()),
        }
    }

    /// Whether the moment has a position on the shared timeline.
    #[must_use]
    pub const fn is_placed(&self) -> bool {
        self.day.is_some()
    }

    pub fn validate(&self) -> Result<(), DomainError> {
        let has_label = self
            .label
            .as_deref()
            .is_some_and(|label| !label.trim().is_empty());
        if self.label.is_some() && !has_label {
            return Err(DomainError::BlankText("time label"));
        }
        match self.day {
            None if !has_label => Err(DomainError::InvalidTime(
                "a moment needs a day, a label, or both".to_owned(),
            )),
            None if self.earliest.is_some() || self.latest.is_some() => {
                Err(DomainError::InvalidTime(
                    "bounds only make sense for a moment that has a day".to_owned(),
                ))
            }
            Some(day)
                if self.earliest.is_some_and(|earliest| earliest > day)
                    || self.latest.is_some_and(|latest| latest < day) =>
            {
                Err(DomainError::InvalidTime(
                    "the earliest bound must not be after the day, nor the latest before it"
                        .to_owned(),
                ))
            }
            _ => Ok(()),
        }
    }
}

/// When something happens in the universe.
///
/// Unknown and unconstrained are different statements: an unknown date still has to be found,
/// while an unconstrained record deliberately has no place on the timeline.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Temporal {
    /// It has a date, but the date is not known yet.
    #[default]
    Unknown,
    /// It deliberately has no place on the timeline.
    Unconstrained,
    At {
        instant: Instant,
    },
    /// A span with either end optional; an open end means "not known to end" or "no known start".
    Span {
        start: Option<Instant>,
        end: Option<Instant>,
    },
}

impl Temporal {
    pub fn validate(&self) -> Result<(), DomainError> {
        match self {
            Self::Unknown | Self::Unconstrained => Ok(()),
            Self::At { instant } => instant.validate(),
            Self::Span { start, end } => {
                for instant in [start, end].into_iter().flatten() {
                    instant.validate()?;
                }
                match (start, end) {
                    (Some(start), Some(end)) if start.day > end.day => Err(
                        DomainError::InvalidTime("a span cannot end before it starts".to_owned()),
                    ),
                    (None, None) => Err(DomainError::InvalidTime(
                        "a span needs a start or an end; use unknown or unconstrained instead"
                            .to_owned(),
                    )),
                    _ => Ok(()),
                }
            }
        }
    }

    /// Whether the record has not ended: unknown, unconstrained or a span with no end. A record
    /// that has ended is history.
    #[must_use]
    pub const fn is_open(&self) -> bool {
        match self {
            Self::Unknown | Self::Unconstrained => true,
            Self::At { .. } => false,
            Self::Span { end, .. } => end.is_none(),
        }
    }

    /// Day used to order and range-query records, if the record has a known start.
    #[must_use]
    pub fn start_day(&self) -> Option<i64> {
        match self {
            Self::At { instant } => instant.day,
            Self::Span { start, .. } => start.as_ref().and_then(|instant| instant.day),
            Self::Unknown | Self::Unconstrained => None,
        }
    }

    /// Last day of the record, if it has a known end.
    #[must_use]
    pub fn end_day(&self) -> Option<i64> {
        match self {
            Self::At { instant } => instant.day,
            Self::Span { end, .. } => end.as_ref().and_then(|instant| instant.day),
            Self::Unknown | Self::Unconstrained => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_moment_can_be_told_without_a_day() {
        let told = Instant::told("the winter the river froze");

        assert_eq!(told.validate(), Ok(()));
        assert!(!told.is_placed());
        assert_eq!(Temporal::At { instant: told }.start_day(), None);
    }

    #[test]
    fn a_moment_needs_a_day_or_a_label() {
        let empty = Instant {
            day: None,
            precision: Precision::Day,
            earliest: None,
            latest: None,
            calendar: None,
            label: None,
        };
        assert!(empty.validate().is_err());
        assert!(Instant::told("  ").validate().is_err());
    }

    #[test]
    fn unknown_and_unconstrained_are_distinct() {
        assert_ne!(Temporal::Unknown, Temporal::Unconstrained);
        assert_eq!(Temporal::default(), Temporal::Unknown);
    }

    #[test]
    fn uncertainty_bounds_must_surround_the_day() {
        let mut instant = Instant::at_day(100);
        instant.earliest = Some(90);
        instant.latest = Some(120);
        assert!(instant.validate().is_ok());

        instant.earliest = Some(101);
        assert!(instant.validate().is_err());
    }

    #[test]
    fn spans_cannot_end_before_they_start_or_be_empty() {
        let span = |start: Option<i64>, end: Option<i64>| Temporal::Span {
            start: start.map(Instant::at_day),
            end: end.map(Instant::at_day),
        };

        assert!(span(Some(10), Some(20)).validate().is_ok());
        assert!(span(Some(10), None).validate().is_ok());
        assert!(span(None, Some(20)).validate().is_ok());
        assert!(span(Some(20), Some(10)).validate().is_err());
        assert!(span(None, None).validate().is_err());
    }

    #[test]
    fn ordering_days_are_exposed_for_indexing() {
        let at = Temporal::At {
            instant: Instant::at_day(5),
        };
        let open = Temporal::Span {
            start: Some(Instant::at_day(7)),
            end: None,
        };

        assert_eq!((at.start_day(), at.end_day()), (Some(5), Some(5)));
        assert_eq!((open.start_day(), open.end_day()), (Some(7), None));
        assert_eq!(Temporal::Unknown.start_day(), None);
    }

    #[test]
    fn an_instant_can_come_from_a_calendar_date() -> Result<(), DomainError> {
        let calendar = Calendar::new(
            "Gregorian",
            crate::calendar::gregorian_definition(),
            crate::timestamp::Timestamp::from_unix_millis(0),
        );

        let instant = Instant::from_date(
            &calendar,
            CalendarDate {
                year: 2000,
                month: 1,
                day: 1,
            },
        )?;

        assert_eq!(instant.calendar, Some(calendar.id));
        assert_eq!(
            instant
                .day
                .map(|day| calendar.date_of(day).map(|date| date.year)),
            Some(Ok(2000))
        );
        Ok(())
    }
}
