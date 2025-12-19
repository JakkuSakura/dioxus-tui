use std::any::Any;

use dioxus_tui::{ColorMode, Config};

#[path = "support/mod.rs"]
mod render_support;

#[path = "run_many_small_edit_stress.rs"]
#[allow(dead_code)]
mod run_many_small_edit_stress;

fn main() {
    let (width, height) = render_support::dims_from_args_env_or_default(100, 40);
    let cfg = Config::default().with_color_mode(ColorMode::Rgb);

    // This example expects a `usize` context (grid size).
    let size = 8usize;
    let contexts: Vec<dioxus_tui::ContextFactory> = vec![Box::new(move || Box::new(size) as Box<dyn Any>)];

    dioxus_tui::render(
        dioxus_tui::RenderRequest::new(run_many_small_edit_stress::app)
            .with_config(cfg)
            .with_size(width, height)
            .with_contexts(contexts),
    )
    .unwrap();
}
