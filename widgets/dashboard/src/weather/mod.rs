use gtk::glib;
use gtk::glib::*;
use gtk::prelude::*;
use gtk::{Align::*, Orientation::*};
use std::rc::Rc;
use std::time::Duration;
use widgetbuilder::load_css;

pub(crate) mod api;
mod cache;

use cache::*;

// 5 min update interval
const UPDATE_INTERVAL: Duration = Duration::from_secs(300);

// use trait also with load_css method
pub fn popoulate(container: &gtk::Box) {
    // Init CSS
    load_css(include_bytes!("style.css"));

    // Init cache
    let cache = WeatherCache::new();
    cache.update();

    // Init UI
    let inner_container = gtk::Box::builder()
        .orientation(Vertical)
        .spacing(10)
        //.space_evenly(false)
        .vexpand(false)
        .hexpand(false)
        .build();

    let section1 = gtk::Box::builder()
        .orientation(Horizontal)
        .vexpand(false)
        .hexpand(false)
        .build();

    let section2 = gtk::Box::builder()
        .orientation(Vertical)
        .spacing(10)
        .halign(Center)
        .vexpand(false)
        .hexpand(false)
        .build();

    let label_icon = gtk::Label::builder().halign(Start).build();
    label_icon.set_widget_name("label_icon"); // set CSS id

    let label_temp = gtk::Label::builder().halign(End).build();
    label_temp.set_widget_name("label_temp"); // set CSS id

    let label_stat = gtk::Label::builder().build();
    label_stat.set_widget_name("label_stat"); // set CSS id

    container.add(&inner_container);
    inner_container.add(&section1);
    inner_container.add(&section2);
    section1.add(&label_icon);
    section1.add(&label_temp);
    section2.add(&label_stat);

    // Init values and CSS properties
    let stat = cache.stat.read_content().unwrap();
    let temp = cache.temp.read_content().unwrap();
    let icon = cache.icon.read_content().unwrap();
    let hex = cache.hex.read_content().unwrap();
    label_stat.set_text(&stat);
    label_temp.set_text(&temp);
    label_icon.set_text(&icon);
    let css = format!("#label_icon {{ color: {}; }}", hex);
    load_css(css.as_bytes());

    // Update loop
    let _main_context = MainContext::default();
    let cache = Rc::new(cache);

    glib::timeout_add_local(
        UPDATE_INTERVAL,
        clone!(
            @weak label_stat,
            @weak label_temp,
            @weak label_icon,
            @strong cache
            => @default-return Continue(false), move || {

            // Update data
            println!("Updating weather info");
            let updated = cache.update();

            if updated {
                let stat = cache.stat.read_content().unwrap();
                let temp = cache.temp.read_content().unwrap();
                let icon = cache.icon.read_content().unwrap();

                // Change the label text
                label_stat.set_text(&stat);
                label_temp.set_text(&temp);
                label_icon.set_text(&icon);

                // Change CSS
                let hex = cache.hex.read_content().unwrap();
                let css = format!("#label_temp {{ color: {}; }}", hex);
                load_css(css.as_bytes());
            }

            // Continue the timeout
            glib::Continue(true)
        }),
    );
}
