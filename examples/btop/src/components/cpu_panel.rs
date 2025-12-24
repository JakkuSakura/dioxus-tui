use dioxus_tui::builders::RectBuilder;

use crate::components::ComponentBlock;
use crate::data::{CpuCoreData, CpuData};
use crate::render::bar_repeat;
use crate::theme;

const BASE_WIDTH: usize = 120;
const HEIGHT: usize = 8;
const INNER_LEFT: usize = 48;

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

pub fn render_with_width(data: &CpuData, width: usize) -> RectBuilder {
    let width = width.max(2);
    let mut rect = RectBuilder::new(width, HEIGHT);
    let border = theme::fg(theme::CPU_BOX);
    let outer_left = 0;
    let outer_right = width - 1;
    let inner_right = width.saturating_sub(2);
    let extra = width.saturating_sub(BASE_WIDTH);
    let right_corner_pos = width.saturating_sub(3);
    let freq_len = data.freq.chars().count();
    let freq_start = right_corner_pos.saturating_sub(freq_len);
    let bar_start = 65;
    let bar_len = freq_start.saturating_sub(bar_start + 2);

    // Line 0
    if let Some(line) = rect.line_mut(0) {
        line.set_str_styled(outer_left, "│", border.clone());
        line.set_str_styled(outer_right, "│", border.clone());
        line.set_str_styled(INNER_LEFT, "╭─┐", border.clone());
        line.set_str_styled(
            INNER_LEFT + 3,
            &format!("{}-", data.model),
            theme::fg_bold(theme::TITLE),
        );
        line.set_str_styled(bar_start, &format!("┌{}┐", bar_repeat('─', bar_len)), border.clone());
        line.set_str_styled(freq_start, data.freq, theme::fg(theme::TITLE));
        line.set_str_styled(right_corner_pos, "┌╮", border.clone());
    }

    // Line 1
    if let Some(line) = rect.line_mut(1) {
        line.set_str_styled(outer_left, "│", border.clone());
        line.set_str_styled(INNER_LEFT, "│", border.clone());
        line.set_str_styled(inner_right, "│", border.clone());
        line.set_str_styled(outer_right, "│", border.clone());
        line.set_str_styled(INNER_LEFT + 1, "CPU ", theme::fg_bold(theme::TITLE));
        line.set_str_styled(
            INNER_LEFT + 5,
            &bar_repeat('■', 48 + extra),
            theme::fg(theme::CPU_MID),
        );
        line.set_str_styled(
            INNER_LEFT + 56 + extra,
            &format!("{}%", data.total_percent),
            theme::fg_bold(theme::TITLE),
        );
        line.set_str_styled(
            INNER_LEFT + 59 + extra,
            "⣀⣀⣀⣀⣀",
            theme::fg(theme::GRAPH_TEXT),
        );
        line.set_str_styled(
            INNER_LEFT + 66 + extra,
            &format!("{:>2}°C", data.temp_c),
            theme::fg(theme::TEMP_MID),
        );
    }

    // Line 2
    if let Some(line) = rect.line_mut(2) {
        line.set_str_styled(outer_left, "│", border.clone());
        line.set_str_styled(INNER_LEFT, "│", border.clone());
        line.set_str_styled(inner_right, "│", border.clone());
        line.set_str_styled(outer_right, "│", border.clone());
        let row = row_from_indices(data.cores, &[0, 4, 8, 12, 16]);
        line.set_str_styled(INNER_LEFT + 1, &row, theme::fg(theme::CPU_START));
    }

    // Line 3
    if let Some(line) = rect.line_mut(3) {
        line.set_str_styled(outer_left, "│", border.clone());
        line.set_str_styled(outer_right, "│", border.clone());
        line.set_str_styled(1, &bar_repeat('⣀', INNER_LEFT - 1), theme::fg(theme::GRAPH_TEXT));
        line.set_str_styled(INNER_LEFT, "│", border.clone());
        line.set_str_styled(inner_right, "│", border.clone());
        let row = row_from_indices(data.cores, &[1, 5, 9, 13, 17]);
        line.set_str_styled(INNER_LEFT + 1, &row, theme::fg(theme::CPU_START));
    }

    // Line 4
    if let Some(line) = rect.line_mut(4) {
        line.set_str_styled(outer_left, "│", border.clone());
        line.set_str_styled(outer_right, "│", border.clone());
        line.set_str_styled(1, &bar_repeat('⠉', INNER_LEFT - 1), theme::fg(theme::GRAPH_TEXT));
        line.set_str_styled(INNER_LEFT, "│", border.clone());
        line.set_str_styled(inner_right, "│", border.clone());
        let row = row_from_indices(data.cores, &[2, 6, 10, 14, 18]);
        line.set_str_styled(INNER_LEFT + 1, &row, theme::fg(theme::CPU_START));
    }

    // Line 5
    if let Some(line) = rect.line_mut(5) {
        line.set_str_styled(outer_left, "│", border.clone());
        line.set_str_styled(outer_right, "│", border.clone());
        line.set_str_styled(1, &bar_repeat(' ', INNER_LEFT - 1), theme::fg(theme::MAIN_FG));
        line.set_str_styled(INNER_LEFT, "│", border.clone());
        line.set_str_styled(inner_right, "│", border.clone());
        let mut row = row_from_indices(data.cores, &[3, 7, 11, 15]);
        let load = format!("L {} {} {} ⣀  0%", data.load.0, data.load.1, data.load.2);
        row.push('│');
        row.push_str(&load);
        line.set_str_styled(INNER_LEFT + 1, &row, theme::fg(theme::CPU_START));
    }

    // Line 6
    if let Some(line) = rect.line_mut(6) {
        line.set_str_styled(outer_left, "│", border.clone());
        line.set_str_styled(outer_right, "│", border.clone());
        line.set_str_styled(2, data.uptime, theme::fg(theme::MAIN_FG));
        if inner_right > INNER_LEFT + 2 {
            line.set_str_styled(
                INNER_LEFT,
                &format!("╰{}╯", bar_repeat('─', inner_right - INNER_LEFT - 1)),
                border.clone(),
            );
        }
    }

    // Line 7
    if let Some(line) = rect.line_mut(7) {
        line.set_str_styled(0, "╰", border.clone());
        line.set_str_styled(1, &bar_repeat('─', width.saturating_sub(2)), border.clone());
        line.set_str_styled(width - 1, "╯", border);
    }

    rect
}

pub fn render(data: &CpuData) -> ComponentBlock {
    ComponentBlock {
        x: 0,
        y: 1,
        rect: render_with_width(data, BASE_WIDTH),
    }
}
