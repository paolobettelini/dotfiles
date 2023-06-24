use gtk::{prelude::*, Application, ApplicationWindow, Orientation::*};
use widgetbuilder::{load_css, widget_layer::*};

const APP_ID: &str = "ch.bettelini.paolo.Dashboard";

mod actions;
use actions::*;

// Widgets
mod clock;
mod sysbtn;
mod textboxnotes;
mod volume;
mod weather;
mod webbtn;

fn main() {
    let app = Application::builder().application_id(APP_ID).build();
    app.connect_activate(move |app| build_ui(&app));
    app.run();
}

pub fn build_ui(app: &Application) {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Dashboard")
        .build();

    // Load CSS
    load_css(include_bytes!("style.css"));

    // Dashboard geometry
    WidgetLayer::new(
        &window,
        false,
        true,
        Anchor::Center,
        Anchor::Center,
        (400, 200),
    );

    /*
        container
            left_container
                left_row1
                    (btn1 + btn2 + btn3 + btn4)
                left_row2
                    (volume + clock)
            weather_box
                (weather)
                (notes)
    */

    let container = gtk::Box::builder().orientation(Horizontal).build();

    let left_container = gtk::Box::builder().orientation(Vertical).build();

    let right_container = gtk::Box::builder().orientation(Vertical).build();

    let left_row1 = gtk::Box::builder().orientation(Horizontal).build();

    let left_row2 = gtk::Box::builder().orientation(Horizontal).build();

    let left_row3 = gtk::Box::builder().orientation(Horizontal).build();

    // Shutdown button
    let shutdown_box = gtk::Box::builder().build();
    shutdown_box.style_context().add_class("margin1");
    shutdown_box.set_widget_name("shutdown");
    sysbtn::popoulate(&shutdown_box, "⏻", shutdown);

    // Reboot button
    let reboot_box = gtk::Box::builder().build();
    reboot_box.style_context().add_class("margin1");
    reboot_box.set_widget_name("reboot");
    sysbtn::popoulate(&reboot_box, "", reboot);

    // Suspend button
    let suspend_box = gtk::Box::builder().build();
    suspend_box.style_context().add_class("margin1");
    suspend_box.set_widget_name("suspend");
    sysbtn::popoulate(&suspend_box, "", suspend);

    // Logout button
    let logout_box = gtk::Box::builder().build();
    logout_box.style_context().add_class("margin1");
    logout_box.set_widget_name("logout");
    sysbtn::popoulate(&logout_box, "󰗼", logout);

    // Volume
    let volume_box = gtk::Box::builder().build();
    volume_box.style_context().add_class("keepbackground");
    volume_box.style_context().add_class("margin1");
    volume::popoulate(&volume_box);

    // Clock
    let clock_box = gtk::Box::builder().build();
    clock_box.style_context().add_class("margin1");
    clock_box.style_context().add_class("keepbackground");
    clock::popoulate(&clock_box);

    // Youtube
    let youtube_box = gtk::Box::builder().build();
    youtube_box.style_context().add_class("margin1");
    youtube_box.set_widget_name("youtube");
    webbtn::popoulate(&youtube_box, "", open_youtube);

    // Reddit
    let reddit_box = gtk::Box::builder().build();
    reddit_box.style_context().add_class("margin1");
    reddit_box.set_widget_name("reddit");
    webbtn::popoulate(&reddit_box, "樓", open_reddit);

    // Whatsapp
    let whatsapp_box = gtk::Box::builder().build();
    whatsapp_box.style_context().add_class("margin1");
    whatsapp_box.set_widget_name("whatsapp");
    webbtn::popoulate(&whatsapp_box, "", open_whatsapp);

    // GitHub
    let github_box = gtk::Box::builder().build();
    github_box.style_context().add_class("margin1");
    github_box.set_widget_name("github");
    webbtn::popoulate(&github_box, "", open_github);

    // Weather widget
    let weather_box = gtk::Box::builder().build();
    weather_box.set_widget_name("weather");
    weather_box.style_context().add_class("keepbackground");
    weather::popoulate(&weather_box);

    // Textbox notes
    let textbox_notes = gtk::Box::builder().build();
    textbox_notes.style_context().add_class("margin1");
    textbox_notes.style_context().add_class("keepbackground");
    textboxnotes::popoulate(&textbox_notes);

    // Add elements
    left_container.add(&left_row1);
    left_container.add(&left_row2);
    left_container.add(&left_row3);

    right_container.add(&weather_box);
    right_container.add(&textbox_notes);

    left_row1.add(&shutdown_box);
    left_row1.add(&reboot_box);
    left_row1.add(&suspend_box);
    left_row1.add(&logout_box);

    left_row2.add(&volume_box);
    left_row2.add(&clock_box);

    left_row3.add(&youtube_box);
    left_row3.add(&reddit_box);
    left_row3.add(&whatsapp_box);
    left_row3.add(&github_box);

    container.add(&left_container);
    container.add(&right_container);

    window.set_child(Some(&container));

    window.show_all();
}
