use gtk::prelude::*;
use gtk::Orientation::*;

use widgetbuilder::load_css;

pub fn popoulate(container: &gtk::Box, icon: &str, action: fn() -> ()) {
    // Init CSS
    load_css(include_bytes!("style.css"));

    // Init UI
    let inner_container = gtk::Box::builder()
        .orientation(Vertical)
        .build();
    inner_container.style_context().add_class("sysbtn");

    let button = gtk::Button::builder()
        .label(icon)
        //.timeout(60s) ???
        .build();

    button.connect_clicked(move |_| {
        action();
    });

    container.add(&inner_container);
    inner_container.add(&button);
}