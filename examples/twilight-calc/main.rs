extern crate chrono;
extern crate reqwest;
extern crate serde;
#[macro_use]
extern crate serde_derive;

extern crate twilight;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let location = get_location()?;
    println!("    location: {}", location);

    let now = chrono::Local::now();
    println!("    time now: {}", now);

    let tw = twilight::Twilight::calculate(now, location.lat, location.lng);
    println!("   day/night: {:?}", tw.state());

    match tw.twilight_times() {
        Some(tt) => {
            let sunrise = tt.sunrise_time(chrono::Local);
            println!("     sunrise: {}", sunrise);

            let sunset = tt.sunset_time(chrono::Local);
            println!("      sunset: {}", sunset);
        }

        None => {
            println!("polar day/night!");
        }
    }

    Ok(())
}

#[derive(Copy, Clone, Debug, Deserialize)]
struct Location {
    pub lat: f64,
    pub lng: f64,
}

#[derive(Copy, Clone, Debug, Deserialize)]
struct GeolocateResponse {
    pub location: Location,
    pub accuracy: f64,
}

fn get_location() -> Result<Location, Box<dyn std::error::Error>> {
    // use geoclue's api key
    let url = "https://location.services.mozilla.com/v1/geolocate?key=geoclue";

    let resp: GeolocateResponse = reqwest::get(url)?.json()?;

    Ok(resp.location)
}

impl ::std::fmt::Display for Location {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        let sign_lat = if self.lat >= 0.0 { "N" } else { "S" };
        let sign_lng = if self.lng >= 0.0 { "E" } else { "W" };

        write!(f, "({}°{}, {}°{})", self.lat, sign_lat, self.lng, sign_lng)
    }
}
