pub mod cache;
pub mod widget_layer;

pub fn load_css(style: &str) {
    let provider = gtk::CssProvider::new();

    provider.connect_parsing_error(|_, _, error| {
        eprintln!("dashboard CSS parsing error: {error}");
    });

    #[allow(deprecated)]
    provider.load_from_data(style);

    let Some(display) = gtk::gdk::Display::default() else {
        eprintln!("dashboard: no GDK display available; CSS was not installed");
        return;
    };

    gtk::style_context_add_provider_for_display(
        &display,
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}
