use gtk::glib::{self, ControlFlow};
use gtk::prelude::*;
use std::cell::Cell;
use std::process::{Command, Stdio};
use std::rc::Rc;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

const SET_DEBOUNCE: Duration = Duration::from_millis(90);

pub fn build() -> gtk::Box {
    let root = gtk::Box::new(gtk::Orientation::Vertical, 0);
    root.add_css_class("tile");
    root.add_css_class("card");
    root.add_css_class("volume-card");

    let icon = gtk::Label::new(Some("󰕾"));
    icon.add_css_class("volume-icon");
    icon.set_hexpand(true);
    icon.set_halign(gtk::Align::Center);

    let scale = gtk::Scale::with_range(gtk::Orientation::Horizontal, 0.0, 100.0, 1.0);
    scale.set_draw_value(true);
    scale.set_digits(0);
    scale.set_value_pos(gtk::PositionType::Bottom);
    scale.set_hexpand(true);
    scale.set_sensitive(false);
    scale.add_css_class("volume-scale");

    root.append(&icon);
    root.append(&scale);

    let suppress_change = Rc::new(Cell::new(false));
    let write_generation = Rc::new(Cell::new(0_u64));

    let suppress_changed = suppress_change.clone();
    let generation_changed = write_generation.clone();
    scale.connect_value_changed(move |scale| {
        let value = scale.value().clamp(0.0, 100.0);

        if suppress_changed.get() {
            return;
        }

        let generation = generation_changed.get().wrapping_add(1);
        generation_changed.set(generation);
        let generation_write = generation_changed.clone();
        glib::timeout_add_local_once(SET_DEBOUNCE, move || {
            if generation_write.get() != generation {
                return;
            }

            thread::spawn(move || {
                if let Err(error) = set_volume(value) {
                    eprintln!("dashboard volume update failed: {error}");
                }
            });
        });
    });

    // Do not touch wpctl/pactl/amixer until GTK has had a chance to present
    // the window. The initial value is filled asynchronously afterwards.
    let (sender, receiver) = mpsc::channel::<Result<f64, String>>();
    glib::idle_add_local_once(move || {
        thread::spawn(move || {
            let result = read_volume();
            let _ = sender.send(result);
        });
    });

    let scale_init = scale.clone();
    let root_init = root.clone();
    let suppress_init = suppress_change.clone();
    glib::timeout_add_local(Duration::from_millis(60), move || match receiver.try_recv() {
        Ok(Ok(value)) => {
            suppress_init.set(true);
            scale_init.set_value(value);
            suppress_init.set(false);
            scale_init.set_sensitive(true);
            root_init.set_tooltip_text(None);
            ControlFlow::Break
        }
        Ok(Err(error)) => {
            scale_init.set_sensitive(false);
            root_init.set_tooltip_text(Some(&error));
            suppress_init.set(false);
            ControlFlow::Break
        }
        Err(mpsc::TryRecvError::Empty) => ControlFlow::Continue,
        Err(mpsc::TryRecvError::Disconnected) => {
            root_init.set_tooltip_text(Some("volume worker stopped unexpectedly"));
            suppress_init.set(false);
            ControlFlow::Break
        }
    });

    root
}

fn read_volume() -> Result<f64, String> {
    if let Some(stdout) = output("wpctl", &["get-volume", "@DEFAULT_AUDIO_SINK@"]) {
        if let Some(value) = parse_wpctl(&stdout) {
            return Ok(value);
        }
    }

    if let Some(stdout) = output("pactl", &["get-sink-volume", "@DEFAULT_SINK@"]) {
        if let Some(value) = parse_first_percent(&stdout) {
            return Ok(value);
        }
    }

    if let Some(stdout) = output("amixer", &["get", "Master"]) {
        if let Some(value) = parse_first_percent(&stdout) {
            return Ok(value);
        }
    }

    Err("no supported volume backend found (tried wpctl, pactl and amixer)".to_string())
}

fn set_volume(value: f64) -> Result<(), String> {
    let percent = format!("{}%", value.round().clamp(0.0, 100.0) as i32);

    if status("wpctl", &["set-volume", "@DEFAULT_AUDIO_SINK@", &percent]) {
        return Ok(());
    }
    if status(
        "pactl",
        &["set-sink-volume", "@DEFAULT_SINK@", &percent],
    ) {
        return Ok(());
    }
    if status("amixer", &["-D", "default", "sset", "Master", &percent]) {
        return Ok(());
    }

    Err("all supported volume backends failed".to_string())
}

fn output(program: &str, args: &[&str]) -> Option<String> {
    let result = Command::new(program)
        .args(args)
        .stderr(Stdio::null())
        .output()
        .ok()?;

    if !result.status.success() {
        return None;
    }

    Some(String::from_utf8_lossy(&result.stdout).into_owned())
}

fn status(program: &str, args: &[&str]) -> bool {
    Command::new(program)
        .args(args)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

fn parse_wpctl(output: &str) -> Option<f64> {
    let value = output.split_whitespace().find_map(|token| {
        token
            .parse::<f64>()
            .ok()
            .filter(|value| (0.0..=1.5).contains(value))
    })?;
    Some((value * 100.0).clamp(0.0, 100.0))
}

fn parse_first_percent(output: &str) -> Option<f64> {
    output.split_whitespace().find_map(|token| {
        let cleaned = token
            .trim_matches(|c: char| c == '[' || c == ']' || c == ',' || c == '/')
            .trim_end_matches('%');

        if token.contains('%') {
            cleaned.parse::<f64>().ok().map(|value| value.clamp(0.0, 100.0))
        } else {
            None
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_wpctl_output() {
        assert_eq!(parse_wpctl("Volume: 0.52"), Some(52.0));
    }

    #[test]
    fn parses_pactl_percent() {
        let output = "front-left: 32768 / 50% / -18.06 dB";
        assert_eq!(parse_first_percent(output), Some(50.0));
    }

    #[test]
    fn parses_amixer_percent() {
        let output = "Mono: Playback 32 [50%] [-30.00dB] [on]";
        assert_eq!(parse_first_percent(output), Some(50.0));
    }
}
