use gtk::glib;
use gtk::prelude::*;
use std::cell::Cell;
use std::env;
use std::path::PathBuf;
use std::rc::Rc;
use std::time::Duration;
use widgetbuilder::cache::{legacy_widgets_cache, FsValue};

const SAVE_DEBOUNCE: Duration = Duration::from_millis(450);

pub fn build() -> gtk::Box {
    let root = gtk::Box::new(gtk::Orientation::Vertical, 0);
    root.add_css_class("tile");
    root.add_css_class("card");
    root.add_css_class("notes-card");
    root.set_hexpand(true);
    root.set_vexpand(true);

    let buffer = gtk::TextBuffer::new(None::<&gtk::TextTagTable>);
    buffer.set_enable_undo(true);

    let text_view = gtk::TextView::with_buffer(&buffer);
    text_view.set_wrap_mode(gtk::WrapMode::WordChar);
    text_view.set_hexpand(true);
    text_view.set_vexpand(true);
    text_view.set_left_margin(10);
    text_view.set_right_margin(10);
    text_view.set_top_margin(10);
    text_view.set_bottom_margin(10);
    text_view.add_css_class("notes-editor");

    let scroller = gtk::ScrolledWindow::new();
    scroller.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
    scroller.set_min_content_height(150);
    scroller.set_hexpand(true);
    scroller.set_vexpand(true);
    scroller.set_child(Some(&text_view));
    scroller.add_css_class("notes-scroller");

    root.append(&scroller);

    let store = match notes_store() {
        Ok(store) => {
            migrate_legacy_notes(&store);
            match store.read_optional() {
                Ok(Some(text)) => {
                    buffer.set_text(&text);
                    root.set_tooltip_text(Some(&store.path().display().to_string()));
                }
                Ok(None) => {
                    root.set_tooltip_text(Some(&store.path().display().to_string()));
                }
                Err(error) => {
                    root.set_tooltip_text(Some(&format!("Could not read notes: {error}")));
                }
            }
            Some(store)
        }
        Err(error) => {
            root.set_tooltip_text(Some(&format!("Notes are not persistent: {error}")));
            None
        }
    };

    // Generation-based debounce avoids source-removal races and writes only
    // after the user has stopped typing for a short moment.
    let save_generation = Rc::new(Cell::new(0_u64));
    let generation_changed = save_generation.clone();
    let buffer_changed = buffer.clone();
    let root_changed = root.clone();

    let store_on_unmap = store.clone();
    let buffer_on_unmap = buffer.clone();
    root.connect_unmap(move |_| {
        if let Some(store) = store_on_unmap.as_ref() {
            if let Err(error) = write_buffer(&buffer_on_unmap, store) {
                eprintln!("dashboard notes final save failed: {error}");
            }
        }
    });

    buffer.connect_changed(move |_| {
        let Some(store) = store.clone() else {
            root_changed.set_tooltip_text(Some("Notes are not persistent on this system"));
            return;
        };

        let generation = generation_changed.get().wrapping_add(1);
        generation_changed.set(generation);

        let generation_save = generation_changed.clone();
        let buffer_save = buffer_changed.clone();
        let root_save = root_changed.clone();
        glib::timeout_add_local_once(SAVE_DEBOUNCE, move || {
            if generation_save.get() != generation {
                return;
            }

            match write_buffer(&buffer_save, &store) {
                Ok(()) => {
                    root_save.set_tooltip_text(Some(&store.path().display().to_string()));
                }
                Err(error) => {
                    root_save.set_tooltip_text(Some(&format!("Could not save notes: {error}")));
                }
            }
        });
    });

    root
}

fn write_buffer(buffer: &gtk::TextBuffer, store: &FsValue) -> std::io::Result<()> {
    let (start, end) = buffer.bounds();
    let text = buffer.text(&start, &end, true);
    store.write_atomic(text.as_str())
}

fn notes_store() -> std::io::Result<FsValue> {
    if let Some(path) = env::var_os("DASHBOARD_NOTES_FILE").filter(|value| !value.is_empty()) {
        return FsValue::from_path(PathBuf::from(path));
    }

    FsValue::state("dashboard", "notes.txt")
}

fn migrate_legacy_notes(store: &FsValue) {
    if store.exists() {
        return;
    }

    let Ok(old_path) = legacy_widgets_cache("notes") else {
        return;
    };
    let Ok(text) = std::fs::read_to_string(old_path) else {
        return;
    };

    if let Err(error) = store.write_atomic(&text) {
        eprintln!("dashboard: could not migrate old notes: {error}");
    }
}
