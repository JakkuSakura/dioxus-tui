use dioxus_tui::builders::{LineBuilder, RectBuilder, Style};

use crate::components::ComponentBlock;
use crate::data::MemData;
use crate::render::bar_repeat;
use crate::theme;

const WIDTH: usize = 28;

fn line_total(line: &mut LineBuilder, data: &MemData, border: &Style) {
    line.set_str_styled(0, "│", border.clone());
    line.set_str_styled(2, "Total:", theme::fg(theme::MAIN_FG));
    line.set_str_styled(18, data.total_gib, theme::fg(theme::TITLE));
    line.set_str_styled(22, "GiB", theme::fg(theme::MAIN_FG));
    line.set_str_styled(26, "├─", border.clone());
}

fn line_used(line: &mut LineBuilder, data: &MemData, border: &Style) {
    line.set_str_styled(0, "├─Used:", border.clone());
    line.set_str_styled(7, &bar_repeat('─', 11), border.clone());
    line.set_str_styled(18, data.used_gib, theme::fg(theme::TITLE));
    line.set_str_styled(21, "─", border.clone());
    line.set_str_styled(22, "GiB", theme::fg(theme::MAIN_FG));
    line.set_str_styled(25, "─┤", border.clone());
}

fn line_bar(line: &mut LineBuilder, ch: char, percent: u8, color: &str, border: &Style) {
    line.set_str_styled(0, "│", border.clone());
    line.set_str_styled(1, " ", theme::fg(theme::MAIN_FG));
    line.set_str_styled(2, &bar_repeat(ch, 19), theme::fg(color));
    line.set_str_styled(22, &format!("{:>2}%", percent), theme::fg(theme::TITLE));
    line.set_str_styled(25, " ", theme::fg(theme::MAIN_FG));
    line.set_str_styled(26, "│", border.clone());
}

fn line_available(line: &mut LineBuilder, data: &MemData, border: &Style) {
    line.set_str_styled(0, "├─Available:", border.clone());
    line.set_str_styled(12, &bar_repeat('─', 6), border.clone());
    line.set_str_styled(18, data.available_gib, theme::fg(theme::TITLE));
    line.set_str_styled(21, "─", border.clone());
    line.set_str_styled(22, "GiB", theme::fg(theme::MAIN_FG));
    line.set_str_styled(25, "─├─", border.clone());
}

fn line_cached(line: &mut LineBuilder, data: &MemData, border: &Style) {
    line.set_str_styled(0, "├─Cached:", border.clone());
    line.set_str_styled(9, &bar_repeat('─', 9), border.clone());
    line.set_str_styled(18, data.cached_gib, theme::fg(theme::TITLE));
    line.set_str_styled(21, "─", border.clone());
    line.set_str_styled(22, "GiB", theme::fg(theme::MAIN_FG));
    line.set_str_styled(25, "─├─", border.clone());
}

fn line_free(line: &mut LineBuilder, data: &MemData, border: &Style) {
    line.set_str_styled(0, "├─Free:", border.clone());
    line.set_str_styled(7, &bar_repeat('─', 10), border.clone());
    line.set_str_styled(17, data.free_gib, theme::fg(theme::TITLE));
    line.set_str_styled(21, "─", border.clone());
    line.set_str_styled(22, "GiB", theme::fg(theme::MAIN_FG));
    line.set_str_styled(25, "─├─", border.clone());
}

pub fn render(data: &MemData) -> ComponentBlock {
    let mut rect = RectBuilder::new(WIDTH, 11);
    let border = theme::fg(theme::MEM_BOX);

    if let Some(line) = rect.line_mut(0) {
        line.set_str_styled(0, "╭─┐²", border.clone());
        line.set_str_styled(4, "mem", theme::fg_bold(theme::TITLE));
        line.set_str_styled(7, "┌", border.clone());
        line.set_str_styled(8, &bar_repeat('─', 18), border.clone());
        line.set_str_styled(26, "┬─", border.clone());
    }

    if let Some(line) = rect.line_mut(1) {
        line_total(line, data, &border);
    }
    if let Some(line) = rect.line_mut(2) {
        line_used(line, data, &border);
    }
    if let Some(line) = rect.line_mut(3) {
        line_bar(line, '⣤', data.used_pct, theme::USED_MID, &border);
    }
    if let Some(line) = rect.line_mut(4) {
        line_available(line, data, &border);
    }
    if let Some(line) = rect.line_mut(5) {
        line_bar(line, '⣶', data.available_pct, theme::AVAILABLE_MID, &border);
    }
    if let Some(line) = rect.line_mut(6) {
        line_cached(line, data, &border);
    }
    if let Some(line) = rect.line_mut(7) {
        line_bar(line, '⣤', data.cached_pct, theme::CACHED_MID, &border);
    }
    if let Some(line) = rect.line_mut(8) {
        line_free(line, data, &border);
    }
    if let Some(line) = rect.line_mut(9) {
        line_bar(line, '⣀', data.free_pct, theme::FREE_MID, &border);
    }

    if let Some(line) = rect.line_mut(10) {
        line.set_str_styled(0, "╰", border.clone());
        line.set_str_styled(1, &bar_repeat('─', 25), border.clone());
        line.set_str_styled(26, "┴─", border.clone());
    }

    ComponentBlock {
        x: 0,
        y: 9,
        rect,
    }
}
