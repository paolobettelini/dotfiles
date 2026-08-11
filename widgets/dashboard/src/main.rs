use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, Orientation};
use widgetbuilder::{
    load_css,
    widget_layer::{Anchor, WidgetLayer},
};

const APP_ID: &str = "ch.bettelini.paolo.Dashboard";

mod actions;
mod clock;
mod sysbtn;
mod textboxnotes;
mod volume;
mod weather;
mod webbtn;

fn main() -> gtk::glib::ExitCode {
    let app = Application::builder().application_id(APP_ID).build();
    app.connect_activate(build_ui);
    app.run()
}

fn build_ui(app: &Application) {
    if let Some(window) = app.active_window() {
        window.present();
        return;
    }

    load_css(include_str!("style.css"));

    let window = ApplicationWindow::builder()
        .application(app)
        .title("Dashboard")
        .build();
    window.add_css_class("dashboard-window");

    WidgetLayer::new(false, true, Anchor::Center, Anchor::Center)
        .with_namespace("paolo-dashboard")
        .apply(&window);

    let escape = gtk::EventControllerKey::new();
    let window_escape = window.clone();
    escape.connect_key_pressed(move |_, key, _, _| {
        if key == gtk::gdk::Key::Escape {
            window_escape.close();
            gtk::glib::Propagation::Stop
        } else {
            gtk::glib::Propagation::Proceed
        }
    });
    window.add_controller(escape);

    // Keep the same geometry as the GTK3 dashboard: three rows on the left,
    // weather + notes on the right. Spacing lives on each tile (5px per side),
    // so the 130px/410px proportions stay stable across GTK themes.
    let root = gtk::Box::new(Orientation::Horizontal, 0);
    root.add_css_class("dashboard-root");

    let left = gtk::Box::new(Orientation::Vertical, 0);
    left.add_css_class("dashboard-left");

    let right = gtk::Box::new(Orientation::Vertical, 0);
    right.add_css_class("dashboard-right");
    right.set_vexpand(true);

    let system_row = gtk::Box::new(Orientation::Horizontal, 0);
    system_row.set_homogeneous(true);
    system_row.append(&sysbtn::build(
        "⏻",
        "Shut down",
        "action-shutdown",
        actions::shutdown,
    ));
    system_row.append(&sysbtn::build(
        "",
        "Reboot",
        "action-reboot",
        actions::reboot,
    ));
    system_row.append(&sysbtn::build(
        "",
        "Suspend",
        "action-suspend",
        actions::suspend,
    ));
    system_row.append(&sysbtn::build(
        "󰗼",
        "Log out of Hyprland",
        "action-logout",
        actions::logout,
    ));

    let middle_row = gtk::Box::new(Orientation::Horizontal, 0);
    middle_row.append(&volume::build());
    middle_row.append(&clock::build());

    let web_row = gtk::Box::new(Orientation::Horizontal, 0);
    web_row.set_homogeneous(true);
    web_row.append(&webbtn::build(
        "",
        "YouTube",
        "https://www.youtube.com/",
        "web-youtube",
    ));
    web_row.append(&webbtn::build(
        "樓",
        "Reddit",
        "https://www.reddit.com/",
        "web-reddit",
    ));
    web_row.append(&webbtn::build(
        "",
        "WhatsApp Web",
        "https://web.whatsapp.com/",
        "web-whatsapp",
    ));
    web_row.append(&webbtn::build(
        "",
        "GitHub",
        "https://github.com/",
        "web-github",
    ));

    left.append(&system_row);
    left.append(&middle_row);
    left.append(&web_row);

    right.append(&weather::build());
    right.append(&textboxnotes::build());

    root.append(&left);
    root.append(&right);
    window.set_child(Some(&root));

    window.present();
}
