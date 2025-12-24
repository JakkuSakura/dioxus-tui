use dioxus_tui::{ColorMode, Config, RenderRequest};
use dioxus_tui_btop::app;

fn main() {
    let cfg = Config::default().with_color_mode(ColorMode::Rgb);
    dioxus_tui::render(RenderRequest::new(app::app).with_config(cfg)).unwrap();
}
