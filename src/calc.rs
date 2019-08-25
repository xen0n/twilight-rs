use crate::State;

const DEGREES_TO_RADIANS: f32 = ::std::f32::consts::PI / 180.0f32;

// element for calculating solar transit.
const J0: f32 = 0.0009f32;

// correction for civil twilight
const ALTIDUTE_CORRECTION_CIVIL_TWILIGHT: f32 = -0.104719755f32;

// coefficients for calculating Equation of Center.
const C1: f32 = 0.0334196f32;
const C2: f32 = 0.000349066f32;
const C3: f32 = 0.000005236f32;

const OBLIQUITY: f32 = 0.40927971f32;

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
    let daysSince2000: f32 = (time - UTC_2000) as f32 / (DAY_IN_MILLIS as f32);

    // mean anomaly
    let meanAnomaly: f32 = 6.240059968f32 + daysSince2000 * 0.01720197f32;

    // true anomaly
    let trueAnomaly: f64 = (meanAnomaly + C1 * meanAnomaly.sin() + C2
            * (2f32 * meanAnomaly).sin() + C3 * (3f32 * meanAnomaly).sin()) as f64;

    // ecliptic longitude
    let solarLng: f64 = trueAnomaly + 1.796593063f64 + ::std::f64::consts::PI;

    // solar transit in days since 2000
    let arcLongitude: f64 = -longitude / 360f64;
    let n: f32 = ((daysSince2000 - J0) as f64 - arcLongitude).round() as f32;
    let solarTransitJ2000: f64 = (n + J0) as f64 + arcLongitude + 0.0053f64 * (meanAnomaly as f64).sin()
            + -0.0069f64 * (2f64 * solarLng).sin();

    // declination of sun
    let solarDec: f64 = (solarLng.sin() * (OBLIQUITY as f64).sin()).asin();

    let latRad: f64 = latitude * (DEGREES_TO_RADIANS as f64);

    let cosHourAngle: f64 = ((ALTIDUTE_CORRECTION_CIVIL_TWILIGHT as f64).sin() - latRad.sin()
            * solarDec.sin()) / (latRad.cos() * solarDec.cos());
    // The day or night never ends for the given date and location, if this value is out of
    // range.
    if cosHourAngle >= 1f64 {
        return TwilightResult {
            state: State::Night,
            sunset: -1,
            sunrise: -1,
        };
    } else if cosHourAngle <= -1f64 {
        return TwilightResult {
            state: State::Day,
            sunset: -1,
            sunrise: -1,
        };
    }

    let hourAngle: f32 = (cosHourAngle.acos() / (2f64 * ::std::f64::consts::PI)) as f32;

    let sunset = ((solarTransitJ2000 + hourAngle as f64) * DAY_IN_MILLIS as f64).round() as i64 + UTC_2000;
    let sunrise = ((solarTransitJ2000 - hourAngle as f64) * DAY_IN_MILLIS as f64).round() as i64 + UTC_2000;

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
        assert_eq!(result.sunrise, 1566680515294);  // 05:01:55
        assert_eq!(result.sunset, 1566730449118);  // 18:54:09
    }
}
