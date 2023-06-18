use gtk::prelude::*;

use gtk::ApplicationWindow;
use gtk_layer_shell::Edge::*;

#[derive(Debug, Copy, Clone)]
pub enum Anchor {
    Start(i32),
    Center,
    End(i32),
}

#[derive(Debug)]
pub struct WidgetLayer {
    exclusive: bool,
    h_anchor: Anchor,
    v_anchor: Anchor,
    size: (i32, i32),
}

impl WidgetLayer {
    pub fn new(
        window: &ApplicationWindow,
        exclusive: bool,
        h_anchor: Anchor,
        v_anchor: Anchor,
        size: (i32, i32),
    ) -> Self {
        let layer = Self {
            exclusive,
            h_anchor,
            v_anchor,
            size,
        };

        layer.adapt_window(window);

        layer
    }

    pub fn adapt_window(&self, window: &ApplicationWindow) {
        gtk_layer_shell::init_for_window(window);

        window.set_default_size(self.size.0, self.size.1);
        window.set_resizable(false);
        window.set_decorated(false);

        if self.exclusive {
            gtk_layer_shell::auto_exclusive_zone_enable(window);
        }

        Self::set_anchors(window, self.h_anchor, self.v_anchor);
    }

    pub fn set_anchors(window: &ApplicationWindow, h_anchor: Anchor, v_anchor: Anchor) {
        match h_anchor {
            Anchor::Start(margin) => {
                gtk_layer_shell::set_anchor(window, Left, true);
                gtk_layer_shell::set_margin(window, Left, margin);
            }
            Anchor::Center => {}
            Anchor::End(margin) => {
                gtk_layer_shell::set_anchor(window, Right, true);
                gtk_layer_shell::set_margin(window, Right, margin);
            }
        }

        match v_anchor {
            Anchor::Start(margin) => {
                gtk_layer_shell::set_anchor(window, Top, true);
                gtk_layer_shell::set_margin(window, Top, margin);
            }
            Anchor::Center => {}
            Anchor::End(margin) => {
                gtk_layer_shell::set_anchor(window, Bottom, true);
                gtk_layer_shell::set_margin(window, Bottom, margin);
            }
        }
    }
}

/*
    gtk_layer_shell::set_namespace(&window, namespace);
    gtk_layer_shell::set_keyboard_interactivity(&window, window_def.backend_options.wayland.focusable);

    match window_def.stacking {
        WindowStacking::Foreground => gtk_layer_shell::set_layer(&window, gtk_layer_shell::Layer::Top),
        WindowStacking::Background => gtk_layer_shell::set_layer(&window, gtk_layer_shell::Layer::Background),
        WindowStacking::Bottom => gtk_layer_shell::set_layer(&window, gtk_layer_shell::Layer::Bottom),
        WindowStacking::Overlay => gtk_layer_shell::set_layer(&window, gtk_layer_shell::Layer::Overlay),
    }
*/
