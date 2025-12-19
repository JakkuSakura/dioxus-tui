use dioxus_tui::{ColorMode, Config, RenderRequest};

#[path = "support/mod.rs"]
mod render_support;

#[path = "run_quadrants.rs"]
#[allow(dead_code)]
mod run_quadrants;

fn main() {
    let (width, height) = render_support::dims_from_args_env_or_default(100, 40);
    let cfg = Config::default().with_color_mode(ColorMode::Rgb);

    dioxus_tui::render(RenderRequest::new(run_quadrants::app)
        .with_config(cfg)
        .with_size(width, height))
    .unwrap();
}
