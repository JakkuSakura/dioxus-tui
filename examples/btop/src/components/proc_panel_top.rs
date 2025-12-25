use dioxus_tui::builders::{LineBuilder, RectBuilder, Style};

use crate::components::ComponentBlock;
use crate::data::{ProcData, ProcRow};
use crate::theme;

const BASE_WIDTH: usize = 66;
const BASE_HEIGHT: usize = 11;

fn pad_or_trim(text: &str, width: usize) -> String {
    let mut out = String::new();
    for ch in text.chars().take(width) {
        out.push(ch);
    }
    let len = out.chars().count();
    if len < width {
        out.push_str(&" ".repeat(width - len));
    }
    out
}

fn row_content(row: &ProcRow, width: usize) -> String {
    if width <= 2 {
        return String::new();
    }
    let usable = width - 2;
    let pid = format!("{:>7}", row.pid);
    let pid_width = pid.chars().count() + 1;
    let name_width = 8;
    let user_width = 6;
    let mem_width = 4;
    let bar_width = row.bar.chars().count();
    let cpu_width = row.cpu.chars().count();
    let tail_width = row.tail.chars().count();
    let fixed = pid_width + name_width + user_width + mem_width + bar_width + cpu_width + tail_width + 5;
    let cmd_width = usable.saturating_sub(fixed).max(8);

    let mut line = String::new();
    line.push(' ');
    line.push_str(&pid);
    line.push(' ');
    line.push_str(&pad_or_trim(row.name, name_width));
    line.push(' ');
    line.push_str(&pad_or_trim(row.cmd, cmd_width));
    line.push(' ');
    line.push_str(&pad_or_trim(row.user, user_width));
    line.push(' ');
    line.push_str(&pad_or_trim(row.mem, mem_width));
    line.push(' ');
    line.push_str(row.bar);
    line.push(' ');
    line.push_str(&pad_or_trim(row.cpu, cpu_width));
    line.push(' ');
    line.push_str(row.tail);
    pad_or_trim(&line, usable)
}

fn header_content(width: usize) -> String {
    if width <= 2 {
        return String::new();
    }
    let usable = width - 2;
    let pid = "Pid:";
    let program = "Program:";
    let command = "Command:";
    let user = "User:";
    let memb = "MemB";
    let cpu = "Cpu%";
    let arrow = "↑";

    let pid_width = pid.chars().count() + 1;
    let program_width = 8;
    let user_width = 6;
    let mem_width = 4;
    let cpu_width = cpu.chars().count();
    let fixed = pid_width + program_width + user_width + mem_width + cpu_width + arrow.chars().count() + 5;
    let cmd_width = usable.saturating_sub(fixed).max(command.chars().count());

    let mut line = String::new();
    line.push(' ');
    line.push_str(pid);
    line.push(' ');
    line.push_str(&pad_or_trim(program, program_width));
    line.push(' ');
    line.push_str(&pad_or_trim(command, cmd_width));
    line.push(' ');
    line.push_str(&pad_or_trim(user, user_width));
    line.push(' ');
    line.push_str(&pad_or_trim(memb, mem_width));
    line.push(' ');
    line.push_str(cpu);
    line.push(' ');
    line.push_str(arrow);
    pad_or_trim(&line, usable)
}

fn header_row(line: &mut LineBuilder, border: &Style, width: usize) {
    line.set_str_styled(0, "╭─┐⁴", border.clone());
    line.set_str_styled(4, "proc", theme::fg_bold(theme::TITLE));
    let prefix = "┌┐filter┌─────────┐per-core┌┐reverse┌┐tree┌┐< threads >┌";
    let prefix_len = 8 + prefix.chars().count();
    line.set_str_styled(8, prefix, border.clone());
    if width > prefix_len + 1 {
        let dash_len = width.saturating_sub(prefix_len + 1);
        line.set_str_styled(prefix_len, &"─".repeat(dash_len), border.clone());
        line.set_str_styled(width - 1, "╮", border.clone());
    } else {
        line.set_str_styled(prefix_len, "╮", border.clone());
    }
}

fn columns_row(line: &mut LineBuilder, border: &Style, width: usize) {
    if width != BASE_WIDTH {
        line.set_str_styled(0, "│", border.clone());
        line.set_str_styled(width - 1, "│", border.clone());
        let content = header_content(width);
        line.set_str_styled(1, &content, theme::fg(theme::PROC_MISC));
        return;
    }
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
    if width != BASE_WIDTH {
        line.set_str_styled(0, "│", border.clone());
        line.set_str_styled(width - 1, "│", border.clone());
        let content = row_content(row, width);
        line.set_str_styled(1, &content, theme::fg(theme::MAIN_FG));
        return;
    }
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
        header_row(line, &border, width);
    }
    if let Some(line) = rect.line_mut(1) {
        columns_row(line, &border, width);
    }

    let rows = height.saturating_sub(2);
    for idx in 0..rows {
        if let Some(line) = rect.line_mut(2 + idx) {
            if data.rows_top.is_empty() {
                blank_row(line, &border, width);
            } else {
                let row = &data.rows_top[idx % data.rows_top.len()];
                row_line(line, row, &border, width);
            }
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
