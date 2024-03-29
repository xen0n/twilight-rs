//! # twilight
//!
//! This is a straight-forward port of Android's `TwilightCalculator`,
//! like [this one from LineageOS's sources][java-link], that one can use
//! to calculate sunrise and sunset times on a given time and location.
//!
//! The public API is meant to be used together with [the `chrono` library][chrono].
//!
//! [java-link]: https://github.com/LineageOS/android_frameworks_support/blob/lineage-16.0/v7/appcompat/src/main/java/androidx/appcompat/app/TwilightCalculator.java
//! [chrono]: https://github.com/chronotope/chrono

#![deny(missing_docs, warnings)]

mod calc;

/// State of day.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum State {
    /// Daytime.
    Day,
    /// Nighttime.
    Night,
}

/// Twilight times of a given day.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct TwilightTimes {
    sunrise: i64,
    sunset: i64,
}

impl TwilightTimes {
    /// Time of sunrise (civil twilight) in the given day.
    pub fn sunrise_time<Tz: ::chrono::TimeZone>(&self, tz: Tz) -> ::chrono::DateTime<Tz> {
        let (s, ns) = ms_to_s_ns(self.sunrise);
        tz.timestamp(s, ns)
    }

    /// Time of sunset (civil twilight) in the given day.
    pub fn sunset_time<Tz: ::chrono::TimeZone>(&self, tz: Tz) -> ::chrono::DateTime<Tz> {
        let (s, ns) = ms_to_s_ns(self.sunset);
        tz.timestamp(s, ns)
    }
}

/// Result of twilight calculations.
pub struct Twilight {
    state: State,
    times: Option<TwilightTimes>,
}

impl Twilight {
    /// Calculates civil twilight times for a given time and location.
    pub fn calculate<T: Timestamp>(time_of_day: T, latitude: f64, longitude: f64) -> Self {
        let ms = time_of_day.as_unix_timestamp_ms();
        calc::calculate_twilight(ms, latitude, longitude)
    }

    /// Convenient method for calculating civil twilight times with the
    /// current time, for a given location.
    pub fn now(latitude: f64, longitude: f64) -> Self {
        let time_of_day = ::chrono::Utc::now();
        Self::calculate(time_of_day, latitude, longitude)
    }

    /// Returns if the specified time is day or night at the specified location.
    pub fn state(&self) -> State {
        self.state
    }

    /// Returns the civil twilight times, if the specified location is not
    /// under polar day/night at the given time.
    pub fn twilight_times(&self) -> Option<TwilightTimes> {
        self.times
    }
}

// Converts from millisecond timestamp to (second, nanosecond) format.
fn ms_to_s_ns(ms: i64) -> (i64, u32) {
    (ms / 1000, (ms % 1000) as u32 * 1000_000)
}

/// Timestamp suitable for this library's consumption.
pub trait Timestamp {
    /// Convert the time into Unix timestamp, in milliseconds.
    fn as_unix_timestamp_ms(&self) -> i64;
}

impl<Tz: ::chrono::TimeZone> Timestamp for ::chrono::DateTime<Tz> {
    fn as_unix_timestamp_ms(&self) -> i64 {
        self.timestamp_millis()
    }
}
