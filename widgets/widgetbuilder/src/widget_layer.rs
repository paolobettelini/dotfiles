use gtk::prelude::*;
use gtk::ApplicationWindow;

#[cfg(feature = "wayland")]
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};

#[derive(Debug, Copy, Clone)]
pub enum Anchor {
    Start(i32),
    Center,
    End(i32),
}

#[derive(Debug, Clone)]
pub struct WidgetLayer {
    exclusive: bool,
    interactive_keyboard: bool,
    h_anchor: Anchor,
    v_anchor: Anchor,
    default_size: Option<(i32, i32)>,
    namespace: String,
}

impl WidgetLayer {
    pub fn new(
        exclusive: bool,
        interactive_keyboard: bool,
        h_anchor: Anchor,
        v_anchor: Anchor,
    ) -> Self {
        Self {
            exclusive,
            interactive_keyboard,
            h_anchor,
            v_anchor,
            default_size: None,
            namespace: "dashboard".to_string(),
        }
    }

    pub fn with_default_size(mut self, width: i32, height: i32) -> Self {
        self.default_size = Some((width, height));
        self
    }

    pub fn with_namespace(mut self, namespace: impl Into<String>) -> Self {
        self.namespace = namespace.into();
        self
    }

    pub fn apply(&self, window: &ApplicationWindow) {
        window.set_resizable(false);
        window.set_decorated(false);

        if let Some((width, height)) = self.default_size {
            window.set_default_size(width, height);
        }

        self.apply_backend(window);
    }

    #[cfg(feature = "wayland")]
    fn apply_backend(&self, window: &ApplicationWindow) {
        window.init_layer_shell();
        window.set_namespace(Some(self.namespace.as_str()));
        window.set_layer(Layer::Overlay);
        window.set_keyboard_mode(if self.interactive_keyboard {
            KeyboardMode::OnDemand
        } else {
            KeyboardMode::None
        });

        if self.exclusive {
            window.auto_exclusive_zone_enable();
        } else {
            window.set_exclusive_zone(0);
        }

        Self::set_anchors(window, self.h_anchor, self.v_anchor);
    }

    #[cfg(not(feature = "wayland"))]
    fn apply_backend(&self, _window: &ApplicationWindow) {}

    #[cfg(feature = "wayland")]
    fn set_anchors(window: &ApplicationWindow, h_anchor: Anchor, v_anchor: Anchor) {
        match h_anchor {
            Anchor::Start(margin) => {
                window.set_anchor(Edge::Left, true);
                window.set_margin(Edge::Left, margin);
            }
            Anchor::Center => {}
            Anchor::End(margin) => {
                window.set_anchor(Edge::Right, true);
                window.set_margin(Edge::Right, margin);
            }
        }

        match v_anchor {
            Anchor::Start(margin) => {
                window.set_anchor(Edge::Top, true);
                window.set_margin(Edge::Top, margin);
            }
            Anchor::Center => {}
            Anchor::End(margin) => {
                window.set_anchor(Edge::Bottom, true);
                window.set_margin(Edge::Bottom, margin);
            }
        }
    }
}
