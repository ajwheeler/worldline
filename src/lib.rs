use regex::Regex;
use std::cell::LazyCell;
use std::fs;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct Date {
    year: i32,
    month: u8,
    day: u8,
}

impl Date {
    const MONTH_LENGTHS: [u8; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];

    /// Construct a new Date from year, month, and day.
    /// months and days can be 0 to indicate that they are not known.
    pub fn new(year: i32, month: u8, day: u8) -> Result<Self, String> {
        if month > 12 {
            Err(format!("Invalid month: {}", month))
        } else if month != 0 && day > Self::MONTH_LENGTHS[month as usize - 1] {
            Err(format!("Invalid day: {}", day))
        } else {
            Ok(Self { year, month, day })
        }
    }

    /// Return the date one units of precision (could be days, months, years) higher.
    pub fn next(&self) -> Self {
        if self.day != 0 && self.day < Self::MONTH_LENGTHS[self.month as usize - 1] {
            Self::new(self.year, self.month, self.day + 1).unwrap()
        } else if self.month != 0 && self.month < 12 {
            Self::new(self.year, self.month + 1, 0).unwrap()
        } else {
            Self::new(self.year + 1, 0, 0).unwrap()
        }
    }
}

impl Date {
    /// Construct the regex for parsing dates. Only evaluated once, lazily, for DATE_REGEX.
    fn construct_date_regex() -> Regex {
        let era = r"(?<era>(?i:BCE|BC|CE|AD))?"; // Optional era prefix, case-insensitive
        let year = r"(?<year>-?\d{1,4})"; // Year with optional minus sign
        let month = r"(?:-(?<month>\d{1,2}))?"; // Optional month part. Outer group is non-capturing.
        let day = r"(?:-(?<day>\d{1,2}))?"; // Optional day part. Outer group is non-capturing.
        let pattern = format!(r"^{era}\s*{year}{month}{day}(?:\s+|$)");
        Regex::new(&pattern).unwrap()
    }
    const DATE_REGEX: LazyCell<Regex> = LazyCell::new(Self::construct_date_regex);

    /// Parse a date string into a [year, month, day] array.
    ///
    /// Accepts dates in the following formats:
    /// - BCE/BC dates: "BCE 44" or "-44"
    /// - CE/AD dates: "CE 2023", "2023-12", "2023-12-25"
    ///
    /// Returns Ok(([year, month, day], index)) on success, with month/day set to 0 if not
    ///     specified. index is the index of the first character in the string that was not parsed.
    /// Returns Err with error message on invalid input.
    ///
    /// Note: BCE years are stored as negative numbers, e.g. "BCE 44" -> [-44, 0, 0]
    pub fn parse(date_string: &str) -> Result<(Date, usize), String> {
        let caps = Self::DATE_REGEX
            .captures(date_string)
            .ok_or_else(|| format!("Invalid date format: {}", date_string))?;

        let mut year = caps["year"].parse::<i32>().unwrap();
        if caps
            .name("era")
            .map_or(false, |e| e.as_str().starts_with(['B', 'b']))
        {
            year = -year;
        }

        // safe to unwrap parse because month and day groups are all digits by construction
        // can't use direct indexing into caps because month and day are optional
        let month = caps
            .name("month")
            .map_or(0, |m| m.as_str().parse().unwrap());
        let day = caps.name("day").map_or(0, |d| d.as_str().parse().unwrap());

        // Get the length of the matched substring by finding the end position of the match
        let match_len = caps.get(0).unwrap().end();
        Ok((Date::new(year, month, day)?, match_len))
    }

    /// Format a date into a string for writing to a file.
    pub fn format(&self, display_era: bool) -> String {
        let prefix = if display_era {
            if self.year < 0 {
                "BCE "
            } else {
                " CE "
            }
        } else {
            ""
        };
        let year = self.year.abs().to_string();

        if self.month == 0 {
            format!("{}{:0>4}      ", prefix, year)
        } else if self.day == 0 {
            format!("{}{:0>4}-{:02}   ", prefix, year, self.month)
        } else {
            format!("{}{:0>4}-{:02}-{:02}", prefix, year, self.month, self.day)
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Event {
    pub date: Date,
    pub description: String,
}

impl Event {
    pub fn new(date: Date, description: String) -> Self {
        Self { date, description }
    }

    pub fn parse(event_string: &str) -> Result<Self, String> {
        let (date, index) = Date::parse(event_string)?;
        let description = event_string[index..].to_string();
        Ok(Self::new(date, description))
    }

    pub fn format_for_file(&self) -> String {
        format!("{} {}", self.date.format(true), self.description)
    }

    pub fn format_for_display(&self, display_era: bool) -> String {
        let ansi_reset = "\u{001B}[0m";
        let ansi_blue = "\u{001B}[34m";

        // don't pad year
        format!(
            "{}{}{} {}",
            ansi_blue,
            self.date.format(display_era),
            ansi_reset,
            self.description
        )
    }
}

pub struct WorldLine {
    events: Vec<Event>,
}

impl WorldLine {
    pub fn from_file(file_path: &str) -> Result<Self, String> {
        let events = fs::read_to_string(file_path)
            .map_err(|e| e.to_string())?
            .lines()
            .map(Event::parse)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self { events })
    }

    fn print_for_display(&self, start_idx: usize, end_idx: usize) {
        if self.events[start_idx..end_idx].is_empty() {
            println!("No events");
        } else {
            let show_era =
                self.events[start_idx].date.year < 0 && self.events[end_idx - 1].date.year > 0;
            for event in &self.events[start_idx..end_idx] {
                println!("{}", event.format_for_display(show_era));
            }
        }
    }

    pub fn print_all(&self) {
        self.print_for_display(0, self.events.len());
    }

    /// Find the index of the first event after the given date.
    fn first_geq(&self, date: &Date) -> usize {
        self.events.partition_point(|e| e.date < *date)
    }

    /// Find the index of the last event before the given date.
    fn last_less(&self, date: &Date) -> usize {
        self.events.partition_point(|e| e.date < *date)
    }

    /// Print all events for an implicitly specified date range, e.g.
    ///    1994       -> 1994-01-01 to 1994-12-31 (inclusive)
    ///    1994-05    -> 1994-05-01 to 1994-05-31 (inclusive)
    ///    1994-05-15 -> 1994-05-15 to 1994-05-15 (inclusive)
    pub fn print_implicit_date_range(&self, date: Date) {
        self.print_date_range(date.clone(), date);
    }

    pub fn print_date_range(&self, start: Date, end: Date) {
        let start_idx = self.first_geq(&start);
        let end_idx = self.last_less(&end.next());
        dbg!(&start_idx, &end_idx);
        self.print_for_display(start_idx, end_idx);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_dates() {
        let test_cases = [
            // CE dates
            ("CE 2023", (2023, 0, 0)),
            ("CE 2023-12", (2023, 12, 0)),
            ("CE 2023-12-25", (2023, 12, 25)),
            ("1-2-3", (1, 2, 3)),
            ("AD 2023", (2023, 0, 0)),
            // BCE dates
            ("BCE 44", (-44, 0, 0)),
            ("BC 44", (-44, 0, 0)),
            ("-44", (-44, 0, 0)),
            ("-44-12", (-44, 12, 0)),
            ("-44-12-25", (-44, 12, 25)),
        ];

        for (input, (year, month, day)) in test_cases {
            assert_eq!(
                Date::parse(input).unwrap().0,
                Date::new(year, month, day).unwrap()
            );
        }
    }

    #[test]
    fn test_invalid_dates() {
        assert!(Date::parse("CE").is_err());
        assert!(Date::parse("CE 2023-13").is_err()); // Invalid month
        assert!(Date::parse("CE 2023-12-32").is_err()); // Invalid day
        assert!(Date::parse("CE 2023-01-01-01").is_err()); // hours???
        assert!(Date::parse("invalid").is_err());
    }

    #[test]
    fn test_format_dates() {
        let test_cases = [
            // CE dates
            ((2023, 0, 0), " CE 2023      "),
            ((2023, 12, 0), " CE 2023-12   "),
            ((2023, 12, 25), " CE 2023-12-25"),
            ((1, 2, 3), " CE 0001-02-03"),
            // BCE dates
            ((-44, 0, 0), "BCE 0044      "),
            ((-44, 12, 0), "BCE 0044-12   "),
            ((-44, 12, 25), "BCE 0044-12-25"),
            ((-1, 0, 0), "BCE 0001      "),
        ];

        for ((year, month, day), expected) in test_cases {
            assert_eq!(Date::new(year, month, day).unwrap().format(true), expected);
        }
    }

    #[test]
    fn test_date_next() {
        assert_eq!(
            Date::new(2023, 11, 30).unwrap().next(),
            Date::new(2023, 12, 0).unwrap()
        );
        assert_eq!(
            Date::new(2023, 12, 31).unwrap().next(),
            Date::new(2024, 0, 0).unwrap()
        );
        assert_eq!(
            Date::new(2023, 12, 0).unwrap().next(),
            Date::new(2024, 0, 0).unwrap()
        );
        assert_eq!(
            Date::new(2023, 0, 0).unwrap().next(),
            Date::new(2024, 0, 0).unwrap()
        );
    }

    #[test]
    fn test_parse_events() {
        let test_cases = [
            ("CE 2023 Some event", (2023, 0, 0), "Some event"),
            ("2023-12-25 Christmas Day", (2023, 12, 25), "Christmas Day"),
            ("-44 et tu", (-44, 0, 0), "et tu"),
        ];

        for (input, (year, month, day), desc) in test_cases {
            // TODO test these with an actual test?
            //println!("{}", Event::parse(input).unwrap().format_for_display(true));
            //println!("{}", Event::parse(input).unwrap().format_for_display(false));
            //println!("{}", Event::parse(input).unwrap().format_for_file());

            assert_eq!(
                Event::parse(input).unwrap(),
                Event::new(Date::new(year, month, day).unwrap(), desc.to_string())
            );
        }
    }

    #[test]
    fn test_invalid_events() {
        assert!(Event::parse("").is_err());
        assert!(Event::parse("Invalid date Some event").is_err());
        assert!(Event::parse("CE 2023-13-01 Invalid month").is_err());
    }
}
