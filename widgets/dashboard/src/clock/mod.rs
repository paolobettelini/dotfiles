use chrono::prelude::*;
use gtk::glib;
use gtk::glib::*;
use gtk::prelude::*;
use gtk::{Align::*, Orientation::*};
use std::time::Duration;
use widgetbuilder::load_css;

// 5 sec update interval
const UPDATE_INTERVAL: Duration = Duration::from_secs(5);

pub fn popoulate(container: &gtk::Box) {
    // Init CSS
    load_css(include_bytes!("style.css"));
    container.set_widget_name("clock");

    // Init UI

    let inner_container = gtk::Box::builder()
        .orientation(Horizontal)
        .spacing(50)
        //.spave_evenyl(50)
        .vexpand(false)
        .hexpand(false)
        .build();

    let left = gtk::Box::builder()
        .orientation(Horizontal)
        .spacing(0)
        .build();

    let right = gtk::Box::builder().orientation(Vertical).spacing(0).build();

    let label_hour = gtk::Label::builder()
        .valign(Start)
        .wrap(true)
        //.limit_width(25)
        .build();
    label_hour.set_widget_name("time_hour");
    label_hour.set_text(&get_hour());

    let label_min = gtk::Label::builder()
        .valign(End)
        .wrap(true)
        //.limit_width(25)
        .build();
    label_min.set_widget_name("time_min");
    label_min.set_text(&get_minute());

    let label_mer = gtk::Label::builder()
        .valign(Start)
        .halign(End)
        //.limit_width(25)
        .wrap(true)
        .build();
    label_mer.set_widget_name("time_mer");
    label_mer.set_text(&get_am_pm());

    let label_day = gtk::Label::builder()
        .valign(End)
        .halign(End)
        //.limit_width(25)
        .wrap(true)
        .build();
    label_day.set_widget_name("time_day");
    label_day.set_text(&get_day_of_week());

    container.add(&inner_container);

    inner_container.add(&left);
    inner_container.add(&right);

    left.add(&label_hour);
    left.add(&label_min);

    right.add(&label_mer);
    right.add(&label_day);

    // Interval loop

    glib::timeout_add_local(
        UPDATE_INTERVAL,
        clone!(
            @weak label_hour,
            @weak label_min,
            @weak label_mer,
            @weak label_day
            => @default-return Continue(false), move || {

            // Update values
            label_min.set_text(&get_hour());
            label_min.set_text(&get_minute());
            label_mer.set_text(&get_am_pm());
            label_day.set_text(&get_day_of_week());

            // Continue the timeout
            glib::Continue(true)
        }),
    );
}

fn get_hour() -> String {
    let local = Local::now();
    local.format("%I").to_string()
}

fn get_minute() -> String {
    let local = Local::now();
    local.format("%M").to_string()
}

fn get_am_pm() -> String {
    let local = Local::now();
    local.format("%p").to_string()
}

fn get_day_of_week() -> String {
    let local = Local::now();
    local.format("%A").to_string()
}
