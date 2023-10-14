use std::rc::Rc;

use chrono::{DateTime, Duration, Local, NaiveTime, Timelike};
use serde::{Deserialize, Serialize};
use slint::ComponentHandle;

use crate::{smhi, ui};

pub fn refresh(
    latitude: f32,
    longitude: f32,
    timezone: String,
    window: slint::Weak<ui::MainWindow>,
) {
    if let Some(window) = window.upgrade() {
        let daytime = window.global::<ui::DayTime>();
        daytime.set_refreshing(true);
    }
    std::thread::spawn(move || {
        // TODO: error handling
        let mut sunrises = Vec::new();
        let mut sunsets = Vec::new();
        for i in 0..=10 {
            let date = Local::now() + Duration::days(i);
            let response = fetch(latitude, longitude, &date, &timezone).unwrap();
            sunrises.push(response.results.sunrise.hour() as i32);
            sunsets.push(response.results.sunset.hour() as i32);
        }
        window
            .upgrade_in_event_loop(move |window| {
                let daytime = window.global::<ui::DayTime>();
                daytime.set_sunrises(slint::ModelRc::from(Rc::new(slint::VecModel::from(
                    sunrises,
                ))));
                daytime.set_sunsets(slint::ModelRc::from(Rc::new(slint::VecModel::from(
                    sunsets,
                ))));
                daytime.set_refreshing(false);

                smhi::refresh(latitude, longitude, window.as_weak());
            })
            .unwrap();
    });
}

fn fetch(
    latitude: f32,
    longitude: f32,
    date: &DateTime<Local>,
    timezone: &String,
) -> Result<Response, Box<dyn std::error::Error>> {
    let body = reqwest::blocking::get(format!(
        "https://api.sunrisesunset.io/json?lat={:.6}&lng={:.6}&timezone={}&date={}",
        latitude,
        longitude,
        timezone,
        date.format("%Y-%m-%d"),
    ))?
    .text()?;
    Ok(serde_json::from_str::<Response>(&body)?)
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Response {
    pub results: Results,
    pub status: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Results {
    #[serde(with = "time_format")]
    pub sunrise: NaiveTime,
    #[serde(with = "time_format")]
    pub sunset: NaiveTime,
    #[serde(with = "time_format")]
    pub first_light: NaiveTime,
    #[serde(with = "time_format")]
    pub last_light: NaiveTime,
    #[serde(with = "time_format")]
    pub dawn: NaiveTime,
    #[serde(with = "time_format")]
    pub dusk: NaiveTime,
    #[serde(with = "time_format")]
    pub solar_noon: NaiveTime,
    #[serde(with = "time_format")]
    pub golden_hour: NaiveTime,
    pub day_length: NaiveTime,
    pub timezone: String,
    pub utc_offset: f32,
}

mod time_format {
    use chrono::NaiveTime;
    use serde::{self, Deserialize, Deserializer, Serializer};

    const FORMAT: &str = "%I:%M:%S %p";

    pub fn serialize<S>(time: &NaiveTime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("{}", time.format(FORMAT));
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<NaiveTime, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        NaiveTime::parse_from_str(&s, FORMAT).map_err(serde::de::Error::custom)
    }
}
