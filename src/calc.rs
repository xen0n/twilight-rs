use crate::State;

const DEGREES_TO_RADIANS: f64 = ::std::f64::consts::PI / 180.0;

// element for calculating solar transit.
const J0: f64 = 0.0009;

// correction for civil twilight
const ALTIDUTE_CORRECTION_CIVIL_TWILIGHT: f64 = -0.104719755;

// coefficients for calculating Equation of Center.
const C1: f64 = 0.0334196;
const C2: f64 = 0.000349066;
const C3: f64 = 0.000005236;

const OBLIQUITY: f64 = 0.40927971;

// Java time on Jan 1, 2000 12:00 UTC.
const UTC_2000: i64 = 946728000000;

const DAY_IN_MILLIS: i64 = 1000 * 60 * 60 * 24;

pub(crate) struct TwilightResult {
    /**
     * Current state
     */
    pub state: State,
    /**
     * Time of sunset (civil twilight) in milliseconds or -1 in the case the day
     * or night never ends.
     */
    pub sunset: Option<i64>,
    /**
     * Time of sunrise (civil twilight) in milliseconds or -1 in the case the
     * day or night never ends.
     */
    pub sunrise: Option<i64>,
}

/**
 * calculates the civil twilight bases on time and geo-coordinates.
 *
 * @param time time in milliseconds.
 * @param latitude latitude in degrees.
 * @param longitude latitude in degrees.
 */
pub(crate) fn calculate_twilight(time: i64, latitude: f64, longitude: f64) -> TwilightResult {
    let days_since_2000 = (time - UTC_2000) as f64 / (DAY_IN_MILLIS as f64);

    // mean anomaly
    let mean_anomaly = 6.240059968 + days_since_2000 * 0.01720197;

    // true anomaly
    let true_anomaly = mean_anomaly + C1 * mean_anomaly.sin() + C2
            * (2.0 * mean_anomaly).sin() + C3 * (3.0 * mean_anomaly).sin();

    // ecliptic longitude
    let solar_lng = true_anomaly + 1.796593063 + ::std::f64::consts::PI;

    // solar transit in days since 2000
    let arc_longitude = -longitude / 360.0;
    let n = (days_since_2000 - J0 - arc_longitude).round();
    let solar_transit_j2000 = n + J0 + arc_longitude + 0.0053 * mean_anomaly.sin()
            + -0.0069 * (2.0 * solar_lng).sin();

    // declination of sun
    let solar_dec = (solar_lng.sin() * OBLIQUITY.sin()).asin();

    let lat_rad = latitude * DEGREES_TO_RADIANS;

    let cos_hour_angle = (ALTIDUTE_CORRECTION_CIVIL_TWILIGHT.sin() - lat_rad.sin()
            * solar_dec.sin()) / (lat_rad.cos() * solar_dec.cos());
    // The day or night never ends for the given date and location, if this value is out of
    // range.
    if cos_hour_angle >= 1.0 {
        return TwilightResult {
            state: State::Night,
            sunset: None,
            sunrise: None,
        };
    } else if cos_hour_angle <= -1.0 {
        return TwilightResult {
            state: State::Day,
            sunset: None,
            sunrise: None,
        };
    }

    let hour_angle = cos_hour_angle.acos() / (2.0 * ::std::f64::consts::PI);

    let sunset = ((solar_transit_j2000 + hour_angle) * DAY_IN_MILLIS as f64).round() as i64 + UTC_2000;
    let sunrise = ((solar_transit_j2000 - hour_angle) * DAY_IN_MILLIS as f64).round() as i64 + UTC_2000;

    let state = if sunrise < time && sunset > time {
        State::Day
    } else {
        State::Night
    };

    TwilightResult {
        state: state,
        sunset: Some(sunset),
        sunrise: Some(sunrise),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        // location of Shanghai took from Wikipedia (People's Square)
        let lat = (31 * 3600 + 13 * 60 + 43) as f64 / 3600.0;
        let lon = (121 * 3600 + 28 * 60 + 29) as f64 / 3600.0;

        // the time I wrote this test
        let now_milliseconds = 1566703808294;  // 2019-08-25T11:30:08.294+08:00

        let result = calculate_twilight(now_milliseconds, lat, lon);
        assert_eq!(result.state, State::Day);
        assert_eq!(result.sunrise, Some(1566680508648));  // 05:01:48
        assert_eq!(result.sunset, Some(1566730442552));  // 18:54:02
    }
}
