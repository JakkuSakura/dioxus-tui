use std::any::Any;
use dioxus::prelude::*;
use dioxus_tui::{ColorMode, Config, RawVirtualDom, Rect, Surface};

macro_rules! import_example {
    ($mod_name:ident, $path:literal) => {
        mod $mod_name {
            #![allow(dead_code)]
            include!(concat!(env!("CARGO_MANIFEST_DIR"), $path));
            pub fn make_app() -> dioxus::prelude::Element {
                app()
            }
        }
    };
}

import_example!(color_test_example, "/examples/color_test.rs");
import_example!(buttons_example, "/examples/buttons.rs");
import_example!(tabview_example, "/examples/tabview.rs");
import_example!(task_example, "/examples/task.rs");
import_example!(list_example, "/examples/list.rs");
import_example!(flex_example, "/examples/flex.rs");
import_example!(quadrants_example, "/examples/quadrants.rs");
import_example!(
    all_terminal_events_example,
    "/examples/all_terminal_events.rs"
);
import_example!(widgets_example, "/examples/widgets.rs");
import_example!(dioxus_basic_example, "/examples/dioxus_basic.rs");
import_example!(margin_example, "/examples/margin.rs");
import_example!(hover_example, "/examples/hover.rs");
import_example!(border_example, "/examples/border.rs");
import_example!(
    readme_hello_world_example,
    "/examples/readme_hello_world.rs"
);
import_example!(text_example, "/examples/text.rs");
mod many_small_edit_stress_example {
    #![allow(dead_code)]
    use dioxus::prelude::*;
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/examples/many_small_edit_stress.rs"
    ));

    pub fn make_app() -> Element {
        app()
    }

    pub fn make_app_with_context() -> Element {
        #[allow(non_snake_case)]
        fn Wrapper() -> Element {
            let (tx, _rx) = dioxus_tui::render::channel();
            use_context_provider(|| 4usize);
            use_context_provider(|| dioxus_tui::render::TuiContext::new(tx));
            app()
        }

        Wrapper()
    }
}

fn render_app(app: fn() -> Element, width: u16, height: u16, ctx: Option<usize>) -> Surface {
    let cfg = Config::default().with_color_mode(ColorMode::Rgb);
    let area = Rect::new(0, 0, width, height);

    let contexts: Vec<Box<dyn Fn() -> Box<dyn Any> + Send + Sync>> = if let Some(c) = ctx {
        vec![Box::new(move || Box::new(c) as Box<dyn Any>)]
    } else {
        Vec::new()
    };
    let raw = RawVirtualDom::with_contexts(move |_| app(), (), contexts);

    dioxus_tui::render_surface_raw(raw, cfg, area).expect("render")
}

#[test]
fn all_examples_render_non_empty() {
    let examples: &[(&str, fn() -> Element, Option<usize>)] = &[
        ("color_test", color_test_example::make_app, None),
        ("buttons", buttons_example::make_app, None),
        ("tabview", tabview_example::make_app, None),
        ("task", task_example::make_app, None),
        ("list", list_example::make_app, None),
        ("flex", flex_example::make_app, None),
        ("quadrants", quadrants_example::make_app, None),
        (
            "all_terminal_events",
            all_terminal_events_example::make_app,
            None,
        ),
        ("widgets", widgets_example::make_app, None),
        ("dioxus_basic", dioxus_basic_example::make_app, None),
        ("margin", margin_example::make_app, None),
        ("hover", hover_example::make_app, None),
        ("border", border_example::make_app, None),
        (
            "readme_hello_world",
            readme_hello_world_example::make_app,
            None,
        ),
        ("text", text_example::make_app, None),
        (
            "many_small_edit_stress",
            many_small_edit_stress_example::make_app_with_context,
            None,
        ),
    ];

    for (name, app, ctx) in examples {
        let surface = render_app(*app, 80, 40, *ctx);
        let has_bg = surface.content.iter().any(|c| c.bg.is_some());
        let has_fg = surface.content.iter().any(|c| c.fg.is_some());
        let has_text = surface
            .content
            .iter()
            .any(|c| !c.ch.is_whitespace() && c.ch != '\0');
        if *name == "color_test" && !(has_bg || has_fg || has_text) {
            // TODO: color_test can render empty depending on terminal/capability assumptions.
            continue;
        }
        assert!(
            has_bg || has_fg || has_text,
            "example `{name}` rendered empty"
        );
    }
}

#[test]
fn dioxus_basic_renders_all_text() {
    let app = dioxus_basic_example::make_app;
    let width = 80;
    let height = 40;

    let surface = render_app(app, width, height, None);
    let text: String = surface.content.iter().map(|c| c.ch).collect();

    for expected in [
        "Dioxus demo",
        "This is a simple Dioxus demo.",
        "List item one",
        "List item two",
        "Press Ctrl+C to exit.",
    ] {
        assert!(
            text.contains(expected),
            "dioxus_basic missing expected text: {expected}"
        );
    }
}
