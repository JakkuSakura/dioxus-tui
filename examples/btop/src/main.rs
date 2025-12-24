use dioxus_tui::{launch_cfg, ColorMode, Config, CustomDrawMode};
use dioxus_tui_btop::app;

fn main() {
    let cfg = Config::default()
        .with_color_mode(ColorMode::Rgb)
        .with_custom_draw_mode(CustomDrawMode::Native);
    launch_cfg(app::app, cfg).expect("launch btop");
}
