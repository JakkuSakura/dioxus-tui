use dioxus_tui::{ColorMode, Config, RenderRequest};
use dioxus_tui_btop::btop;

fn main() {
    let cfg = Config::default().with_color_mode(ColorMode::Rgb);
    dioxus_tui::render(RenderRequest::new(btop::app).with_config(cfg)).unwrap();
}
