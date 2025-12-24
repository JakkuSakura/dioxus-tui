use dioxus_tui::builders::{LineBuilder, RectBuilder, Style};

use crate::components::ComponentBlock;
use crate::data::{ProcData, ProcRow};
use crate::theme;

const WIDTH: usize = 66;

fn header_row(line: &mut LineBuilder, border: &Style) {
    line.set_str_styled(0, "╭─┐⁴", border.clone());
    line.set_str_styled(4, "proc", theme::fg_bold(theme::TITLE));
    line.set_str_styled(
        8,
        "┌┐filter┌─────────┐per-core┌┐reverse┌┐tree┌┐< threads >┌─╮",
        border.clone(),
    );
}

fn columns_row(line: &mut LineBuilder, border: &Style) {
    line.set_str_styled(0, "│", border.clone());
    line.set_str_styled(WIDTH - 1, "│", border.clone());
    line.set_str_styled(5, "Pid:", theme::fg(theme::PROC_MISC));
    line.set_str_styled(10, "Program:", theme::fg(theme::PROC_MISC));
    line.set_str_styled(19, "Command:", theme::fg(theme::PROC_MISC));
    line.set_str_styled(41, "User:", theme::fg(theme::PROC_MISC));
    line.set_str_styled(48, "MemB", theme::fg(theme::PROC_MISC));
    line.set_str_styled(59, "Cpu%", theme::fg(theme::PROC_MISC));
    line.set_str_styled(64, "↑", theme::fg(theme::PROC_MISC));
}

fn row_line(line: &mut LineBuilder, row: &ProcRow, border: &Style) {
    line.set_str_styled(0, "│", border.clone());
    line.set_str_styled(WIDTH - 1, "│", border.clone());
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

pub fn render(data: &ProcData) -> ComponentBlock {
    let mut rect = RectBuilder::new(WIDTH, 11);
    let border = theme::fg(theme::PROC_BOX);

    if let Some(line) = rect.line_mut(0) {
        header_row(line, &border);
    }
    if let Some(line) = rect.line_mut(1) {
        columns_row(line, &border);
    }

    for (idx, row) in data.rows_top.iter().enumerate() {
        if let Some(line) = rect.line_mut(2 + idx) {
            row_line(line, row, &border);
        }
    }

    ComponentBlock {
        x: 54,
        y: 9,
        rect,
    }
}
