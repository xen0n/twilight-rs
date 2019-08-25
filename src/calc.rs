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
    pub sunset: i64,
    /**
     * Time of sunrise (civil twilight) in milliseconds or -1 in the case the
     * day or night never ends.
     */
    pub sunrise: i64,
}

/**
 * calculates the civil twilight bases on time and geo-coordinates.
 *
 * @param time time in milliseconds.
 * @param latitude latitude in degrees.
 * @param longitude latitude in degrees.
 */
pub(crate) fn calculateTwilight(time: i64, latitude: f64, longitude: f64) -> TwilightResult {
    let daysSince2000 = (time - UTC_2000) as f64 / (DAY_IN_MILLIS as f64);

    // mean anomaly
    let meanAnomaly = 6.240059968 + daysSince2000 * 0.01720197;

    // true anomaly
    let trueAnomaly = meanAnomaly + C1 * meanAnomaly.sin() + C2
            * (2.0 * meanAnomaly).sin() + C3 * (3.0 * meanAnomaly).sin();

    // ecliptic longitude
    let solarLng = trueAnomaly + 1.796593063 + ::std::f64::consts::PI;

    // solar transit in days since 2000
    let arcLongitude = -longitude / 360.0;
    let n = (daysSince2000 - J0 - arcLongitude).round();
    let solarTransitJ2000 = n + J0 + arcLongitude + 0.0053 * meanAnomaly.sin()
            + -0.0069 * (2.0 * solarLng).sin();

    // declination of sun
    let solarDec = (solarLng.sin() * OBLIQUITY.sin()).asin();

    let latRad = latitude * DEGREES_TO_RADIANS;

    let cosHourAngle = (ALTIDUTE_CORRECTION_CIVIL_TWILIGHT.sin() - latRad.sin()
            * solarDec.sin()) / (latRad.cos() * solarDec.cos());
    // The day or night never ends for the given date and location, if this value is out of
    // range.
    if cosHourAngle >= 1.0 {
        return TwilightResult {
            state: State::Night,
            sunset: -1,
            sunrise: -1,
        };
    } else if cosHourAngle <= -1.0 {
        return TwilightResult {
            state: State::Day,
            sunset: -1,
            sunrise: -1,
        };
    }

    let hourAngle = cosHourAngle.acos() / (2.0 * ::std::f64::consts::PI);

    let sunset = ((solarTransitJ2000 + hourAngle) * DAY_IN_MILLIS as f64).round() as i64 + UTC_2000;
    let sunrise = ((solarTransitJ2000 - hourAngle) * DAY_IN_MILLIS as f64).round() as i64 + UTC_2000;

    let state = if sunrise < time && sunset > time {
        State::Day
    } else {
        State::Night
    };

    TwilightResult {
        state: state,
        sunset: sunset,
        sunrise: sunrise,
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

        let result = calculateTwilight(now_milliseconds, lat, lon);
        assert_eq!(result.state, State::Day);
        assert_eq!(result.sunrise, 1566680508648);  // 05:01:48
        assert_eq!(result.sunset, 1566730442552);  // 18:54:02
    }
}
