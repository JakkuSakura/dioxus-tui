use dioxus::prelude::*;
use dioxus_tui::Surface;

fn render(root: fn() -> Element) -> Surface {
    dioxus_tui::render_surface(root, 80, 20).expect("render formatting context")
}

fn text_position(surface: &Surface, text: &str) -> (usize, usize) {
    surface
        .lines()
        .iter()
        .enumerate()
        .find_map(|(y, line)| line.find(text).map(|x| (x, y)))
        .unwrap_or_else(|| {
            panic!(
                "missing {text:?} in surface:\n{}",
                surface.lines().join("\n")
            )
        })
}

#[test]
fn full_height_flex_columns_keep_computed_positions() {
    fn app() -> Element {
        rsx! {
            div { style: "display:flex;width:640px;height:320px",
                div { style: "width:128px;height:320px", "LEFT_PANE" }
                div { style: "flex:1;height:320px", "RIGHT_PANE" }
            }
        }
    }
    let surface = render(app);
    assert_eq!(text_position(&surface, "LEFT_PANE"), (0, 0));
    assert_eq!(text_position(&surface, "RIGHT_PANE"), (16, 0));
}

#[test]
fn full_height_grid_columns_keep_computed_positions() {
    fn app() -> Element {
        rsx! {
            div { style: "display:grid;grid-template-columns:128px 1fr;width:640px;height:320px",
                div { "LEFT_PANE" }
                div { "RIGHT_PANE" }
            }
        }
    }
    let surface = render(app);
    assert_eq!(text_position(&surface, "LEFT_PANE"), (0, 0));
    assert_eq!(text_position(&surface, "RIGHT_PANE"), (16, 0));
}

#[test]
fn ordinary_blocks_still_stack_vertically() {
    fn app() -> Element {
        rsx! {
            div {
                div { "FIRST_BLOCK" }
                div { "SECOND_BLOCK" }
            }
        }
    }
    let surface = render(app);
    assert_eq!(text_position(&surface, "FIRST_BLOCK"), (0, 0));
    assert_eq!(text_position(&surface, "SECOND_BLOCK"), (0, 1));
}

#[test]
fn reversed_flex_column_keeps_visual_order() {
    fn app() -> Element {
        rsx! {
            div { style: "display:flex;flex-direction:column-reverse;width:640px;height:64px",
                div { style: "height:32px", "FIRST_ITEM" }
                div { style: "height:32px", "SECOND_ITEM" }
            }
        }
    }
    let surface = render(app);
    assert_eq!(text_position(&surface, "FIRST_ITEM"), (0, 2));
    assert_eq!(text_position(&surface, "SECOND_ITEM"), (0, 0));
}

#[test]
fn forward_flex_column_retains_cell_rounding_fallback() {
    fn app() -> Element {
        rsx! {
            div { style: "display:flex;flex-direction:column;width:640px;height:320px",
                div { style: "height:1px", "FIRST_ITEM" }
                div { style: "height:1px", "SECOND_ITEM" }
            }
        }
    }
    let surface = render(app);
    assert_eq!(text_position(&surface, "FIRST_ITEM"), (0, 0));
    assert_eq!(text_position(&surface, "SECOND_ITEM"), (0, 1));
}

#[test]
fn wrapped_flex_column_keeps_new_column_position() {
    fn app() -> Element {
        rsx! {
            div { style: "display:flex;flex-direction:column;flex-wrap:wrap;align-content:flex-start;width:640px;height:64px",
                div { style: "width:128px;height:32px;flex-shrink:0", "FIRST_ITEM" }
                div { style: "width:128px;height:32px;flex-shrink:0", "SECOND_ITEM" }
                div { style: "width:128px;height:32px;flex-shrink:0", "THIRD_ITEM" }
            }
        }
    }
    let surface = render(app);
    assert_eq!(text_position(&surface, "FIRST_ITEM"), (0, 0));
    assert_eq!(text_position(&surface, "SECOND_ITEM"), (0, 2));
    assert_eq!(text_position(&surface, "THIRD_ITEM"), (16, 0));
}
