use std::{
    cmp::{Ordering, PartialOrd}, fmt, ops::{Add, Sub}, str::FromStr,
};

#[derive(Clone, Copy, Deserialize, PartialEq)]
pub struct Time {
    hours: u8,
    #[serde(default)]
    minutes: u8,
}

impl Time {
    pub fn new(hours: i32, minutes: i32) -> Self {
        Self {
            hours: hours as u8,
            minutes: minutes as u8,
        }
    }

    pub fn with_hours(hours: u8) -> Self {
        Self { hours, minutes: 0 }
    }
}

impl FromStr for Time {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !s.contains(":") {
            return Err(());
        }
        let parts = s.split(" ")
            .map(|c| c.parse::<u8>().unwrap_or_default())
            .collect::<Vec<u8>>();
        Ok(Self {
            hours: parts[0],
            minutes: parts[1],
        })
    }
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
        write!(
            f,
            "{}:{:02} {}",
            hours,
            self.minutes,
            if pm { "PM" } else { "AM" }
        )
    }
}

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

#[derive(Deserialize)]
struct HoursMap {
    sunday: Option<Hours>,
    monday: Option<Hours>,
    tuesday: Option<Hours>,
    wednesday: Option<Hours>,
    thursday: Option<Hours>,
    friday: Option<Hours>,
    saturday: Option<Hours>,
}

#[derive(Deserialize, Clone, Copy)]
pub struct Hours {
    start: Time,
    end: Time,
}

impl fmt::Display for Hours {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}–{}", self.start, self.end)
    }
}

#[derive(Deserialize)]
pub struct Restaurant {
    pub name: String,
    hours: HoursMap,
}

impl Restaurant {
    pub fn get_list() -> Vec<Self> {
        serde_json::from_str(include_str!("../food.json")).unwrap_or_default()
    }

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

    pub fn is_open(&self, day: Day) -> bool {
        self.get_hours(day).is_some()
    }

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
