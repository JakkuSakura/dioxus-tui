use dioxus::prelude::*;
use dioxus_tui::Surface;

fn render(root: fn() -> Element) -> Surface {
    dioxus_tui::render_surface(root, 80, 20).expect("render positioned boxes")
}

fn position(surface: &Surface, text: &str) -> (usize, usize) {
    surface
        .lines()
        .iter()
        .enumerate()
        .find_map(|(y, line)| line.find(text).map(|x| (x, y)))
        .unwrap_or_else(|| panic!("missing {text:?}:\n{}", surface.lines().join("\n")))
}

#[test]
fn open_positioned_dialog_uses_its_computed_box() {
    fn app() -> Element {
        rsx! {
            div { style: "position:relative;width:640px;height:320px",
                p { "CONTROL" }
                dialog { open: true, style: "position:absolute;left:16px;top:32px;display:block;width:320px;height:64px;margin:0;padding:0;border:0",
                    p { "DIALOG_CONTENT" }
                }
            }
        }
    }
    let surface = render(app);
    assert_eq!(position(&surface, "CONTROL"), (0, 0));
    assert_eq!(position(&surface, "DIALOG_CONTENT"), (2, 2));
}

#[test]
fn positioned_box_does_not_advance_normal_flow() {
    fn app() -> Element {
        rsx! {
            div { style: "position:relative;width:640px;height:320px",
                p { style: "height:16px", "BEFORE" }
                div { style: "position:absolute;left:16px;top:64px;width:320px;height:128px", "OVERLAY" }
                p { style: "height:16px", "AFTER" }
            }
        }
    }
    let surface = render(app);
    assert_eq!(position(&surface, "AFTER"), (0, 1));
    assert_eq!(position(&surface, "OVERLAY"), (2, 4));
}

#[test]
fn prior_flow_does_not_move_positioned_box() {
    fn app() -> Element {
        rsx! {
            div { style: "position:relative;width:640px;height:320px",
                div { style: "height:128px", "F" }
                div { style: "position:absolute;left:16px;top:0px;width:320px;height:64px", "OVERLAY" }
            }
        }
    }
    assert_eq!(position(&render(app), "OVERLAY"), (2, 0));
}

#[test]
fn positioned_box_paints_above_later_normal_flow() {
    fn app() -> Element {
        rsx! {
            div { style: "position:relative;width:640px;height:320px",
                dialog { open: true, style: "position:absolute;left:0px;top:0px;display:block;width:320px;height:64px;margin:0;padding:0;border:0",
                    p { "OVERLAY" }
                }
                p { "UNDERLAY" }
            }
        }
    }
    assert_eq!(position(&render(app), "OVERLAY"), (0, 0));
}

#[test]
fn display_none_positioned_subtree_paints_no_characters() {
    fn app() -> Element {
        rsx! {
            div { style: "position:relative;width:640px;height:320px",
                p { "V" }
                dialog { open: true, style: "position:absolute;display:none",
                    span { "HIDDEN_MARKER" }
                }
            }
        }
    }
    let surface = render(app);
    let remaining = surface.lines().join("\n").replacen('V', "", 1);
    assert!(remaining.chars().all(char::is_whitespace), "{remaining}");
}

#[test]
fn display_none_inline_subtree_is_not_flattened_into_parent() {
    fn app() -> Element {
        rsx! {
            div { "VISIBLE" span { style: "display:none", "HIDDEN_MARKER" } }
        }
    }
    let surface = render(app);
    let remaining = surface.lines().join("\n").replacen("VISIBLE", "", 1);
    assert!(remaining.chars().all(char::is_whitespace), "{remaining}");
}

#[test]
fn negative_positioned_layer_paints_below_normal_flow() {
    fn app() -> Element {
        rsx! {
            div { style: "position:relative;width:640px;height:320px",
                p { "FLOW" }
                dialog { open: true, style: "position:absolute;z-index:-1;left:0;top:0;display:block;width:320px;height:64px;margin:0;padding:0;border:0",
                    p { "BACK_TAIL" }
                }
            }
        }
    }
    let surface = render(app);
    assert_eq!(position(&surface, "FLOW"), (0, 0));
    assert_eq!(position(&surface, "TAIL"), (5, 0));
}

#[test]
fn positioned_siblings_follow_z_index_instead_of_dom_order() {
    fn app() -> Element {
        rsx! {
            div { style: "position:relative;width:640px;height:320px",
                div { style: "position:absolute;z-index:10;left:0;top:0;width:320px;height:64px", "FRONT" }
                dialog { open: true, style: "position:absolute;z-index:1;left:0;top:0;display:block;width:320px;height:64px;margin:0;padding:0;border:0",
                    p { "BACK_TAIL" }
                }
            }
        }
    }
    let surface = render(app);
    assert_eq!(position(&surface, "FRONT"), (0, 0));
    assert_eq!(position(&surface, "TAIL"), (5, 0));
}

#[test]
fn hidden_block_does_not_advance_flow() {
    fn app() -> Element {
        rsx! {
            div {
                p { style: "height:16px", "BEFORE" }
                div { style: "display:none;height:160px", "HIDDEN_MARKER" }
                p { style: "height:16px", "AFTER" }
            }
        }
    }
    let surface = render(app);
    assert_eq!(position(&surface, "BEFORE"), (0, 0));
    assert_eq!(position(&surface, "AFTER"), (0, 1));
}

#[test]
fn visible_button_omits_hidden_descendants_from_its_label() {
    fn app() -> Element {
        rsx! {
            button {
                "BEFORE"
                span { span { style: "display:none", "HIDDEN_MARKER" } }
                "AFTER"
            }
        }
    }
    let surface = render(app);
    assert_eq!(position(&surface, "BEFOREAFTER"), (0, 0));
    let remaining = surface.lines().join("\n").replacen("BEFOREAFTER", "", 1);
    assert!(remaining.chars().all(char::is_whitespace), "{remaining}");
}

#[test]
fn negative_positioned_layer_paints_below_direct_inline_text() {
    fn app() -> Element {
        rsx! {
            div { style: "position:relative;width:640px;height:320px",
                "FLOW"
                dialog { open: true, style: "position:absolute;z-index:-1;left:0;top:0;display:block;width:320px;height:64px;margin:0;padding:0;border:0",
                    p { "BACK_TAIL" }
                }
            }
        }
    }
    let surface = render(app);
    assert_eq!(position(&surface, "FLOW"), (0, 0));
    assert_eq!(position(&surface, "TAIL"), (5, 0));
}
