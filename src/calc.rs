use crate::{State, Twilight, TwilightTimes};

const DEGREES_TO_RADIANS: f64 = ::std::f64::consts::PI / 180.0;

// element for calculating solar transit.
const J0: f64 = 0.0009;

// correction for civil twilight
const ALTITUDE_CORRECTION_CIVIL_TWILIGHT: f64 = ::std::f64::consts::PI / 180.0 * 6.0;

// coefficients for calculating Equation of Center.
const C1: f64 = 0.0334196;
const C2: f64 = 0.000349066;
const C3: f64 = 0.000005236;

const OBLIQUITY: f64 = 0.40927971;

// Java time on Jan 1, 2000 12:00 UTC.
const UTC_2000: i64 = 946728000000;

const DAY_IN_MILLIS: i64 = 1000 * 60 * 60 * 24;

/// calculates the civil twilight bases on time and geo-coordinates.
///
/// @param time time in milliseconds.
/// @param latitude latitude in degrees.
/// @param longitude latitude in degrees.
pub(crate) fn calculate_twilight(time: i64, latitude: f64, longitude: f64) -> Twilight {
    let days_since_2000 = (time - UTC_2000) as f64 / (DAY_IN_MILLIS as f64);

    // mean anomaly
    let mean_anomaly = 6.240059968 + days_since_2000 * 0.01720197;

    // true anomaly
    let true_anomaly = mean_anomaly
        + C1 * mean_anomaly.sin()
        + C2 * (2.0 * mean_anomaly).sin()
        + C3 * (3.0 * mean_anomaly).sin();

    // ecliptic longitude
    let solar_lng = true_anomaly + 1.796593063 + ::std::f64::consts::PI;

    // solar transit in days since 2000
    let arc_longitude = -longitude / 360.0;
    let n = (days_since_2000 - J0 - arc_longitude).round();
    let solar_transit_j2000 =
        n + J0 + arc_longitude + 0.0053 * mean_anomaly.sin() + -0.0069 * (2.0 * solar_lng).sin();

    // declination of sun
    let solar_dec = (solar_lng.sin() * OBLIQUITY.sin()).asin();

    let lat_rad = latitude * DEGREES_TO_RADIANS;

    let result_factory = |sun_altitude_delta: f64| {
        let cos_hour_angle = (sun_altitude_delta.sin()
            - lat_rad.sin() * solar_dec.sin())
            / (lat_rad.cos() * solar_dec.cos());

        cos_hour_angle_to_times(time, solar_transit_j2000, cos_hour_angle)
    };

    result_factory(ALTITUDE_CORRECTION_CIVIL_TWILIGHT)
}

fn cos_hour_angle_to_times(time: i64, solar_transit_j2000: f64, cos_hour_angle: f64) -> Twilight {
    // The day or night never ends for the given date and location, if this value is out of
    // range.
    if cos_hour_angle >= 1.0 {
        return Twilight {
            state: State::Night,
            times: None,
        };
    } else if cos_hour_angle <= -1.0 {
        return Twilight {
            state: State::Day,
            times: None,
        };
    }

    let hour_angle = cos_hour_angle.acos() / (2.0 * ::std::f64::consts::PI);

    let times = hour_angle_to_times(solar_transit_j2000, hour_angle);

    let state = if times.sunrise < time && times.sunset > time {
        State::Day
    } else {
        State::Night
    };

    Twilight {
        state: state,
        times: Some(times),
    }
}

fn hour_angle_to_times(solar_transit_j2000: f64, hour_angle: f64) -> TwilightTimes {
    let sunset =
        ((solar_transit_j2000 + hour_angle) * DAY_IN_MILLIS as f64).round() as i64 + UTC_2000;
    let sunrise =
        ((solar_transit_j2000 - hour_angle) * DAY_IN_MILLIS as f64).round() as i64 + UTC_2000;

    TwilightTimes {
        sunset: sunset,
        sunrise: sunrise,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        macro_rules! testcase {
            (($lat_h: expr , $lat_m: expr , $lat_s: expr,
              $lon_h: expr , $lon_m: expr , $lon_s: expr)
             @ $now: expr
             => {$state: expr, $sunrise: expr, $sunset: expr, }) => {
                testcase!(
                    ($lat_h, $lat_m, $lat_s, $lon_h, $lon_m, $lon_s) @ $now
                    => {
                        $state,
                        Some(TwilightTimes {
                            sunrise: $sunrise,
                            sunset: $sunset,
                        })
                    }
                );
            };

            (($lat_h: expr , $lat_m: expr , $lat_s: expr,
              $lon_h: expr , $lon_m: expr , $lon_s: expr)
             @ $now: expr
             => {$state: expr, }) => {
                testcase!(
                    ($lat_h, $lat_m, $lat_s, $lon_h, $lon_m, $lon_s) @ $now
                    => {
                        $state,
                        None
                    }
                );
            };

            (($lat_h: expr , $lat_m: expr , $lat_s: expr,
              $lon_h: expr , $lon_m: expr , $lon_s: expr)
             @ $now: expr
             => {$state: expr, $times: expr}) => {
                let lat_h = $lat_h;
                let (lat_sign, lat_h) = if lat_h < 0 { (-1.0, -lat_h) } else { (1.0, lat_h) };
                let lat = lat_sign * (lat_h * 3600 + $lat_m * 60 + $lat_s) as f64 / 3600.0;

                let lon_h = $lon_h;
                let (lon_sign, lon_h) = if lon_h < 0 { (-1.0, -lon_h) } else { (1.0, lon_h) };
                let lon = lon_sign * (lon_h * 3600 + $lon_m * 60 + $lon_s) as f64 / 3600.0;

                let result = calculate_twilight($now, lat, lon);

                assert_eq!(result.state, $state);
                assert_eq!(result.times, $times);
            };
        }

        testcase!(
            // location of Shanghai took from Wikipedia (People's Square)
            (31, 13, 43, 121, 28, 29)
            // the time I wrote this test
            @ 1566703808294  // 2019-08-25T11:30:08.294+08:00
            => {
                State::Day,
                1566683966997,  // 05:59:26
                1566726984202,  // 17:56:24
            }
        );

        testcase!(
            // location of the Antarctic Taishan station
            (-73, 51, 50, 76, 58, 29)
            @ 1546272000000  // 2019-01-01T00:00:00+08:00
            => {
                State::Day,
            }
        );
    }
}
