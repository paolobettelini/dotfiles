use crate::weather::api::query_data;
use widgetbuilder::cache::*;

// 4.9 minutes of time to live before update
const TTL: u64 = 290;

pub struct WeatherCache {
    pub icon: FSCachedValue,
    pub stat: FSCachedValue,
    pub temp: FSCachedValue,
    pub hex: FSCachedValue,
}

impl WeatherCache {
    pub fn new() -> Self {
        let icon = FSCachedValue::new("weather_icon");
        let stat = FSCachedValue::new("weather_stat");
        let temp = FSCachedValue::new("weather_temp");
        let hex = FSCachedValue::new("weather_hex");

        Self {
            icon,
            stat,
            temp,
            hex,
        }
    }

    pub fn update(&self) -> bool {
        let last_update = self.icon.last_update();
        if let Some(time) = last_update {
            if time < TTL {
                return false; // Data is still good
            }
        }

        let data = query_data();

        if let Some((icon, description, temp, hex)) = data {
            let _ = self.icon.write_content(&icon);
            let _ = self.stat.write_content(&description);
            let _ = self.temp.write_content(&temp);
            let _ = self.hex.write_content(&hex);
        } else {
            let _ = self.icon.write_content("");
            let _ = self.stat.write_content("Weather Unavailable");
            let _ = self.temp.write_content("-");
            let _ = self.hex.write_content("#adadff");
        }

        true
    }
}
