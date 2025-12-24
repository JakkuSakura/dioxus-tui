use dioxus_tui::builders::RectBuilder;

use crate::components::ComponentBlock;
use crate::data::{CpuCoreData, CpuData};
use crate::render::bar_repeat;
use crate::theme;

const WIDTH: usize = 120;
const OUTER_LEFT: usize = 0;
const OUTER_RIGHT: usize = 119;
const INNER_LEFT: usize = 48;
const INNER_RIGHT: usize = 118;

fn core_cell(core: &CpuCoreData) -> String {
    format!("{:<2} {} {:>2}%", core.idx, core.bar, core.percent)
}

fn row_from_indices(cores: &[CpuCoreData], idxs: &[usize]) -> String {
    let mut parts = Vec::new();
    for idx in idxs {
        if let Some(core) = cores.get(*idx) {
            parts.push(core_cell(core));
        }
    }
    parts.join("│")
}

pub fn render(data: &CpuData) -> ComponentBlock {
    let mut rect = RectBuilder::new(WIDTH, 8);
    let border = theme::fg(theme::CPU_BOX);

    // Line 0
    if let Some(line) = rect.line_mut(0) {
        line.set_str_styled(OUTER_LEFT, "│", border.clone());
        line.set_str_styled(OUTER_RIGHT, "│", border.clone());
        line.set_str_styled(INNER_LEFT, "╭─┐", border.clone());
        line.set_str_styled(
            INNER_LEFT + 3,
            &format!("{}-", data.model),
            theme::fg_bold(theme::TITLE),
        );
        line.set_str_styled(65, &format!("┌{}┐", bar_repeat('─', 43)), border.clone());
        line.set_str_styled(110, data.freq, theme::fg(theme::TITLE));
        line.set_str_styled(117, "┌╮", border.clone());
    }

    // Line 1
    if let Some(line) = rect.line_mut(1) {
        line.set_str_styled(OUTER_LEFT, "│", border.clone());
        line.set_str_styled(INNER_LEFT, "│", border.clone());
        line.set_str_styled(INNER_RIGHT, "│", border.clone());
        line.set_str_styled(OUTER_RIGHT, "│", border.clone());
        line.set_str_styled(INNER_LEFT + 1, "CPU ", theme::fg_bold(theme::TITLE));
        line.set_str_styled(INNER_LEFT + 5, &bar_repeat('■', 48), theme::fg(theme::CPU_MID));
        line.set_str_styled(
            INNER_LEFT + 56,
            &format!("{}%", data.total_percent),
            theme::fg_bold(theme::TITLE),
        );
        line.set_str_styled(
            INNER_LEFT + 59,
            "⣀⣀⣀⣀⣀",
            theme::fg(theme::GRAPH_TEXT),
        );
        line.set_str_styled(
            INNER_LEFT + 66,
            &format!("{:>2}°C", data.temp_c),
            theme::fg(theme::TEMP_MID),
        );
    }

    // Line 2
    if let Some(line) = rect.line_mut(2) {
        line.set_str_styled(OUTER_LEFT, "│", border.clone());
        line.set_str_styled(INNER_LEFT, "│", border.clone());
        line.set_str_styled(INNER_RIGHT, "│", border.clone());
        line.set_str_styled(OUTER_RIGHT, "│", border.clone());
        let row = row_from_indices(data.cores, &[0, 4, 8, 12, 16]);
        line.set_str_styled(INNER_LEFT + 1, &row, theme::fg(theme::CPU_START));
    }

    // Line 3
    if let Some(line) = rect.line_mut(3) {
        line.set_str_styled(OUTER_LEFT, "│", border.clone());
        line.set_str_styled(OUTER_RIGHT, "│", border.clone());
        line.set_str_styled(1, &bar_repeat('⣀', INNER_LEFT - 1), theme::fg(theme::GRAPH_TEXT));
        line.set_str_styled(INNER_LEFT, "│", border.clone());
        line.set_str_styled(INNER_RIGHT, "│", border.clone());
        let row = row_from_indices(data.cores, &[1, 5, 9, 13, 17]);
        line.set_str_styled(INNER_LEFT + 1, &row, theme::fg(theme::CPU_START));
    }

    // Line 4
    if let Some(line) = rect.line_mut(4) {
        line.set_str_styled(OUTER_LEFT, "│", border.clone());
        line.set_str_styled(OUTER_RIGHT, "│", border.clone());
        line.set_str_styled(1, &bar_repeat('⠉', INNER_LEFT - 1), theme::fg(theme::GRAPH_TEXT));
        line.set_str_styled(INNER_LEFT, "│", border.clone());
        line.set_str_styled(INNER_RIGHT, "│", border.clone());
        let row = row_from_indices(data.cores, &[2, 6, 10, 14, 18]);
        line.set_str_styled(INNER_LEFT + 1, &row, theme::fg(theme::CPU_START));
    }

    // Line 5
    if let Some(line) = rect.line_mut(5) {
        line.set_str_styled(OUTER_LEFT, "│", border.clone());
        line.set_str_styled(OUTER_RIGHT, "│", border.clone());
        line.set_str_styled(1, &bar_repeat(' ', INNER_LEFT - 1), theme::fg(theme::MAIN_FG));
        line.set_str_styled(INNER_LEFT, "│", border.clone());
        line.set_str_styled(INNER_RIGHT, "│", border.clone());
        let mut row = row_from_indices(data.cores, &[3, 7, 11, 15]);
        let load = format!("L {} {} {} ⣀  0%", data.load.0, data.load.1, data.load.2);
        row.push('│');
        row.push_str(&load);
        line.set_str_styled(INNER_LEFT + 1, &row, theme::fg(theme::CPU_START));
    }

    // Line 6
    if let Some(line) = rect.line_mut(6) {
        line.set_str_styled(OUTER_LEFT, "│", border.clone());
        line.set_str_styled(OUTER_RIGHT, "│", border.clone());
        line.set_str_styled(2, data.uptime, theme::fg(theme::MAIN_FG));
        line.set_str_styled(
            INNER_LEFT,
            &format!("╰{}╯", bar_repeat('─', INNER_RIGHT - INNER_LEFT - 1)),
            border.clone(),
        );
    }

    // Line 7
    if let Some(line) = rect.line_mut(7) {
        line.set_str_styled(0, "╰", border.clone());
        line.set_str_styled(1, &bar_repeat('─', WIDTH - 2), border.clone());
        line.set_str_styled(WIDTH - 1, "╯", border);
    }

    ComponentBlock {
        x: 0,
        y: 1,
        rect,
    }
}
