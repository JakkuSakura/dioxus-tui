use dioxus_tui::builders::RectBuilder;

use crate::data::{CpuCoreData, CpuData};
use crate::render::bar_repeat;
use crate::theme;

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

pub fn render_with_size(data: &CpuData, width: usize, height: usize) -> RectBuilder {
    let width = width.max(2);
    let height = height.max(2);
    let mut rect = RectBuilder::new(width, height);
    let border = theme::fg(theme::CPU_BOX);
    let outer_left = 0;
    let outer_right = width - 1;
    let inner_right = width.saturating_sub(2);
    let right_corner_pos = width.saturating_sub(3);
    let freq_len = data.freq.chars().count();
    let freq_start = right_corner_pos.saturating_sub(freq_len);
    let bar_start = 65;
    let bar_len = freq_start.saturating_sub(bar_start + 2).max(1);
    let bar_row_start = INNER_LEFT + 5;
    let percent = format!("{}%", data.total_percent);
    let graph = "⣀⣀⣀⣀⣀";
    let temp = format!("{:>2}°C", data.temp_c);
    let right_len = 1 + percent.chars().count() + 1 + graph.chars().count() + 1 + temp.chars().count();
    let bar_row_len = inner_right.saturating_sub(bar_row_start + right_len).max(1);
    let percent_pos = bar_row_start + bar_row_len + 1;
    let graph_pos = percent_pos + percent.chars().count() + 1;
    let temp_pos = graph_pos + graph.chars().count() + 1;

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
            bar_row_start,
            &bar_repeat('■', bar_row_len),
            theme::fg(theme::CPU_MID),
        );
        line.set_str_styled(percent_pos, &percent, theme::fg_bold(theme::TITLE));
        line.set_str_styled(graph_pos, graph, theme::fg(theme::GRAPH_TEXT));
        line.set_str_styled(temp_pos, &temp, theme::fg(theme::TEMP_MID));
    }

    let core_rows = height.saturating_sub(4);
    for idx in 0..core_rows {
        if let Some(line) = rect.line_mut(2 + idx) {
            line.set_str_styled(outer_left, "│", border.clone());
            line.set_str_styled(outer_right, "│", border.clone());
            line.set_str_styled(INNER_LEFT, "│", border.clone());
            line.set_str_styled(inner_right, "│", border.clone());

            match idx {
                1 => line.set_str_styled(1, &bar_repeat('⣀', INNER_LEFT - 1), theme::fg(theme::GRAPH_TEXT)),
                2 => line.set_str_styled(1, &bar_repeat('⠉', INNER_LEFT - 1), theme::fg(theme::GRAPH_TEXT)),
                _ => line.set_str_styled(1, &bar_repeat(' ', INNER_LEFT - 1), theme::fg(theme::MAIN_FG)),
            }

            let base = (idx % 4) as usize;
            let indices = [base, base + 4, base + 8, base + 12, base + 16];
            let mut row = row_from_indices(data.cores, &indices);
            if idx + 1 == core_rows {
                let load = format!("L {} {} {} ⣀  0%", data.load.0, data.load.1, data.load.2);
                row.push('│');
                row.push_str(&load);
            }
            line.set_str_styled(INNER_LEFT + 1, &row, theme::fg(theme::CPU_START));
        }
    }

    // Uptime line
    if let Some(line) = rect.line_mut(height.saturating_sub(2)) {
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

    // Bottom border
    if let Some(line) = rect.line_mut(height.saturating_sub(1)) {
        line.set_str_styled(0, "╰", border.clone());
        line.set_str_styled(1, &bar_repeat('─', width.saturating_sub(2)), border.clone());
        line.set_str_styled(width - 1, "╯", border);
    }

    rect
}
