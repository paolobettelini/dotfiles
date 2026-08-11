use chrono::Local;
use gtk::glib::{self, ControlFlow};
use gtk::prelude::*;

pub fn build() -> gtk::Box {
    let root = gtk::Box::new(gtk::Orientation::Horizontal, 50);
    root.add_css_class("tile");
    root.add_css_class("card");
    root.add_css_class("clock-card");
    root.set_vexpand(false);
    root.set_hexpand(false);

    let time_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    let details = gtk::Box::new(gtk::Orientation::Vertical, 0);

    let hour = gtk::Label::new(None);
    let minute = gtk::Label::new(None);
    let meridiem = gtk::Label::new(None);
    let day = gtk::Label::new(None);

    hour.add_css_class("clock-hour");
    minute.add_css_class("clock-minute");
    meridiem.add_css_class("clock-meridiem");
    day.add_css_class("clock-day");

    hour.set_valign(gtk::Align::Start);
    minute.set_valign(gtk::Align::End);
    meridiem.set_valign(gtk::Align::Start);
    meridiem.set_halign(gtk::Align::End);
    day.set_valign(gtk::Align::End);
    day.set_halign(gtk::Align::End);

    time_box.append(&hour);
    time_box.append(&minute);
    details.append(&meridiem);
    details.append(&day);
    root.append(&time_box);
    root.append(&details);

    update_labels(&hour, &minute, &meridiem, &day);

    let hour_tick = hour.clone();
    let minute_tick = minute.clone();
    let meridiem_tick = meridiem.clone();
    let day_tick = day.clone();
    glib::timeout_add_seconds_local(5, move || {
        update_labels(&hour_tick, &minute_tick, &meridiem_tick, &day_tick);
        ControlFlow::Continue
    });

    root
}

fn update_labels(
    hour: &gtk::Label,
    minute: &gtk::Label,
    meridiem: &gtk::Label,
    day: &gtk::Label,
) {
    let now = Local::now();
    hour.set_text(&now.format("%I").to_string());
    minute.set_text(&now.format("%M").to_string());
    meridiem.set_text(&now.format("%p").to_string());
    day.set_text(&now.format("%A").to_string());
}
