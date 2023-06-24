use gtk::glib;
use gtk::glib::*;
use gtk::prelude::*;
use gtk::{Align::*, Orientation::*};
use std::rc::Rc;
use std::time::Duration;
use gtk::TextTagTable;
use gtk::TextBuffer;
use std::cell::RefCell;
use std::sync::Arc;

use widgetbuilder::load_css;
use widgetbuilder::cache::*;

pub fn popoulate(container: &gtk::Box) {
    // Init CSS
    load_css(include_bytes!("style.css"));

    // Init cache
    let text_cache = FSCachedValue::new("notes");

    // Init UI
    let inner_container = gtk::Box::builder()
        .orientation(Vertical)
        .spacing(10)
        .vexpand(true)
        .hexpand(true)
        .build();

    // Create a scrolled window
    let scrolled_window = gtk::ScrolledWindow::new(None::<&gtk::Adjustment>, None::<&gtk::Adjustment>);
    scrolled_window.set_policy(gtk::PolicyType::Automatic, gtk::PolicyType::Automatic);

    let textview = gtk::TextView::builder()
        .hexpand(true)
        .vexpand(true)
        .build();
    textview.set_wrap_mode(gtk::WrapMode::WordChar);
    textview.set_editable(true);
    textview.set_widget_name("notes-editor"); // set CSS id

    // Create a text buffer
    let buffer = TextBuffer::new(None::<&gtk::TextTagTable>);
    let text = text_cache.read_content().unwrap_or("".to_string());
    buffer.set_text(&text);

    textview.set_buffer(Some(&buffer));

    scrolled_window.add(&textview);
    container.add(&inner_container);
    inner_container.add(&scrolled_window);

    let buffer_ref = Rc::new(RefCell::new(buffer));
    let timeout_id = Rc::new(RefCell::new(None::<SourceId>));

    // Keep latest timeout
    let timeout_id_clone = timeout_id.clone();
    let buffer_clone = buffer_ref.clone();

    let main_context = glib::MainContext::default();

    // Event triggered 500 ms after having stopped writing
    buffer_ref.borrow().connect_changed(move |_| {
        // Stop the old task
        let mut timeout_id = timeout_id_clone.borrow_mut();
        if let Some(source_id) = timeout_id.take() {
            let source = main_context.find_source_by_id(&source_id);
            
            if source.is_some() {
                source_id.remove();
            }
            
            *timeout_id = None;
        }

        let buffer_clone = buffer_clone.clone();

        // Create the new task
        let source_id = timeout_add_local(Duration::from_millis(500), move || {
            let buffer = buffer_clone.borrow();
            let (start, end) = buffer.bounds();
            let text = buffer.text(&start, &end, true);

            if let Some(text) = text {
                let text_cache = FSCachedValue::new("notes");
                let _ = text_cache.write_content(&text);
            }

            Continue(false)
        });

        *timeout_id = Some(source_id);
    });
}
