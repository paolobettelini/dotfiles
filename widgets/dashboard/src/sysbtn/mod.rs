use crate::actions::ActionResult;
use gtk::prelude::*;

pub fn build(
    symbol: &str,
    tooltip: &str,
    css_class: &str,
    action: fn() -> ActionResult,
) -> gtk::Button {
    let button = gtk::Button::with_label(symbol);
    button.add_css_class("tile");
    button.add_css_class("action-button");
    button.add_css_class("icon-glyph");
    button.add_css_class(css_class);
    button.set_tooltip_text(Some(tooltip));

    button.connect_clicked(move |_| {
        if let Err(error) = action() {
            eprintln!("dashboard action failed: {error}");
        }
    });

    button
}
