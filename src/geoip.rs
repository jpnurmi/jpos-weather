use serde::{Deserialize, Serialize};
use slint::ComponentHandle;

use crate::{daytime, ui};

pub fn refresh(window: slint::Weak<ui::MainWindow>) {
    if let Some(window) = window.upgrade() {
        let geoip = window.global::<ui::GeoIP>();
        geoip.set_refreshing(true);
    }
    std::thread::spawn(move || {
        let response = from_env().or_else(|_| fetch()).unwrap();
        window
            .upgrade_in_event_loop(move |window| {
                let geoip = window.global::<ui::GeoIP>();
                geoip.set_latitude(response.latitude);
                geoip.set_longitude(response.longitude);
                geoip.set_city(slint::SharedString::from(response.city));
                geoip.set_country(slint::SharedString::from(response.country));
                geoip.set_timezone(slint::SharedString::from(&response.timezone));
                geoip.set_refreshing(false);

                daytime::refresh(
                    response.latitude,
                    response.longitude,
                    response.timezone,
                    window.as_weak(),
                );
            })
            .unwrap();
    });
}

fn from_env() -> Result<Response, Box<dyn std::error::Error>> {
    let response = Response {
        latitude: std::env::var("GEOIP_LATITUDE")?.parse::<f32>()?,
        longitude: std::env::var("GEOIP_LONGITUDE")?.parse::<f32>()?,
        city: std::env::var("GEOIP_CITY").unwrap_or_default(),
        country: std::env::var("GEOIP_COUNTRY").unwrap_or_default(),
        timezone: std::env::var("GEOIP_TIMEZONE").unwrap_or_default(),
    };
    Ok(response)
}

fn fetch() -> Result<Response, Box<dyn std::error::Error>> {
    let body = reqwest::blocking::get("https://api.ip2location.io/?format=json")?.text()?;
    Ok(serde_json::from_str::<Response>(&body)?)
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Response {
    pub latitude: f32,
    pub longitude: f32,
    #[serde(rename = "city_name")]
    pub city: String,
    #[serde(rename = "country_name")]
    pub country: String,
    #[serde(rename = "time_zone")]
    pub timezone: String,
}
