use crate::actions;
use gtk::prelude::*;

pub fn build(label: &str, tooltip: &str, url: &'static str, css_class: &str) -> gtk::Button {
    let button = gtk::Button::with_label(label);
    button.add_css_class("tile");
    button.add_css_class("web-button");
    button.add_css_class("icon-glyph");
    button.add_css_class(css_class);
    button.set_tooltip_text(Some(tooltip));

    button.connect_clicked(move |_| {
        if let Err(error) = actions::open_url(url) {
            eprintln!("dashboard link failed: {error}");
        }
    });

    button
}
