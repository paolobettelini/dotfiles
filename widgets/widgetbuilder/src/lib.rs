pub mod cache;
pub mod widget_layer;

pub fn load_css(style: &[u8]) {
    use gtk::prelude::CssProviderExt;

    let provider = gtk::CssProvider::new();
    provider.load_from_data(style).expect("Failed to load CSS");
    gtk::StyleContext::add_provider_for_screen(
        &gtk::gdk::Screen::default().expect("Error initializing gtk css provider."),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}
