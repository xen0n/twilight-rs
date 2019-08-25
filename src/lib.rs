mod calc;

/// State of day.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum State {
    /// Daytime.
    Day,
    /// Nighttime.
    Night,
}
