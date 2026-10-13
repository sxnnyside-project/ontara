use std::fmt;

use serde::{Deserialize, Serialize};

use crate::error::DomainError;
use crate::identifier::CalendarId;
use crate::record::Record;
use crate::timestamp::Timestamp;

/// Largest year, in either direction, the arithmetic will compute.
const MAX_YEAR: i64 = 100_000;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Month {
    pub label: String,
    pub days: u32,
}

/// Adds `extra_days` to a month in every year the rule selects.
///
/// A year is selected when it is a multiple of `every_years`, unless it is a multiple of
/// `except_every_years`, which `unless_every_years` can cancel. The Gregorian rule is
/// 4 / 100 / 400.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LeapRule {
    pub every_years: u32,
    pub except_every_years: Option<u32>,
    pub unless_every_years: Option<u32>,
    /// One-based month that receives the extra days.
    pub month: u32,
    pub extra_days: u32,
}

impl LeapRule {
    fn applies_to(&self, year: i64) -> bool {
        let divisible = |divisor: u32| year.rem_euclid(i64::from(divisor)) == 0;
        if !divisible(self.every_years) {
            return false;
        }
        match (self.except_every_years, self.unless_every_years) {
            (Some(except), Some(unless)) => !divisible(except) || divisible(unless),
            (Some(except), None) => !divisible(except),
            (None, _) => true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EraDirection {
    /// Year 1 of the era is its first year and numbers grow with time.
    Forward,
    /// Numbers count down towards the next era, like "before" eras.
    Backward,
}

/// A named span of years. An era lasts until the next era's `start_year`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Era {
    pub label: String,
    pub start_year: i64,
    pub direction: EraDirection,
}

/// A day in a specific calendar. Months and days are one-based.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct CalendarDate {
    pub year: i64,
    pub month: u32,
    pub day: u32,
}

/// Everything that defines how a calendar counts days.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CalendarDefinition {
    pub months: Vec<Month>,
    pub leap_rules: Vec<LeapRule>,
    pub weekdays: Vec<String>,
    /// Index into `weekdays` of the first day of year 0.
    pub epoch_weekday: u32,
    pub eras: Vec<Era>,
    /// Universe day of the first day of year 0. Aligns this calendar with the shared timeline.
    pub universe_offset: i64,
}

/// A calendar of the universe. Years are numbered continuously, and year 0 exists.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Calendar {
    pub id: CalendarId,
    pub name: String,
    pub description: String,
    pub is_default: bool,
    pub definition: CalendarDefinition,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

impl Record for Calendar {
    type Id = CalendarId;

    fn id(&self) -> CalendarId {
        self.id
    }
}

impl Calendar {
    #[must_use]
    pub fn new(name: impl Into<String>, definition: CalendarDefinition, now: Timestamp) -> Self {
        Self {
            id: CalendarId::generate(),
            name: name.into(),
            description: String::new(),
            is_default: false,
            definition,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn validate(&self) -> Result<(), DomainError> {
        if self.name.trim().is_empty() {
            return Err(DomainError::BlankText("calendar name"));
        }
        self.definition.validate()
    }

    /// Number of days in `year`, with leap rules applied.
    #[must_use]
    pub fn year_length(&self, year: i64) -> i64 {
        self.definition.year_length(year)
    }

    pub fn universe_day(&self, date: CalendarDate) -> Result<i64, DomainError> {
        self.definition.universe_day(date)
    }

    pub fn date_of(&self, universe_day: i64) -> Result<CalendarDate, DomainError> {
        self.definition.date_of(universe_day)
    }

    /// The weekday label of a universe day.
    pub fn weekday_of(&self, universe_day: i64) -> Result<&str, DomainError> {
        self.definition.weekday_of(universe_day)
    }

    /// The era a year falls in and its number within that era.
    #[must_use]
    pub fn era_of(&self, year: i64) -> Option<(&str, i64)> {
        self.definition.era_of(year)
    }

    /// A readable form such as `3 Frostmoon 312 Golden Era`.
    pub fn describe(&self, date: CalendarDate) -> Result<String, DomainError> {
        let month = self.definition.month(date.month)?;
        Ok(match self.era_of(date.year) {
            Some((era, number)) => format!("{} {} {number} {era}", date.day, month.label),
            None => format!("{} {} {}", date.day, month.label, date.year),
        })
    }
}

impl fmt::Display for CalendarDate {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

impl CalendarDefinition {
    pub fn validate(&self) -> Result<(), DomainError> {
        let invalid = |reason: &str| Err(DomainError::InvalidCalendar(reason.to_owned()));

        if self.months.is_empty() {
            return invalid("a calendar needs at least one month");
        }
        if self
            .months
            .iter()
            .any(|month| month.label.trim().is_empty() || month.days == 0)
        {
            return invalid("every month needs a label and at least one day");
        }
        if self.weekdays.is_empty() || self.weekdays.iter().any(|day| day.trim().is_empty()) {
            return invalid("a calendar needs at least one named weekday");
        }
        let weekday_count = u32::try_from(self.weekdays.len()).unwrap_or(u32::MAX);
        if self.epoch_weekday >= weekday_count {
            return invalid("the epoch weekday is not one of the weekdays");
        }
        let month_count = u32::try_from(self.months.len()).unwrap_or(u32::MAX);
        for rule in &self.leap_rules {
            if rule.every_years == 0 || rule.extra_days == 0 {
                return invalid("a leap rule needs a period and at least one extra day");
            }
            if rule.month == 0 || rule.month > month_count {
                return invalid("a leap rule points at a month that does not exist");
            }
            if rule.except_every_years == Some(0) || rule.unless_every_years == Some(0) {
                return invalid("a leap rule exception needs a period");
            }
            if rule.except_every_years.is_none() && rule.unless_every_years.is_some() {
                return invalid("a leap rule cannot cancel an exception it does not have");
            }
        }
        for pair in self.eras.windows(2) {
            if let [earlier, later] = pair
                && earlier.start_year >= later.start_year
            {
                return invalid("eras must be listed in increasing order of start year");
            }
        }
        if self
            .eras
            .last()
            .is_some_and(|era| era.direction == EraDirection::Backward)
        {
            return invalid("a backward era needs a later era to count down towards");
        }
        if self.eras.iter().any(|era| era.label.trim().is_empty()) {
            return invalid("every era needs a label");
        }
        Ok(())
    }

    fn base_year_length(&self) -> i64 {
        self.months.iter().map(|month| i64::from(month.days)).sum()
    }

    fn year_length(&self, year: i64) -> i64 {
        let extra: i64 = self
            .leap_rules
            .iter()
            .filter(|rule| rule.applies_to(year))
            .map(|rule| i64::from(rule.extra_days))
            .sum();
        self.base_year_length() + extra
    }

    fn month(&self, month: u32) -> Result<&Month, DomainError> {
        usize::try_from(month)
            .ok()
            .and_then(|number| number.checked_sub(1))
            .and_then(|index| self.months.get(index))
            .ok_or_else(|| DomainError::InvalidCalendar(format!("month {month} does not exist")))
    }

    fn month_length(&self, year: i64, month_index: usize) -> i64 {
        let base = self
            .months
            .get(month_index)
            .map_or(0, |month| i64::from(month.days));
        let extra: i64 = self
            .leap_rules
            .iter()
            .filter(|rule| {
                rule.applies_to(year)
                    && usize::try_from(rule.month).ok() == month_index.checked_add(1)
            })
            .map(|rule| i64::from(rule.extra_days))
            .sum();
        base + extra
    }

    fn days_before_year(&self, year: i64) -> i64 {
        if year >= 0 {
            (0..year).map(|y| self.year_length(y)).sum()
        } else {
            -(year..0).map(|y| self.year_length(y)).sum::<i64>()
        }
    }

    fn universe_day(&self, date: CalendarDate) -> Result<i64, DomainError> {
        if date.year.abs() > MAX_YEAR {
            return Err(DomainError::DateOutOfRange);
        }
        self.month(date.month)?;
        let month_index = usize::try_from(date.month - 1).unwrap_or(0);
        if date.day == 0 || i64::from(date.day) > self.month_length(date.year, month_index) {
            return Err(DomainError::InvalidCalendar(format!(
                "day {} does not exist in month {} of year {}",
                date.day, date.month, date.year
            )));
        }
        let days_in_earlier_months: i64 = (0..month_index)
            .map(|index| self.month_length(date.year, index))
            .sum();
        Ok(self.universe_offset
            + self.days_before_year(date.year)
            + days_in_earlier_months
            + i64::from(date.day - 1))
    }

    fn date_of(&self, universe_day: i64) -> Result<CalendarDate, DomainError> {
        let days = universe_day - self.universe_offset;
        let mut year = days.div_euclid(self.base_year_length());
        if year.abs() > MAX_YEAR + 1 {
            return Err(DomainError::DateOutOfRange);
        }
        let mut start = self.days_before_year(year);
        while start > days {
            year -= 1;
            start -= self.year_length(year);
        }
        while start + self.year_length(year) <= days {
            start += self.year_length(year);
            year += 1;
        }
        if year.abs() > MAX_YEAR {
            return Err(DomainError::DateOutOfRange);
        }

        let mut remaining = days - start;
        for (index, _) in self.months.iter().enumerate() {
            let length = self.month_length(year, index);
            if remaining < length {
                return Ok(CalendarDate {
                    year,
                    month: u32::try_from(index + 1).unwrap_or(u32::MAX),
                    day: u32::try_from(remaining + 1).unwrap_or(u32::MAX),
                });
            }
            remaining -= length;
        }
        Err(DomainError::DateOutOfRange)
    }

    fn weekday_of(&self, universe_day: i64) -> Result<&str, DomainError> {
        let count = i64::try_from(self.weekdays.len()).unwrap_or(1);
        let index =
            (i64::from(self.epoch_weekday) + universe_day - self.universe_offset).rem_euclid(count);
        usize::try_from(index)
            .ok()
            .and_then(|position| self.weekdays.get(position))
            .map(String::as_str)
            .ok_or(DomainError::DateOutOfRange)
    }

    fn era_of(&self, year: i64) -> Option<(&str, i64)> {
        let position = self.eras.iter().rposition(|era| era.start_year <= year)?;
        let era = self.eras.get(position)?;
        let number = match era.direction {
            EraDirection::Forward => year - era.start_year + 1,
            EraDirection::Backward => self.eras.get(position + 1)?.start_year - year,
        };
        Some((era.label.as_str(), number))
    }
}

/// A Gregorian-style calendar, used as the starting point of every project.
#[must_use]
pub fn gregorian_definition() -> CalendarDefinition {
    let month = |label: &str, days: u32| Month {
        label: label.to_owned(),
        days,
    };
    CalendarDefinition {
        months: vec![
            month("January", 31),
            month("February", 28),
            month("March", 31),
            month("April", 30),
            month("May", 31),
            month("June", 30),
            month("July", 31),
            month("August", 31),
            month("September", 30),
            month("October", 31),
            month("November", 30),
            month("December", 31),
        ],
        leap_rules: vec![LeapRule {
            every_years: 4,
            except_every_years: Some(100),
            unless_every_years: Some(400),
            month: 2,
            extra_days: 1,
        }],
        weekdays: [
            "Monday",
            "Tuesday",
            "Wednesday",
            "Thursday",
            "Friday",
            "Saturday",
            "Sunday",
        ]
        .iter()
        .map(|name| (*name).to_owned())
        .collect(),
        epoch_weekday: 5,
        eras: Vec::new(),
        universe_offset: 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gregorian() -> Calendar {
        Calendar::new(
            "Gregorian",
            gregorian_definition(),
            Timestamp::from_unix_millis(0),
        )
    }

    fn date(year: i64, month: u32, day: u32) -> CalendarDate {
        CalendarDate { year, month, day }
    }

    /// Two moons, thirteen months of 28 days, with a leap week every seventh year.
    fn lunar() -> Calendar {
        let months = (1..=13)
            .map(|number| Month {
                label: format!("Moon {number}"),
                days: 28,
            })
            .collect();
        let definition = CalendarDefinition {
            months,
            leap_rules: vec![LeapRule {
                every_years: 7,
                except_every_years: None,
                unless_every_years: None,
                month: 13,
                extra_days: 7,
            }],
            weekdays: (1..=7).map(|number| format!("Day {number}")).collect(),
            epoch_weekday: 0,
            eras: vec![
                Era {
                    label: "Before the Fall".to_owned(),
                    start_year: -500,
                    direction: EraDirection::Backward,
                },
                Era {
                    label: "After the Fall".to_owned(),
                    start_year: 0,
                    direction: EraDirection::Forward,
                },
            ],
            universe_offset: 1_000,
        };
        Calendar::new("Lunar", definition, Timestamp::from_unix_millis(0))
    }

    #[test]
    fn gregorian_leap_years_follow_the_century_rule() {
        let calendar = gregorian();

        assert_eq!(calendar.year_length(2024), 366);
        assert_eq!(calendar.year_length(1900), 365);
        assert_eq!(calendar.year_length(2000), 366);
        assert_eq!(calendar.year_length(2023), 365);
    }

    #[test]
    fn leap_day_exists_only_in_leap_years() {
        let calendar = gregorian();

        assert!(calendar.universe_day(date(2024, 2, 29)).is_ok());
        assert!(calendar.universe_day(date(2023, 2, 29)).is_err());
        assert!(calendar.universe_day(date(1900, 2, 29)).is_err());
    }

    #[test]
    fn consecutive_dates_are_consecutive_days_across_year_and_leap_boundaries()
    -> Result<(), DomainError> {
        let calendar = gregorian();

        let feb_28 = calendar.universe_day(date(2024, 2, 28))?;
        assert_eq!(calendar.universe_day(date(2024, 2, 29))?, feb_28 + 1);
        assert_eq!(calendar.universe_day(date(2024, 3, 1))?, feb_28 + 2);
        let new_years_eve = calendar.universe_day(date(2023, 12, 31))?;
        assert_eq!(calendar.universe_day(date(2024, 1, 1))?, new_years_eve + 1);
        Ok(())
    }

    #[test]
    fn dates_round_trip_through_universe_days_including_negative_years() -> Result<(), DomainError>
    {
        for calendar in [gregorian(), lunar()] {
            for year in [-1_200, -501, -500, -1, 0, 1, 6, 7, 2024] {
                let original = date(year, 1, 1);
                let day = calendar.universe_day(original)?;
                assert_eq!(
                    calendar.date_of(day)?,
                    original,
                    "{original} in {}",
                    calendar.name
                );
            }
            let last = date(
                2024,
                calendar.definition.months.len().try_into().unwrap_or(1),
                28,
            );
            assert_eq!(calendar.date_of(calendar.universe_day(last)?)?, last);
        }
        Ok(())
    }

    #[test]
    fn a_custom_calendar_applies_its_own_leap_week() {
        let calendar = lunar();

        assert_eq!(calendar.year_length(1), 13 * 28);
        assert_eq!(calendar.year_length(7), 13 * 28 + 7);
        assert!(calendar.universe_day(date(7, 13, 35)).is_ok());
        assert!(calendar.universe_day(date(8, 13, 35)).is_err());
    }

    #[test]
    fn universe_offset_aligns_calendars_on_the_shared_timeline() -> Result<(), DomainError> {
        let calendar = lunar();

        assert_eq!(calendar.universe_day(date(0, 1, 1))?, 1_000);
        Ok(())
    }

    #[test]
    fn eras_number_years_forward_and_backward() {
        let calendar = lunar();

        assert_eq!(calendar.era_of(10), Some(("After the Fall", 11)));
        assert_eq!(calendar.era_of(-1), Some(("Before the Fall", 1)));
        assert_eq!(calendar.era_of(-500), Some(("Before the Fall", 500)));
        assert_eq!(calendar.era_of(-501), None);
    }

    #[test]
    fn weekdays_cycle_and_describe_uses_the_era() -> Result<(), DomainError> {
        let calendar = lunar();
        let first = calendar.universe_day(date(0, 1, 1))?;

        assert_eq!(calendar.weekday_of(first)?, "Day 1");
        assert_eq!(calendar.weekday_of(first + 8)?, "Day 2");
        assert_eq!(
            calendar.describe(date(12, 3, 9))?,
            "9 Moon 3 13 After the Fall"
        );
        Ok(())
    }

    #[test]
    fn invalid_definitions_are_rejected() {
        let mut definition = gregorian_definition();
        definition.months.clear();
        assert!(definition.validate().is_err());

        let mut definition = gregorian_definition();
        definition.leap_rules[0].month = 13;
        assert!(definition.validate().is_err());

        let mut definition = gregorian_definition();
        definition.eras = vec![Era {
            label: "Dangling".to_owned(),
            start_year: 0,
            direction: EraDirection::Backward,
        }];
        assert!(definition.validate().is_err());
    }

    #[test]
    fn dates_beyond_the_supported_range_are_refused() {
        let calendar = gregorian();

        assert_eq!(
            calendar.universe_day(date(MAX_YEAR + 1, 1, 1)),
            Err(DomainError::DateOutOfRange)
        );
    }
}
