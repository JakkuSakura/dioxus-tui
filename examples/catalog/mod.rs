use dioxus::prelude::Element;
use dioxus_tui::{ColorMode, Config, ImagePolicy};
use std::sync::OnceLock;

mod all_terminal_events;
mod border;
mod btop;
mod frame;
mod buttons;
mod buttons_hooks;
mod color_test;
mod dashboard;
mod dioxus_basic;
mod flex;
mod hover;
mod keys;
mod list;
mod margin;
mod quadrants;
mod readme_hello_world;
mod tabview;
mod task;
mod text;
mod widgets;

pub use frame::ExampleFrame;

pub struct AppSpec {
    pub name: &'static str,
    pub app: fn() -> Element,
    pub cfg: Config,
}

pub fn apps() -> &'static [AppSpec] {
    static APPS: OnceLock<Vec<AppSpec>> = OnceLock::new();
    APPS.get_or_init(|| {
        vec![
            AppSpec {
                name: "dashboard",
                app: dashboard::app,
                cfg: Config::default()
                    .with_color_mode(ColorMode::Ansi)
                    .with_image_policy(ImagePolicy::Inline),
            },
            AppSpec {
                name: "btop",
                app: btop::app,
                cfg: Config::default().with_color_mode(ColorMode::Rgb),
            },
            AppSpec {
                name: "text",
                app: text::app,
                cfg: Config::default().with_color_mode(ColorMode::Rgb),
            },
            AppSpec {
                name: "widgets",
                app: widgets::app,
                cfg: Config::default().with_color_mode(ColorMode::Rgb),
            },
            AppSpec {
                name: "list",
                app: list::app,
                cfg: Config::default().with_color_mode(ColorMode::Rgb),
            },
            AppSpec {
                name: "border",
                app: border::app,
                cfg: Config::default().with_color_mode(ColorMode::Rgb),
            },
            AppSpec {
                name: "buttons",
                app: buttons::app,
                cfg: Config::default().with_color_mode(ColorMode::Rgb),
            },
            AppSpec {
                name: "buttons_2",
                app: buttons_hooks::app,
                cfg: Config::default().with_color_mode(ColorMode::Rgb),
            },
            AppSpec {
                name: "color_test",
                app: color_test::app,
                cfg: Config::default().with_color_mode(ColorMode::Rgb),
            },
            AppSpec {
                name: "hover",
                app: hover::app,
                cfg: Config::default().with_color_mode(ColorMode::Rgb),
            },
            AppSpec {
                name: "keys",
                app: keys::app,
                cfg: Config::default().with_color_mode(ColorMode::Rgb),
            },
            AppSpec {
                name: "flex",
                app: flex::app,
                cfg: Config::default().with_color_mode(ColorMode::Rgb),
            },
            AppSpec {
                name: "margin",
                app: margin::app,
                cfg: Config::default().with_color_mode(ColorMode::Rgb),
            },
            AppSpec {
                name: "quadrants",
                app: quadrants::app,
                cfg: Config::default().with_color_mode(ColorMode::Rgb),
            },
            AppSpec {
                name: "task",
                app: task::app,
                cfg: Config::default().with_color_mode(ColorMode::Rgb),
            },
            AppSpec {
                name: "tabview",
                app: tabview::app,
                cfg: Config::default().with_color_mode(ColorMode::Rgb),
            },
            AppSpec {
                name: "dioxus_basic",
                app: dioxus_basic::app,
                cfg: Config::default().with_color_mode(ColorMode::Rgb),
            },
            AppSpec {
                name: "readme",
                app: readme_hello_world::app,
                cfg: Config::default().with_color_mode(ColorMode::Rgb),
            },
            AppSpec {
                name: "all_terminal_events",
                app: all_terminal_events::app,
                cfg: Config::default().with_color_mode(ColorMode::Rgb),
            },
        ]
    })
}

pub fn app_by_name(name: &str) -> Option<&'static AppSpec> {
    apps().iter().find(|s| s.name == name)
}
