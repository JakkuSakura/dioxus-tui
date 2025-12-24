use dioxus_tui::builders::{LineBuilder, RectBuilder, Style};

use crate::components::ComponentBlock;
use crate::data::{ProcData, ProcRow};
use crate::theme;

const BASE_WIDTH: usize = 66;
const BASE_HEIGHT: usize = 8;

fn row_line(line: &mut LineBuilder, row: &ProcRow, border: &Style, width: usize) {
    line.set_str_styled(0, "│", border.clone());
    line.set_str_styled(width - 1, "│", border.clone());
    line.set_str_styled(2, &format!("{:>7}", row.pid), theme::fg(theme::MAIN_FG));
    line.set_str_styled(10, row.name, theme::fg(theme::MAIN_FG));
    line.set_str_styled(19, row.cmd, theme::fg(theme::MAIN_FG));
    line.set_str_styled(41, row.user, theme::fg(theme::MAIN_FG));
    line.set_str_styled(48, &format!("{:>4}", row.mem), theme::fg(theme::MAIN_FG));
    line.set_str_styled(52, " ", theme::fg(theme::MAIN_FG));
    line.set_str_styled(53, row.bar, theme::fg(theme::PROCESS_MID));
    line.set_str_styled(58, "  ", theme::fg(theme::MAIN_FG));
    line.set_str_styled(60, row.cpu, theme::fg(theme::MAIN_FG));
    line.set_str_styled(63, " ", theme::fg(theme::MAIN_FG));
    line.set_str_styled(64, row.tail, theme::fg(theme::PROCESS_END));
}

fn footer_line(line: &mut LineBuilder, border: &Style, width: usize) {
    let footer = "╰┘↑ select ↓└┘info ↵└┘terminate└┘kill└┘signals└──────────┘9/4384└╯";
    if width >= BASE_WIDTH {
        line.set_str_styled(0, footer, border.clone());
    } else {
        line.set_str_styled(0, footer, border.clone());
    }
}

fn blank_row(line: &mut LineBuilder, border: &Style, width: usize) {
    line.set_str_styled(0, "│", border.clone());
    line.set_str_styled(width - 1, "│", border.clone());
}

pub fn render_with_size(data: &ProcData, width: usize, height: usize) -> RectBuilder {
    let width = width.max(2);
    let height = height.max(2);
    let mut rect = RectBuilder::new(width, height);
    let border = theme::fg(theme::PROC_BOX);

    let body_rows = height.saturating_sub(1).min(data.rows_bottom.len());
    for idx in 0..body_rows {
        if let Some(line) = rect.line_mut(idx) {
            row_line(line, &data.rows_bottom[idx], &border, width);
        }
    }

    for idx in body_rows..height.saturating_sub(1) {
        if let Some(line) = rect.line_mut(idx) {
            blank_row(line, &border, width);
        }
    }

    if let Some(line) = rect.line_mut(height - 1) {
        footer_line(line, &border, width);
    }

    rect
}

pub fn render(data: &ProcData) -> ComponentBlock {
    ComponentBlock {
        x: 54,
        y: 20,
        rect: render_with_size(data, BASE_WIDTH, BASE_HEIGHT),
    }
}
