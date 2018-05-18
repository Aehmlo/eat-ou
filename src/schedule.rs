use serde::de::Error as DeserializationError;
use serde::de::{Deserialize, Deserializer};
use std::{
    cmp::{Ordering, PartialOrd}, error::Error, fmt, ops::{Add, Sub}, str::FromStr,
};

/// Represents a low-resolution point in time, relative to midnight.
#[derive(Clone, Copy, Deserialize, PartialEq)]
pub struct Time {
    hours: u8,
    #[serde(default)]
    minutes: u8,
}

impl Time {
    /// Creates a new `Time` with the given hours and minutes past midnight.
    pub fn new(hours: i32, minutes: i32) -> Self {
        Self {
            hours: hours as u8,
            minutes: minutes as u8,
        }
    }

    /// Creates a new `Time` with the given hours past midnight.
    pub fn with_hours(hours: u8) -> Self {
        Self { hours, minutes: 0 }
    }
}

/// Represents an error encountered while converting from a string to a `Time`.
#[derive(Debug)]
pub enum FromStrError {
    /// No colon was present in the string.
    MissingColon,
    /// After splitting on colons, too few (<2) components were present.
    InsufficientComponents,
    /// After splitting on colons, too many (>2) components were present.
    ///
    /// This is likely due to multiple colons.
    ExtraComponents,
    /// Another error occurred.
    Generic,
}

impl fmt::Display for FromStrError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "Invalid time string.")
    }
}

impl Error for FromStrError {}

impl FromStr for Time {
    type Err = FromStrError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !s.contains(":") {
            return Err(FromStrError::MissingColon);
        }
        let parts = s.split(":")
            .map(|c| c.parse::<u8>().unwrap_or_default())
            .collect::<Vec<u8>>();
        match parts.len() {
            0..2 => Err(FromStrError::InsufficientComponents),
            2 => Ok(Self {
                hours: parts[0],
                minutes: parts[1],
            }),
            _ => Err(FromStrError::ExtraComponents),
        }
    }
}

fn deserialize_time<'de, D>(deserializer: D) -> Result<Time, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Time::from_str(&s).map_err(DeserializationError::custom)
}

impl Add<u8> for Time {
    type Output = Time;
    fn add(self: Time, rhs: u8) -> Self::Output {
        let mut minutes = self.minutes + rhs;
        let mut hours = self.hours;
        if minutes > 60 {
            hours += 1;
            minutes -= 60;
        }
        if hours > 47 {
            hours -= 48;
        }
        Time {
            hours: hours,
            minutes: minutes,
        }
    }
}

impl Sub<Time> for Time {
    type Output = usize;
    fn sub(self: Time, rhs: Time) -> Self::Output {
        let minutes = self.minutes - rhs.minutes;
        let hours = self.hours - rhs.hours;
        (hours as usize) * 60 + (minutes as usize)
    }
}

impl PartialOrd for Time {
    // TODO: Handle times past midnight
    fn partial_cmp(&self, other: &Time) -> Option<Ordering> {
        if self.hours == other.hours && self.minutes == other.minutes {
            Some(Ordering::Equal)
        } else if self.hours > other.hours
            || (self.hours == other.hours && self.minutes > other.minutes)
        {
            Some(Ordering::Greater)
        } else {
            Some(Ordering::Less)
        }
    }
}

impl fmt::Display for Time {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let mut hours = self.hours;
        let mut pm = false;
        if hours > 24 {
            hours -= 24;
        }
        if hours > 12 {
            hours -= 12;
            pm = true;
        }
        if hours == 12 {
            pm = !pm;
        }
        if hours == 0 {
            hours = 12;
        }
        write!(
            f,
            "{}:{:02} {}",
            hours,
            self.minutes,
            if pm { "PM" } else { "AM" }
        )
    }
}

/// Represents a day of the week.
#[derive(Clone, Copy, Deserialize)]
pub enum Day {
    Sunday,
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
}

impl From<i32> for Day {
    fn from(index: i32) -> Day {
        match index {
            0 => Day::Sunday,
            1 => Day::Monday,
            2 => Day::Tuesday,
            3 => Day::Wednesday,
            4 => Day::Thursday,
            5 => Day::Friday,
            _ => Day::Saturday,
        }
    }
}

#[derive(Deserialize, Clone)]
struct HoursMap {
    sunday: Option<Hours>,
    monday: Option<Hours>,
    tuesday: Option<Hours>,
    wednesday: Option<Hours>,
    thursday: Option<Hours>,
    friday: Option<Hours>,
    saturday: Option<Hours>,
}

/// Represents the times that a business is open.
#[derive(Deserialize, Clone, Copy)]
pub struct Hours {
    #[serde(deserialize_with = "deserialize_time")]
    start: Time,
    #[serde(deserialize_with = "deserialize_time")]
    end: Time,
}

impl fmt::Display for Hours {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let (start, end) = (format!("{}", self.start), format!("{}", self.end));
        if start == end {
            write!(f, "Open 24 hours")
        } else {
            write!(f, "{}–{}", start, end)
        }
    }
}

/// Encapsulates a restaurant/business and its hours.
#[derive(Deserialize, Clone)]
pub struct Restaurant {
    pub name: String,
    hours: HoursMap,
}

impl Restaurant {
    /// Gets the static list of all restaurants.
    pub fn get_list() -> Vec<Self> {
        serde_json::from_str(include_str!("../food.json")).unwrap_or_default()
    }

    /// Gets the hours of this restaurant on the given day.
    pub fn get_hours(&self, day: Day) -> Option<Hours> {
        match day {
            Day::Sunday => self.hours.sunday,
            Day::Monday => self.hours.monday,
            Day::Tuesday => self.hours.tuesday,
            Day::Wednesday => self.hours.wednesday,
            Day::Thursday => self.hours.thursday,
            Day::Friday => self.hours.friday,
            Day::Saturday => self.hours.saturday,
        }
    }

    /// Returns whether this restaurant is open on the given day.
    pub fn is_open(&self, day: Day) -> bool {
        self.get_hours(day).is_some()
    }

    /// Returns whether this restaurant is a suitable candidate for dining, considering
    /// travel time and business hours.
    pub fn is_viable(&self, day: Day, time: Time) -> bool {
        match self.get_hours(day) {
            None => false,
            Some(hours) => {
                let t = time + 10; // Account for travel time, etc.
                hours.start < t && hours.end > t
            }
        }
    }
}
