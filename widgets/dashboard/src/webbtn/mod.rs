use gtk::prelude::*;

use widgetbuilder::load_css;

pub fn popoulate(container: &gtk::Box, icon: &str, action: fn() -> ()) {
    // Init CSS
    load_css(include_bytes!("style.css"));

    // Init UI
    let inner_container = gtk::Box::builder().build();
    inner_container.style_context().add_class("webbtn");
    let event_box = gtk::EventBox::builder()
        .width_request(130)
        .hexpand(false)
        .build();

    let label = gtk::Label::new(Some(icon));
    label.set_hexpand(true);

    event_box.connect_button_press_event(move |_, _| {
        action();
        Inhibit(false)
    });

    container.add(&inner_container);
    inner_container.add(&event_box);
    event_box.add(&label);
}
