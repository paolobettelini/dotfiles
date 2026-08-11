use gtk::glib::{self, ControlFlow};
use gtk::prelude::*;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc};
use std::thread;
use std::time::Duration;

pub(crate) mod api;
mod cache;

use api::WeatherData;
use cache::{CachedWeather, WeatherCache, CACHE_TTL_SECONDS};

const POLL_INTERVAL: Duration = Duration::from_millis(100);
const WEATHER_CLASSES: &[&str] = &[
    "weather-clear-day",
    "weather-clear-night",
    "weather-partly-cloudy",
    "weather-cloudy",
    "weather-rain",
    "weather-thunder",
    "weather-snow",
    "weather-fog",
    "weather-unknown",
];

enum WorkerMessage {
    Updated(WeatherData),
    Failed(String),
}

pub fn build() -> gtk::Box {
    let root = gtk::Box::new(gtk::Orientation::Vertical, 10);
    root.add_css_class("tile");
    root.add_css_class("card");
    root.add_css_class("weather-card");
    root.set_vexpand(false);

    let section1 = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    let section2 = gtk::Box::new(gtk::Orientation::Vertical, 10);
    section2.set_halign(gtk::Align::Center);

    let icon = gtk::Label::new(Some(""));
    icon.add_css_class("weather-icon");
    icon.add_css_class("weather-unknown");
    icon.set_halign(gtk::Align::Start);

    let temperature = gtk::Label::new(Some("-"));
    temperature.add_css_class("weather-temperature");
    temperature.set_hexpand(true);
    temperature.set_halign(gtk::Align::End);

    let description = gtk::Label::new(Some("Weather Unavailable"));
    description.add_css_class("weather-description");
    description.set_halign(gtk::Align::Center);
    description.set_wrap(true);

    section1.append(&icon);
    section1.append(&temperature);
    section2.append(&description);
    root.append(&section1);
    root.append(&section2);

    let cache = WeatherCache::new();
    let mut had_cached_data = false;
    let mut refresh_immediately = true;
    if let Some(cached) = cache.read() {
        had_cached_data = true;
        refresh_immediately = !cached.is_fresh();
        apply_weather(&icon, &temperature, &description, &cached.data);
        root.set_tooltip_text(Some(&cached_status(&cached)));
    } else {
        root.set_tooltip_text(Some("Weather data unavailable; refreshing in background"));
    }

    let (sender, receiver) = mpsc::channel::<WorkerMessage>();
    let inflight = Arc::new(AtomicBool::new(false));

    // The first API request starts only after the window has entered GTK's
    // main loop, so DNS/network failures can never prevent the UI from opening.
    if refresh_immediately {
        let first_sender = sender.clone();
        let first_cache = cache.clone();
        let first_inflight = inflight.clone();
        glib::idle_add_local_once(move || {
            spawn_refresh(first_sender, first_cache, first_inflight);
        });
    }

    let timer_sender = sender.clone();
    let timer_cache = cache.clone();
    let timer_inflight = inflight.clone();
    glib::timeout_add_seconds_local(CACHE_TTL_SECONDS as u32, move || {
        spawn_refresh(
            timer_sender.clone(),
            timer_cache.clone(),
            timer_inflight.clone(),
        );
        ControlFlow::Continue
    });

    let root_poll = root.clone();
    let icon_poll = icon.clone();
    let temperature_poll = temperature.clone();
    let description_poll = description.clone();
    let cached_state = std::rc::Rc::new(std::cell::Cell::new(had_cached_data));
    let cached_poll = cached_state.clone();

    glib::timeout_add_local(POLL_INTERVAL, move || {
        while let Ok(message) = receiver.try_recv() {
            match message {
                WorkerMessage::Updated(data) => {
                    apply_weather(
                        &icon_poll,
                        &temperature_poll,
                        &description_poll,
                        &data,
                    );
                    root_poll.set_tooltip_text(Some("Weather updated"));
                    cached_poll.set(true);
                }
                WorkerMessage::Failed(error) => {
                    if cached_poll.get() {
                        root_poll.set_tooltip_text(Some(&format!(
                            "Offline; showing cached weather. {error}"
                        )));
                    } else {
                        for class in WEATHER_CLASSES {
                            icon_poll.remove_css_class(class);
                        }
                        icon_poll.add_css_class("weather-unknown");
                        icon_poll.set_text("");
                        temperature_poll.set_text("-");
                        description_poll.set_text("Weather Unavailable");
                        root_poll.set_tooltip_text(Some(&format!(
                            "Offline or weather is not configured. {error}"
                        )));
                    }
                }
            }
        }

        ControlFlow::Continue
    });

    root
}

fn spawn_refresh(sender: mpsc::Sender<WorkerMessage>, cache: WeatherCache, inflight: Arc<AtomicBool>) {
    if inflight.swap(true, Ordering::AcqRel) {
        return;
    }

    thread::spawn(move || {
        let message = match api::query_data() {
            Ok(data) => {
                if let Err(error) = cache.write(&data) {
                    eprintln!("dashboard weather cache: {error}");
                }
                WorkerMessage::Updated(data)
            }
            Err(error) => WorkerMessage::Failed(error),
        };

        let _ = sender.send(message);
        inflight.store(false, Ordering::Release);
    });
}

fn apply_weather(
    icon: &gtk::Label,
    temperature: &gtk::Label,
    description: &gtk::Label,
    data: &WeatherData,
) {
    icon.set_text(&data.icon);
    temperature.set_text(&data.temperature);
    description.set_text(&data.description);

    for class in WEATHER_CLASSES {
        icon.remove_css_class(class);
    }
    icon.add_css_class(data.kind.css_class());
}

fn cached_status(cached: &CachedWeather) -> String {
    let age = cached.age_seconds();
    if cached.is_fresh() {
        "Cached weather is fresh".to_string()
    } else if age < 3600 {
        format!("Cached weather · {} min old", age / 60)
    } else {
        format!("Cached weather · {} h old", age / 3600)
    }
}
