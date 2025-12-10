use dioxus::prelude::Element;
use dioxus_native::{launch_cfg, launch_cfg_with_props};

/// Launch the given Dioxus app in a Blitz GUI window using dioxus-native defaults.
pub async fn launch_blitz_gui(app: fn() -> Element) {
    launch_cfg(app, Vec::new(), Vec::new());
}

/// Launch the given Dioxus app with props in a Blitz GUI window using dioxus-native defaults.
pub async fn launch_blitz_gui_with_props<P: Clone + 'static>(app: fn(P) -> Element, props: P) {
    launch_cfg_with_props(app, props, Vec::new(), Vec::new());
}
