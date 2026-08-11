use crate::weather::api::{WeatherData, WeatherKind};
use serde::{Deserialize, Serialize};
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};
use widgetbuilder::cache::{legacy_widgets_cache, FsValue};

const CACHE_NAMESPACE: &str = "dashboard";
const CACHE_FILE: &str = "weather.json";
pub const CACHE_TTL_SECONDS: u64 = 300;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedWeather {
    pub fetched_at: u64,
    pub data: WeatherData,
}

impl CachedWeather {
    pub fn age_seconds(&self) -> u64 {
        unix_now().saturating_sub(self.fetched_at)
    }

    pub fn is_fresh(&self) -> bool {
        self.age_seconds() < CACHE_TTL_SECONDS
    }
}

#[derive(Debug, Clone)]
pub struct WeatherCache {
    file: Option<FsValue>,
}

impl WeatherCache {
    pub fn new() -> Self {
        Self {
            file: FsValue::cache(CACHE_NAMESPACE, CACHE_FILE).ok(),
        }
    }

    pub fn read(&self) -> Option<CachedWeather> {
        self.read_current().or_else(read_legacy)
    }

    pub fn write(&self, data: &WeatherData) -> Result<(), String> {
        let Some(file) = self.file.as_ref() else {
            return Err("weather cache directory is unavailable".to_string());
        };

        let record = CachedWeather {
            fetched_at: unix_now(),
            data: data.clone(),
        };
        let raw = serde_json::to_string(&record)
            .map_err(|error| format!("could not serialize weather cache: {error}"))?;
        file.write_atomic(&raw)
            .map_err(|error| format!("could not write weather cache: {error}"))
    }

    fn read_current(&self) -> Option<CachedWeather> {
        let file = self.file.as_ref()?;
        let raw = file.read_string().ok()?;
        serde_json::from_str(&raw).ok()
    }
}

fn read_legacy() -> Option<CachedWeather> {
    let description_path = legacy_widgets_cache("weather_stat").ok()?;
    let temperature_path = legacy_widgets_cache("weather_temp").ok()?;
    let description = fs::read_to_string(&description_path).ok()?;
    let temperature = fs::read_to_string(&temperature_path).ok()?;

    let fetched_at = [&description_path, &temperature_path]
        .into_iter()
        .filter_map(|path| fs::metadata(path).ok()?.modified().ok())
        .filter_map(system_time_to_unix)
        .max()
        .unwrap_or_default();

    Some(CachedWeather {
        fetched_at,
        data: WeatherData {
            icon: "".to_string(),
            description,
            temperature,
            kind: WeatherKind::Unknown,
        },
    })
}

fn system_time_to_unix(time: SystemTime) -> Option<u64> {
    time.duration_since(UNIX_EPOCH)
        .ok()
        .map(|duration| duration.as_secs())
}

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default()
}
