use serde_json::Value;
use std::env;

const CITY_VAR: &str = "WEATHER_LOCATION";
const UNITS: &str = "WEATHER_UNITS";
const KEY_VAR: &str = "WEATHER_API_KEY";

fn get_json() -> Option<Value> {
    let city = env::var(CITY_VAR).expect(&format!("Could not read {CITY_VAR} variable"));
    let units = env::var(UNITS).expect(&format!("Could not read {UNITS} variable"));
    let key = env::var(KEY_VAR).expect(&format!("Could not read {KEY_VAR} variable"));

    let url = format!(
        "http://api.openweathermap.org/data/2.5/weather?q={}&units={}&appid={}",
        city, units, key
    );

    let weather = match reqwest::blocking::get(url) {
        Ok(response) => match response.text() {
            Ok(text) => Some(text),
            Err(_) => None,
        },
        Err(_) => None,
    }?;

    serde_json::from_str(&weather).ok()
}

// (icon, description, temp, hex)
pub fn query_data() -> Option<(String, String, String, String)> {
    // Update using API query
    println!("Sending HTTP request to weather API");

    let json = get_json()?;

    let temp = json["main"]["temp"].as_f64().unwrap_or_default();
    let icon_code = json["weather"][0]["icon"].as_str().unwrap_or_default();
    let description = json["weather"][0]["description"]
        .as_str()
        .unwrap_or_default()
        .to_string();

    let description = capitalize_each_word(&description);

    // Round and add unit
    let temp = format!("{temp:.1}°C");

    let (icon, hex) = match icon_code {
        "50d" | "50n" => (" ", "#84afdb"),
        "01d" => (" ", "#ffd86b"),
        "01n" => (" ", "#fcdcf6"),
        "02d" => (" ", "#adadff"),
        "02n" => (" ", "#adadff"),
        "03d" | "03n" | "04d" | "04n" => (" ", "#adadff"),
        "09d" | "09n" | "10d" | "10n" => (" ", "#6b95ff"),
        "11d" | "11n" => ("", "#ffeb57"),
        "13d" | "13n" => (" ", "#e3e6fc"),
        "40d" | "40n" => (" ", "#84afdb"),
        _ => (" ", "#adadff"),
    };

    Some((icon.to_string(), description, temp, hex.to_string()))
}

fn capitalize_each_word(input: &str) -> String {
    input
        .split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first_char) => {
                    let capitalized_word = first_char.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase();
                    capitalized_word
                }
            }
        })
        .collect::<Vec<String>>()
        .join(" ")
}