use dioxus_tui::builders::{LineBuilder, RectBuilder, Style};

use crate::components::ComponentBlock;
use crate::data::{ProcData, ProcRow};
use crate::theme;

const BASE_WIDTH: usize = 66;
const BASE_HEIGHT: usize = 11;

fn header_row(line: &mut LineBuilder, border: &Style) {
    line.set_str_styled(0, "╭─┐⁴", border.clone());
    line.set_str_styled(4, "proc", theme::fg_bold(theme::TITLE));
    line.set_str_styled(
        8,
        "┌┐filter┌─────────┐per-core┌┐reverse┌┐tree┌┐< threads >┌─╮",
        border.clone(),
    );
}

fn columns_row(line: &mut LineBuilder, border: &Style, width: usize) {
    line.set_str_styled(0, "│", border.clone());
    line.set_str_styled(width - 1, "│", border.clone());
    line.set_str_styled(5, "Pid:", theme::fg(theme::PROC_MISC));
    line.set_str_styled(10, "Program:", theme::fg(theme::PROC_MISC));
    line.set_str_styled(19, "Command:", theme::fg(theme::PROC_MISC));
    line.set_str_styled(41, "User:", theme::fg(theme::PROC_MISC));
    line.set_str_styled(48, "MemB", theme::fg(theme::PROC_MISC));
    line.set_str_styled(59, "Cpu%", theme::fg(theme::PROC_MISC));
    line.set_str_styled(64, "↑", theme::fg(theme::PROC_MISC));
}

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

fn blank_row(line: &mut LineBuilder, border: &Style, width: usize) {
    line.set_str_styled(0, "│", border.clone());
    line.set_str_styled(width - 1, "│", border.clone());
}

pub fn render_with_size(data: &ProcData, width: usize, height: usize) -> RectBuilder {
    let width = width.max(2);
    let height = height.max(2);
    let mut rect = RectBuilder::new(width, height);
    let border = theme::fg(theme::PROC_BOX);

    if let Some(line) = rect.line_mut(0) {
        header_row(line, &border);
    }
    if let Some(line) = rect.line_mut(1) {
        columns_row(line, &border, width);
    }

    let rows = height.saturating_sub(2).min(data.rows_top.len());
    for idx in 0..rows {
        if let Some(line) = rect.line_mut(2 + idx) {
            row_line(line, &data.rows_top[idx], &border, width);
        }
    }

    for idx in (2 + rows)..height {
        if let Some(line) = rect.line_mut(idx) {
            blank_row(line, &border, width);
        }
    }

    rect
}

pub fn render(data: &ProcData) -> ComponentBlock {
    ComponentBlock {
        x: 54,
        y: 9,
        rect: render_with_size(data, BASE_WIDTH, BASE_HEIGHT),
    }
}
