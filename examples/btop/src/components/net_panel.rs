use dioxus_tui::builders::RectBuilder;

use crate::components::ComponentBlock;
use crate::data::NetData;
use crate::render::bar_repeat;
use crate::theme;

const WIDTH: usize = 54;

pub fn render(data: &NetData) -> ComponentBlock {
    let mut rect = RectBuilder::new(WIDTH, 8);
    let border = theme::fg(theme::NET_BOX);

    if let Some(line) = rect.line_mut(0) {
        line.set_str_styled(0, "╭─┐³", border.clone());
        line.set_str_styled(4, "net", theme::fg_bold(theme::TITLE));
        line.set_str_styled(7, "┌─────────┐sync┌┐auto┌┐zero┌┐<b ", border.clone());
        line.set_str_styled(39, data.interface, theme::fg(theme::TITLE));
        line.set_str_styled(49, " n>┌╮", border.clone());
    }

    if let Some(line) = rect.line_mut(1) {
        line.set_str_styled(0, "│", border.clone());
        line.set_str_styled(1, data.graph_top, theme::fg(theme::DOWNLOAD_MID));
        line.set_str_styled(26, "╭─┐download┌──────────────╮", border.clone());
        line.set_str_styled(53, "│", border.clone());
    }

    if let Some(line) = rect.line_mut(2) {
        line.set_str_styled(0, "│", border.clone());
        line.set_str_styled(1, data.graph_mid, theme::fg(theme::DOWNLOAD_MID));
        line.set_str_styled(
            26,
            &format!("│▼ {} ({})││", data.down_rate, data.down_rate_mib),
            theme::fg(theme::TITLE),
        );
    }

    if let Some(line) = rect.line_mut(3) {
        line.set_str_styled(0, "│", border.clone());
        line.set_str_styled(1, data.graph_solid, theme::fg(theme::DOWNLOAD_MID));
        line.set_str_styled(
            26,
            &format!("│▼ Total:         {}││", data.down_total),
            theme::fg(theme::MAIN_FG),
        );
    }

    if let Some(line) = rect.line_mut(4) {
        line.set_str_styled(0, "│", border.clone());
        line.set_str_styled(1, data.graph_bottom, theme::fg(theme::UPLOAD_MID));
        line.set_str_styled(
            26,
            &format!("│▲ {}  ({})││", data.up_rate, data.up_rate_mib),
            theme::fg(theme::TITLE),
        );
    }

    if let Some(line) = rect.line_mut(5) {
        line.set_str_styled(0, "│", border.clone());
        line.set_str_styled(1, &bar_repeat(' ', 25), theme::fg(theme::MAIN_FG));
        line.set_str_styled(
            26,
            &format!("│▲ Total:          {}││", data.up_total),
            theme::fg(theme::MAIN_FG),
        );
    }

    if let Some(line) = rect.line_mut(6) {
        line.set_str_styled(0, "│", border.clone());
        line.set_str_styled(1, data.graph_footer, theme::fg(theme::UPLOAD_MID));
        line.set_str_styled(26, "╰─┘upload└────────────────╯", border.clone());
        line.set_str_styled(53, "│", border.clone());
    }

    if let Some(line) = rect.line_mut(7) {
        line.set_str_styled(0, "╰", border.clone());
        line.set_str_styled(1, &bar_repeat('─', WIDTH - 2), border.clone());
        line.set_str_styled(WIDTH - 1, "╯", border.clone());
    }

    ComponentBlock {
        x: 0,
        y: 20,
        rect,
    }
}
