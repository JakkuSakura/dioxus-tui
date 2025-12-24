use dioxus_tui::builders::{LineBuilder, RectBuilder, Style};

use crate::components::ComponentBlock;
use crate::data::DiskData;
use crate::render::bar_repeat;
use crate::theme;

const WIDTH: usize = 26;

fn line_root(line: &mut LineBuilder, data: &DiskData, border: &Style) {
    line.set_str_styled(0, "root", theme::fg(theme::TITLE));
    line.set_str_styled(4, &bar_repeat('─', 5), border.clone());
    line.set_str_styled(9, "▼28K", theme::fg(theme::DOWNLOAD_MID));
    line.set_str_styled(13, &bar_repeat('─', 3), border.clone());
    line.set_str_styled(16, data.root_used, theme::fg(theme::TITLE));
    line.set_str_styled(20, "─", border.clone());
    line.set_str_styled(21, data.root_total, theme::fg(theme::TITLE));
    line.set_str_styled(24, "─", border.clone());
    line.set_str_styled(25, "┤", border.clone());
}

fn line_disk_used(line: &mut LineBuilder, data: &DiskData, border: &Style) {
    line.set_str_styled(0, "Used:", theme::fg(theme::MAIN_FG));
    line.set_str_styled(6, &format!("{:>2}%", data.root_used_pct), theme::fg(theme::TITLE));
    line.set_str_styled(10, "■■■■■", theme::fg(theme::USED_MID));
    line.set_str_styled(16, data.root_used_gib, theme::fg(theme::TITLE));
    line.set_str_styled(21, "GiB", theme::fg(theme::MAIN_FG));
    line.set_str_styled(25, "│", border.clone());
}

fn line_swap(line: &mut LineBuilder, data: &DiskData, border: &Style) {
    line.set_str_styled(0, "swap", theme::fg(theme::TITLE));
    line.set_str_styled(4, &bar_repeat('─', 12), border.clone());
    line.set_str_styled(16, data.swap_total, theme::fg(theme::TITLE));
    line.set_str_styled(20, "─", border.clone());
    line.set_str_styled(21, "GiB", theme::fg(theme::MAIN_FG));
    line.set_str_styled(24, "─", border.clone());
    line.set_str_styled(25, "┤", border.clone());
}

fn line_swap_used(line: &mut LineBuilder, data: &DiskData, border: &Style) {
    line.set_str_styled(0, "Used:", theme::fg(theme::MAIN_FG));
    line.set_str_styled(6, &format!("{:>2}%", data.swap_used_pct), theme::fg(theme::TITLE));
    line.set_str_styled(10, "■■■■■", theme::fg(theme::USED_MID));
    line.set_str_styled(18, data.swap_used, theme::fg(theme::TITLE));
    line.set_str_styled(20, "Byte", theme::fg(theme::MAIN_FG));
    line.set_str_styled(25, "│", border.clone());
}

fn line_proc(line: &mut LineBuilder, border: &Style) {
    line.set_str_styled(0, "proc", theme::fg(theme::TITLE));
    line.set_str_styled(4, &bar_repeat('─', 14), border.clone());
    line.set_str_styled(18, "0", theme::fg(theme::TITLE));
    line.set_str_styled(19, "─", border.clone());
    line.set_str_styled(20, "Byte", theme::fg(theme::MAIN_FG));
    line.set_str_styled(24, "─", border.clone());
    line.set_str_styled(25, "┤", border.clone());
}

fn line_proc_used(line: &mut LineBuilder, data: &DiskData, border: &Style) {
    line.set_str_styled(0, "Used:", theme::fg(theme::MAIN_FG));
    line.set_str_styled(5, &format!("{}", data.proc_used), theme::fg(theme::TITLE));
    line.set_str_styled(10, "■■■■■", theme::fg(theme::USED_MID));
    line.set_str_styled(18, "0 Byte", theme::fg(theme::MAIN_FG));
    line.set_str_styled(25, "│", border.clone());
}

fn line_efi(line: &mut LineBuilder, data: &DiskData, border: &Style) {
    line.set_str_styled(0, "efi", theme::fg(theme::TITLE));
    line.set_str_styled(3, &bar_repeat('─', 13), border.clone());
    line.set_str_styled(16, data.efi_total, theme::fg(theme::TITLE));
    line.set_str_styled(20, "─", border.clone());
    line.set_str_styled(21, "MiB", theme::fg(theme::MAIN_FG));
    line.set_str_styled(24, "─", border.clone());
    line.set_str_styled(25, "┤", border.clone());
}

fn line_io(line: &mut LineBuilder, border: &Style) {
    line.set_str_styled(0, "IO%", theme::fg(theme::MAIN_FG));
    line.set_str_styled(3, " ", theme::fg(theme::MAIN_FG));
    line.set_str_styled(4, &bar_repeat('⣀', 20), theme::fg(theme::GRAPH_TEXT));
    line.set_str_styled(25, "│", border.clone());
}

pub fn render(data: &DiskData) -> ComponentBlock {
    let mut rect = RectBuilder::new(WIDTH, 11);
    let border = theme::fg(theme::MEM_BOX);

    if let Some(line) = rect.line_mut(0) {
        line.set_str_styled(0, "┐", border.clone());
        line.set_str_styled(1, "disks", theme::fg_bold(theme::TITLE));
        line.set_str_styled(6, "┌─────────────┐", border.clone());
        line.set_str_styled(21, "io", theme::fg(theme::TITLE));
        line.set_str_styled(23, "┌─╮", border.clone());
    }

    if let Some(line) = rect.line_mut(1) {
        line_root(line, data, &border);
    }
    if let Some(line) = rect.line_mut(2) {
        line_io(line, &border);
    }
    if let Some(line) = rect.line_mut(3) {
        line_disk_used(line, data, &border);
    }
    if let Some(line) = rect.line_mut(4) {
        line_swap(line, data, &border);
    }
    if let Some(line) = rect.line_mut(5) {
        line_swap_used(line, data, &border);
    }
    if let Some(line) = rect.line_mut(6) {
        line_proc(line, &border);
    }
    if let Some(line) = rect.line_mut(7) {
        line_proc_used(line, data, &border);
    }
    if let Some(line) = rect.line_mut(8) {
        line_efi(line, data, &border);
    }
    if let Some(line) = rect.line_mut(9) {
        line_io(line, &border);
    }

    if let Some(line) = rect.line_mut(10) {
        line.set_str_styled(0, &bar_repeat('─', 25), border.clone());
        line.set_str_styled(25, "╯", border.clone());
    }

    ComponentBlock {
        x: 28,
        y: 9,
        rect,
    }
}
