use dioxus::prelude::*;
use dioxus_html::input_data::keyboard_types::Key;
use dioxus_html::point_interaction::InteractionElementOffset;

pub mod textarea;
pub use textarea::{
    TextareaAction, TextareaView, TextareaViewModel, TextareaViewProps, TextBuffer,
    use_textarea_view_model,
};

#[derive(Clone, Props, PartialEq)]
pub struct ScrollbarViewProps {
    pub content_rows: usize,
    pub viewport_rows: usize,
    #[props(optional)]
    pub scroll_row: Option<usize>,
    #[props(default = 0)]
    pub initial_row: usize,
    #[props(default = 16.0)]
    pub row_height_px: f32,
    #[props(default = 1)]
    pub scroll_step: usize,
    #[props(optional)]
    pub on_scroll: Option<EventHandler<usize>>,
    pub children: Element,
}

#[component]
pub fn ScrollbarView(props: ScrollbarViewProps) -> Element {
    let ScrollbarViewProps {
        content_rows,
        viewport_rows,
        scroll_row,
        initial_row,
        row_height_px,
        scroll_step,
        on_scroll,
        children,
    } = props;

    let max_scroll = content_rows.saturating_sub(viewport_rows);
    let mut scroll_row_signal = use_signal(|| initial_row.min(max_scroll));
    let mut dragging = use_signal(|| false);
    let mut drag_anchor = use_signal(|| 0.0f64);

    let mut set_scroll = move |row: usize| {
        let row = row.min(max_scroll);
        if scroll_row == Some(row) {
            return;
        }
        if row != scroll_row_signal() {
            scroll_row_signal.set(row);
            if let Some(handler) = &on_scroll {
                handler.call(row);
            }
        }
    };

    let scroll_row = scroll_row.unwrap_or(scroll_row_signal());

    let track_px = (viewport_rows as f32) * row_height_px;
    let thumb_rows = if content_rows == 0 {
        viewport_rows.max(1)
    } else {
        let ratio = (viewport_rows as f32) / (content_rows as f32);
        ((ratio * viewport_rows as f32).ceil() as usize).max(1)
    };
    let thumb_px = (thumb_rows as f32) * row_height_px;
    let max_thumb_px = (track_px - thumb_px).max(0.0);
    let scroll_ratio = if max_scroll == 0 { 0.0 } else { scroll_row as f32 / max_scroll as f32 };
    let thumb_top_px = scroll_ratio * max_thumb_px;

    let content_offset_px = (scroll_row as f32) * row_height_px;

    let on_key = move |e: KeyboardEvent| {
        match e.key() {
            Key::ArrowUp => set_scroll(scroll_row.saturating_sub(scroll_step)),
            Key::ArrowDown => set_scroll(scroll_row.saturating_add(scroll_step)),
            Key::PageUp => set_scroll(scroll_row.saturating_sub(viewport_rows.max(1))),
            Key::PageDown => set_scroll(scroll_row.saturating_add(viewport_rows.max(1))),
            Key::Home => set_scroll(0),
            Key::End => set_scroll(max_scroll),
            _ => {}
        }
    };

    let on_wheel = move |e: WheelEvent| {
        let delta = e.delta().strip_units().y;
        if delta == 0.0 {
            return;
        }
        if delta > 0.0 {
            set_scroll(scroll_row.saturating_add(scroll_step));
        } else {
            set_scroll(scroll_row.saturating_sub(scroll_step));
        }
    };

    let on_track_down = move |e: MouseEvent| {
        let y = e.element_coordinates().y;
        let y = y.max(0.0) as f32;
        let thumb_start = thumb_top_px;
        let thumb_end = thumb_top_px + thumb_px;
        if y >= thumb_start && y <= thumb_end {
            dragging.set(true);
            drag_anchor.set(y as f64 - thumb_start as f64);
            return;
        }
        let ratio = if track_px <= thumb_px { 0.0 } else { y / (track_px - thumb_px) };
        let new_row = (ratio * max_scroll as f32).round() as usize;
        set_scroll(new_row);
    };

    let on_track_move = move |e: MouseEvent| {
        if !dragging() {
            return;
        }
        let y = e.element_coordinates().y as f64;
        let anchor = drag_anchor();
        let max_top = (track_px - thumb_px).max(0.0) as f64;
        let top = (y - anchor).clamp(0.0, max_top);
        let ratio = if max_top == 0.0 { 0.0 } else { (top / max_top) as f32 };
        let new_row = (ratio * max_scroll as f32).round() as usize;
        set_scroll(new_row);
    };

    let on_track_up = move |_e: MouseEvent| {
        dragging.set(false);
    };

    rsx! {
        div {
            width: "100%",
            height: "100%",
            display: "flex",
            flex_direction: "row",
            gap: "1px",
            tabindex: "0",
            onkeydown: on_key,
            onwheel: on_wheel,

            div {
                width: "100%",
                height: "100%",
                overflow: "hidden",

                div {
                    width: "100%",
                    height: "100%",
                    margin_top: format!("-{}px", content_offset_px),
                    {children}
                }
            }

            div {
                width: "2px",
                height: format!("{}px", track_px.max(row_height_px)),
                background_color: "rgba(255,255,255,0.1)",
                display: "flex",
                flex_direction: "column",
                onmousedown: on_track_down,
                onmousemove: on_track_move,
                onmouseup: on_track_up,

                div {
                    height: format!("{}px", thumb_top_px.max(0.0)),
                    width: "100%",
                }
                div {
                    height: format!("{}px", thumb_px.max(row_height_px)),
                    width: "100%",
                    background_color: "rgba(255,255,255,0.6)",
                }
            }
        }
    }
}
