use dioxus_tui::{launch_cfg, ColorMode, Config};
use dioxus_tui_btop::app;

fn main() {
    let cfg = Config::default().with_color_mode(ColorMode::Rgb);
    launch_cfg(app::app, cfg).expect("launch btop");
}
