use std::rc::Rc;

use chrono::{DateTime, Local, NaiveDateTime, NaiveTime, Timelike, Utc};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use slint::ComponentHandle;

use crate::ui;

pub fn refresh(latitude: f32, longitude: f32, window: slint::Weak<ui::MainWindow>) {
    if let Some(window) = window.upgrade() {
        let smhi = window.global::<ui::Smhi>();
        smhi.set_refreshing(true);
    }
    std::thread::spawn(move || {
        // TODO: error handling
        let response = fetch(latitude, longitude).unwrap();
        window
            .upgrade_in_event_loop(move |window| {
                let smhi = window.global::<ui::Smhi>();
                smhi.set_forecasts(slint::ModelRc::from(Rc::new(slint::VecModel::from(
                    collect_forecasts(&response.time_series),
                ))));
                smhi.set_refreshing(false);
            })
            .unwrap();
    });
}

fn collect_forecasts(series: &[TimeSeries]) -> Vec<ui::Forecast> {
    series
        .iter()
        .group_by(|s| s.valid_time.with_timezone(&Local).date_naive())
        .into_iter()
        .map(|(date, s)| {
            let params = s
                .into_iter()
                .map(|s| {
                    let temperature = s.parameters.iter().find_map(|p| match p.name.as_str() {
                        "t" => p.values.first().map(|v| *v as f32),
                        _ => None,
                    });
                    let symbol = s.parameters.iter().find_map(|p| match p.name.as_str() {
                        "Wsymb2" => p.values.first().map(|v| *v as i32),
                        _ => None,
                    });
                    ui::Params {
                        hour: s.valid_time.with_timezone(&Local).hour() as i32,
                        temperature: temperature.unwrap_or(f32::NAN),
                        symbol: symbol.unwrap_or(-1),
                    }
                })
                .collect_vec();

            let duration = NaiveDateTime::new(date, NaiveTime::default())
                .signed_duration_since(Utc::now().naive_utc());

            ui::Forecast {
                date: slint::SharedString::from(date.to_string()),
                offset: (duration.num_days() + duration.num_hours().clamp(0, 1)) as i32,
                min_temperature: params
                    .iter()
                    .min_by(|a, b| a.temperature.partial_cmp(&b.temperature).unwrap())
                    .unwrap()
                    .temperature,
                max_temperature: params
                    .iter()
                    .max_by(|a, b| a.temperature.partial_cmp(&b.temperature).unwrap())
                    .unwrap()
                    .temperature,
                params: slint::ModelRc::from(Rc::new(slint::VecModel::from(params))),
            }
        })
        .collect_vec()
}

fn fetch(latitude: f32, longitude: f32) -> Result<Response, Box<dyn std::error::Error>> {
    let body = reqwest::blocking::get(format!("https://opendata-download-metfcst.smhi.se/api/category/pmp3g/version/2/geotype/point/lon/{:.6}/lat/{:.6}/data.json", longitude, latitude))?.text()?;
    Ok(serde_json::from_str::<Response>(&body)?)
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Response {
    pub approved_time: DateTime<Utc>,
    pub reference_time: DateTime<Utc>,
    pub geometry: Geometry,
    pub time_series: Vec<TimeSeries>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Geometry {
    #[serde(rename = "type")]
    pub ty: String,
    pub coordinates: Vec<Vec<f64>>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimeSeries {
    pub valid_time: DateTime<Utc>,
    pub parameters: Vec<Parameter>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Parameter {
    pub name: String,
    pub level_type: String,
    pub level: i64,
    pub unit: String,
    pub values: Vec<f64>,
}
