use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::env;
use std::sync::OnceLock;
use std::time::Duration;

const API_URL: &str = "https://api.openweathermap.org/data/2.5/weather";
const CONNECT_TIMEOUT: Duration = Duration::from_secs(2);
const REQUEST_TIMEOUT: Duration = Duration::from_secs(4);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherData {
    pub icon: String,
    pub description: String,
    pub temperature: String,
    pub kind: WeatherKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WeatherKind {
    ClearDay,
    ClearNight,
    PartlyCloudy,
    Cloudy,
    Rain,
    Thunder,
    Snow,
    Fog,
    Unknown,
}

impl WeatherKind {
    pub fn css_class(self) -> &'static str {
        match self {
            Self::ClearDay => "weather-clear-day",
            Self::ClearNight => "weather-clear-night",
            Self::PartlyCloudy => "weather-partly-cloudy",
            Self::Cloudy => "weather-cloudy",
            Self::Rain => "weather-rain",
            Self::Thunder => "weather-thunder",
            Self::Snow => "weather-snow",
            Self::Fog => "weather-fog",
            Self::Unknown => "weather-unknown",
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum Units {
    Metric,
    Imperial,
    Standard,
}

impl Units {
    fn from_env() -> Self {
        match env::var("WEATHER_UNITS")
            .unwrap_or_else(|_| "metric".to_string())
            .to_ascii_lowercase()
            .as_str()
        {
            "imperial" => Self::Imperial,
            "standard" => Self::Standard,
            _ => Self::Metric,
        }
    }

    fn api_value(self) -> &'static str {
        match self {
            Self::Metric => "metric",
            Self::Imperial => "imperial",
            Self::Standard => "standard",
        }
    }

    fn suffix(self) -> &'static str {
        match self {
            Self::Metric => "°C",
            Self::Imperial => "°F",
            Self::Standard => "K",
        }
    }
}

#[derive(Debug)]
struct WeatherConfig {
    location: String,
    api_key: String,
    units: Units,
}

impl WeatherConfig {
    fn from_env() -> Result<Self, String> {
        let location = env::var("WEATHER_LOCATION")
            .map_err(|_| "WEATHER_LOCATION is not configured".to_string())?;
        let api_key = env::var("WEATHER_API_KEY")
            .map_err(|_| "WEATHER_API_KEY is not configured".to_string())?;

        if location.trim().is_empty() {
            return Err("WEATHER_LOCATION is empty".to_string());
        }
        if api_key.trim().is_empty() {
            return Err("WEATHER_API_KEY is empty".to_string());
        }

        Ok(Self {
            location,
            api_key,
            units: Units::from_env(),
        })
    }
}

#[derive(Debug, Deserialize)]
struct ApiResponse {
    weather: Vec<ApiWeather>,
    main: ApiMain,
}

#[derive(Debug, Deserialize)]
struct ApiWeather {
    icon: String,
    description: String,
}

#[derive(Debug, Deserialize)]
struct ApiMain {
    temp: f64,
}

pub fn query_data() -> Result<WeatherData, String> {
    let config = WeatherConfig::from_env()?;
    let client = http_client()?;

    let response = client
        .get(API_URL)
        .query(&[
            ("q", config.location.as_str()),
            ("units", config.units.api_value()),
            ("appid", config.api_key.as_str()),
        ])
        .send()
        .map_err(|error| format!("weather request failed: {error}"))?
        .error_for_status()
        .map_err(|error| format!("weather API returned an error: {error}"))?
        .json::<ApiResponse>()
        .map_err(|error| format!("invalid weather response: {error}"))?;

    let current = response
        .weather
        .first()
        .ok_or_else(|| "weather response did not contain a condition".to_string())?;
    let kind = weather_kind(&current.icon);

    Ok(WeatherData {
        icon: weather_symbol(kind).to_string(),
        description: capitalize_each_word(&current.description),
        temperature: format!("{:.1}{}", response.main.temp, config.units.suffix()),
        kind,
    })
}


fn http_client() -> Result<Client, String> {
    static CLIENT: OnceLock<Result<Client, String>> = OnceLock::new();
    CLIENT
        .get_or_init(|| {
            Client::builder()
                .connect_timeout(CONNECT_TIMEOUT)
                .timeout(REQUEST_TIMEOUT)
                .user_agent("hypr-dashboard/0.2")
                .build()
                .map_err(|error| format!("could not create HTTP client: {error}"))
        })
        .clone()
}

fn weather_kind(icon_code: &str) -> WeatherKind {
    match icon_code {
        "01d" => WeatherKind::ClearDay,
        "01n" => WeatherKind::ClearNight,
        "02d" | "02n" => WeatherKind::PartlyCloudy,
        "03d" | "03n" | "04d" | "04n" => WeatherKind::Cloudy,
        "09d" | "09n" | "10d" | "10n" => WeatherKind::Rain,
        "11d" | "11n" => WeatherKind::Thunder,
        "13d" | "13n" => WeatherKind::Snow,
        "50d" | "50n" => WeatherKind::Fog,
        _ => WeatherKind::Unknown,
    }
}

fn weather_symbol(kind: WeatherKind) -> &'static str {
    match kind {
        WeatherKind::ClearDay => " ",
        WeatherKind::ClearNight => " ",
        WeatherKind::PartlyCloudy => " ",
        WeatherKind::Cloudy => " ",
        WeatherKind::Rain => " ",
        WeatherKind::Thunder => "",
        WeatherKind::Snow => " ",
        WeatherKind::Fog => " ",
        WeatherKind::Unknown => " ",
    }
}

fn capitalize_each_word(input: &str) -> String {
    input
        .split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => {
                    first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase()
                }
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_openweather_icon_codes() {
        assert_eq!(weather_kind("01d"), WeatherKind::ClearDay);
        assert_eq!(weather_kind("10n"), WeatherKind::Rain);
        assert_eq!(weather_kind("bogus"), WeatherKind::Unknown);
    }

    #[test]
    fn normalizes_description_case() {
        assert_eq!(capitalize_each_word("BROKEN CLOUDS"), "Broken Clouds");
    }
}
