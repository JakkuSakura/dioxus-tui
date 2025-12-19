use termwiz::input::{InputEvent, KeyCode, KeyEvent, Modifiers, MouseButtons, MouseEvent};
use dioxus::prelude::*;
use dioxus_html::input_data::keyboard_types::Code;
use dioxus_tui::TuiContext;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

/// The tui renderer will look for any event that has occured or any future that has resolved in a loop.
/// It will resolve at most one event per loop.
/// This future will resolve after a certain number of polls. If the number of polls is greater than the number of events triggered, and the event has not been recieved there is an issue with the event system.
struct PollN(usize);
impl PollN {
    fn new(n: usize) -> Self {
        PollN(n)
    }
}
impl Future for PollN {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Self::Output> {
        if self.0 == 0 {
            Poll::Ready(())
        } else {
            self.0 -= 1;
            Poll::Pending
        }
    }
}

fn run_launch(app: fn() -> Element, cfg: dioxus_tui::Config) {
    let _ = dioxus_tui::launch_cfg(app, cfg);
}

#[test]
#[ignore]
fn key_down() {
    run_launch(
        app,
        dioxus_tui::Config::new().with_rendering_mode(dioxus_tui::RenderingMode::Headless),
    );

    fn app() -> Element {
        let render_count = use_signal(|| 0);
        let mut render_count_handle = render_count;
        let tui_ctx: TuiContext = consume_context();

        spawn(async move {
            PollN::new(3).await;
            render_count_handle.with_mut(|x| *x + 1);
        });
        if render_count() > 2 {
            panic!("Event was not received");
        }
        // focus the element
        tui_ctx.inject_event(InputEvent::Key(KeyEvent {
            key: KeyCode::Tab,
            modifiers: Modifiers::NONE,
        }));
        tui_ctx.inject_event(InputEvent::Key(KeyEvent {
            key: KeyCode::Char('a'),
            modifiers: Modifiers::NONE,
        }));
        rsx! {
            div {
                width: "100%",
                height: "100%",
                onkeydown: move |evt| {
                    if evt.data.code() == Code::KeyA {
                        tui_ctx.quit();
                    }
                }
            }
        }
    }
}

#[test]
#[ignore]
fn mouse_down() {
    run_launch(
        app,
        dioxus_tui::Config::new().with_rendering_mode(dioxus_tui::RenderingMode::Headless),
    );

    fn app() -> Element {
        let render_count = use_signal(|| 0);
        let tui_ctx: TuiContext = consume_context();
        let mut render_count_handle = render_count;
        spawn(async move {
            PollN::new(2).await;
            render_count_handle.with_mut(|x| *x + 1);
        });
        if render_count() > 2 {
            panic!("Event was not received");
        }
        tui_ctx.inject_event(InputEvent::Mouse(MouseEvent {
            x: 0,
            y: 0,
            mouse_buttons: MouseButtons::LEFT,
            modifiers: Modifiers::NONE,
        }));
        rsx! {
            div {
                width: "100%",
                height: "100%",
                onmousedown: move |evt| {
                    assert!(
                        evt.data.held_buttons().contains(dioxus_html::input_data::MouseButton::Primary)
                    );
                    tui_ctx.quit();
                }
            }
        }
    }
}

#[test]
#[ignore]
fn mouse_up() {
    run_launch(
        app,
        dioxus_tui::Config::new().with_rendering_mode(dioxus_tui::RenderingMode::Headless),
    );

    fn app() -> Element {
        let render_count = use_signal(|| 0);
        let tui_ctx: TuiContext = consume_context();
        let mut render_count_handle = render_count;
        spawn(async move {
            PollN::new(3).await;
            render_count_handle.with_mut(|x| *x + 1);
        });
        if render_count() > 2 {
            panic!("Event was not received");
        }
        tui_ctx.inject_event(InputEvent::Mouse(MouseEvent {
            x: 0,
            y: 0,
            mouse_buttons: MouseButtons::LEFT,
            modifiers: Modifiers::NONE,
        }));
        tui_ctx.inject_event(InputEvent::Mouse(MouseEvent {
            x: 0,
            y: 0,
            mouse_buttons: MouseButtons::NONE,
            modifiers: Modifiers::NONE,
        }));
        rsx! {
            div {
                width: "100%",
                height: "100%",
                onmouseup: move |_| {
                    tui_ctx.quit();
                }
            }
        }
    }
}

#[test]
#[ignore]
fn mouse_enter() {
    run_launch(
        app,
        dioxus_tui::Config::new().with_rendering_mode(dioxus_tui::RenderingMode::Headless),
    );

    fn app() -> Element {
        let render_count = use_signal(|| 0);
        let mut render_count_handle = render_count;
        let tui_ctx: TuiContext = consume_context();
        spawn(async move {
            PollN::new(3).await;
            render_count_handle.with_mut(|x| *x + 1);
        });
        if render_count() > 2 {
            panic!("Event was not received");
        }
        tui_ctx.inject_event(InputEvent::Mouse(MouseEvent {
            x: 100,
            y: 100,
            mouse_buttons: MouseButtons::NONE,
            modifiers: Modifiers::NONE,
        }));
        tui_ctx.inject_event(InputEvent::Mouse(MouseEvent {
            x: 0,
            y: 0,
            mouse_buttons: MouseButtons::NONE,
            modifiers: Modifiers::NONE,
        }));
        rsx! {
            div {
                width: "50%",
                height: "50%",
                onmouseenter: move |_| {
                    tui_ctx.quit();
                }
            }
        }
    }
}

#[test]
#[ignore]
fn mouse_exit() {
    run_launch(
        app,
        dioxus_tui::Config::new().with_rendering_mode(dioxus_tui::RenderingMode::Headless),
    );

    fn app() -> Element {
        let render_count = use_signal(|| 0);
        let tui_ctx: TuiContext = consume_context();
        let mut render_count_handle = render_count;
        spawn(async move {
            PollN::new(3).await;
            render_count_handle.with_mut(|x| *x + 1);
        });
        if render_count() > 2 {
            panic!("Event was not received");
        }
        tui_ctx.inject_event(InputEvent::Mouse(MouseEvent {
            x: 0,
            y: 0,
            mouse_buttons: MouseButtons::NONE,
            modifiers: Modifiers::NONE,
        }));
        tui_ctx.inject_event(InputEvent::Mouse(MouseEvent {
            x: 100,
            y: 100,
            mouse_buttons: MouseButtons::NONE,
            modifiers: Modifiers::NONE,
        }));
        rsx! {
            div {
                width: "50%",
                height: "50%",
                onmouseenter: move |_| {
                    tui_ctx.quit();
                }
            }
        }
    }
}

#[test]
#[ignore]
fn mouse_move() {
    run_launch(
        app,
        dioxus_tui::Config::new().with_rendering_mode(dioxus_tui::RenderingMode::Headless),
    );

    fn app() -> Element {
        let render_count = use_signal(|| 0);
        let tui_ctx: TuiContext = consume_context();
        let mut render_count_handle = render_count;
        spawn(async move {
            PollN::new(3).await;
            render_count_handle.with_mut(|x| *x + 1);
        });
        if render_count() > 2 {
            panic!("Event was not received");
        }
        tui_ctx.inject_event(InputEvent::Mouse(MouseEvent {
            x: 40,
            y: 40,
            mouse_buttons: MouseButtons::NONE,
            modifiers: Modifiers::NONE,
        }));
        tui_ctx.inject_event(InputEvent::Mouse(MouseEvent {
            x: 60,
            y: 60,
            mouse_buttons: MouseButtons::NONE,
            modifiers: Modifiers::NONE,
        }));
        rsx! {
            div {
                width: "100%",
                height: "100%",
                onmousemove: move |_| {
                    tui_ctx.quit();
                }
            }
        }
    }
}

#[test]
#[ignore]
fn wheel() {
    run_launch(
        app,
        dioxus_tui::Config::new().with_rendering_mode(dioxus_tui::RenderingMode::Headless),
    );

    fn app() -> Element {
        let render_count = use_signal(|| 0);
        let tui_ctx: TuiContext = consume_context();
        let mut render_count_handle = render_count;
        spawn(async move {
            PollN::new(3).await;
            render_count_handle.with_mut(|x| *x + 1);
        });
        if render_count() > 2 {
            panic!("Event was not received");
        }
        tui_ctx.inject_event(InputEvent::Mouse(MouseEvent {
            x: 50,
            y: 50,
            mouse_buttons: MouseButtons::NONE,
            modifiers: Modifiers::NONE,
        }));
        tui_ctx.inject_event(InputEvent::Mouse(MouseEvent {
            x: 50,
            y: 50,
            mouse_buttons: MouseButtons::VERT_WHEEL | MouseButtons::WHEEL_POSITIVE,
            modifiers: Modifiers::NONE,
        }));
        rsx! {
            div {
                width: "100%",
                height: "100%",
                onwheel: move |evt| {
                    assert!(evt.data.delta().strip_units().y > 0.0);
                    tui_ctx.quit();
                }
            }
        }
    }
}

#[test]
#[ignore]
fn click() {
    run_launch(
        app,
        dioxus_tui::Config::new().with_rendering_mode(dioxus_tui::RenderingMode::Headless),
    );

    fn app() -> Element {
        let render_count = use_signal(|| 0);
        let tui_ctx: TuiContext = consume_context();
        let mut render_count_handle = render_count;
        spawn(async move {
            PollN::new(3).await;
            render_count_handle.with_mut(|x| *x + 1);
        });
        if render_count() > 2 {
            panic!("Event was not received");
        }
        tui_ctx.inject_event(InputEvent::Mouse(MouseEvent {
            x: 50,
            y: 50,
            mouse_buttons: MouseButtons::LEFT,
            modifiers: Modifiers::NONE,
        }));
        tui_ctx.inject_event(InputEvent::Mouse(MouseEvent {
            x: 50,
            y: 50,
            mouse_buttons: MouseButtons::NONE,
            modifiers: Modifiers::NONE,
        }));
        rsx! {
            div {
                width: "100%",
                height: "100%",
                onclick: move |_| {
                    tui_ctx.quit();
                }
            }
        }
    }
}

#[test]
#[ignore]
fn context_menu() {
    run_launch(
        app,
        dioxus_tui::Config::new().with_rendering_mode(dioxus_tui::RenderingMode::Headless),
    );

    fn app() -> Element {
        let render_count = use_signal(|| 0);
        let tui_ctx: TuiContext = consume_context();
        let mut render_count_handle = render_count;
        spawn(async move {
            PollN::new(3).await;
            render_count_handle.with_mut(|x| *x + 1);
        });
        if render_count() > 2 {
            panic!("Event was not received");
        }
        tui_ctx.inject_event(InputEvent::Mouse(MouseEvent {
            x: 50,
            y: 50,
            mouse_buttons: MouseButtons::RIGHT,
            modifiers: Modifiers::NONE,
        }));
        tui_ctx.inject_event(InputEvent::Mouse(MouseEvent {
            x: 50,
            y: 50,
            mouse_buttons: MouseButtons::NONE,
            modifiers: Modifiers::NONE,
        }));
        rsx! {
            div {
                width: "100%",
                height: "100%",
                oncontextmenu: move |_| {
                    tui_ctx.quit();
                }
            }
        }
    }
}
