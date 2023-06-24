use gtk::prelude::*;
use gtk::Orientation::*;
use std::process::{Command, Stdio};

use widgetbuilder::load_css;

pub fn popoulate(container: &gtk::Box) {
    // Init CSS
    load_css(include_bytes!("style.css"));

    // Init UI
    let inner_container = gtk::Box::builder().orientation(Vertical).build();
    inner_container.style_context().add_class("volume");

    let label = gtk::Label::new(Some("󰕾"));

    let scale = gtk::Scale::with_range(Horizontal, 0.0, 100.0, 1.0);
    scale.set_draw_value(true);

    // Initial volume
    let volume = parse_volume();
    scale.set_value(volume);

    // Slider event
    scale.connect_change_value(move |_, _, v| {
        let v = format!("{v}%");

        Command::new("amixer")
            .args(&["-D", "default", "sset", "Master", &v])
            .stdout(Stdio::null())
            .spawn()
            .expect("Failed to execute amixer command.");

        Inhibit(false)
    });

    container.add(&inner_container);
    inner_container.add(&label);
    inner_container.add(&scale);
}

// This... this is bad
fn parse_volume() -> f64 {
    let output = Command::new("amixer")
        .args(&["get", "Master"])
        .output()
        .expect("Failed to execute amixer command.");

    let stdout = String::from_utf8_lossy(&output.stdout);
    if stdout.contains(" [") {
        let start_index = stdout.find(" [").unwrap() + 2;
        let end_index = stdout.find('%').unwrap();
        let volume_str = &stdout[start_index..end_index];
        return volume_str.parse::<f64>().unwrap();
    }
    0.0 // Default volume if not found
}
