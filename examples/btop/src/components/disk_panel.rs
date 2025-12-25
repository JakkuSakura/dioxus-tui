use dioxus_tui::builders::{LineBuilder, RectBuilder, Style};

use crate::data::DiskData;
use crate::render::bar_repeat;
use crate::theme;


fn line_root(line: &mut LineBuilder, data: &DiskData, border: &Style, right_edge: usize) {
    line.set_str_styled(0, "root", theme::fg(theme::TITLE));
    line.set_str_styled(4, &bar_repeat('─', 5), border.clone());
    line.set_str_styled(9, "▼28K", theme::fg(theme::DOWNLOAD_MID));
    line.set_str_styled(13, &bar_repeat('─', 3), border.clone());
    line.set_str_styled(16, data.root_used, theme::fg(theme::TITLE));
    line.set_str_styled(20, "─", border.clone());
    line.set_str_styled(21, data.root_total, theme::fg(theme::TITLE));
    line.set_str_styled(right_edge - 1, "─", border.clone());
    line.set_str_styled(right_edge, "┤", border.clone());
}

fn line_disk_used(line: &mut LineBuilder, data: &DiskData, border: &Style, right_edge: usize) {
    line.set_str_styled(0, "Used:", theme::fg(theme::MAIN_FG));
    line.set_str_styled(6, &format!("{:>2}%", data.root_used_pct), theme::fg(theme::TITLE));
    line.set_str_styled(10, "■■■■■", theme::fg(theme::USED_MID));
    line.set_str_styled(16, data.root_used_gib, theme::fg(theme::TITLE));
    line.set_str_styled(21, "GiB", theme::fg(theme::MAIN_FG));
    line.set_str_styled(right_edge, "│", border.clone());
}

fn line_swap(line: &mut LineBuilder, data: &DiskData, border: &Style, right_edge: usize) {
    line.set_str_styled(0, "swap", theme::fg(theme::TITLE));
    line.set_str_styled(4, &bar_repeat('─', 12), border.clone());
    line.set_str_styled(16, data.swap_total, theme::fg(theme::TITLE));
    line.set_str_styled(20, "─", border.clone());
    line.set_str_styled(21, "GiB", theme::fg(theme::MAIN_FG));
    line.set_str_styled(right_edge - 1, "─", border.clone());
    line.set_str_styled(right_edge, "┤", border.clone());
}

fn line_swap_used(line: &mut LineBuilder, data: &DiskData, border: &Style, right_edge: usize) {
    line.set_str_styled(0, "Used:", theme::fg(theme::MAIN_FG));
    line.set_str_styled(6, &format!("{:>2}%", data.swap_used_pct), theme::fg(theme::TITLE));
    line.set_str_styled(10, "■■■■■", theme::fg(theme::USED_MID));
    line.set_str_styled(18, data.swap_used, theme::fg(theme::TITLE));
    line.set_str_styled(20, "Byte", theme::fg(theme::MAIN_FG));
    line.set_str_styled(right_edge, "│", border.clone());
}

fn line_proc(line: &mut LineBuilder, border: &Style, right_edge: usize) {
    line.set_str_styled(0, "proc", theme::fg(theme::TITLE));
    line.set_str_styled(4, &bar_repeat('─', 14), border.clone());
    line.set_str_styled(18, "0", theme::fg(theme::TITLE));
    line.set_str_styled(19, "─", border.clone());
    line.set_str_styled(20, "Byte", theme::fg(theme::MAIN_FG));
    line.set_str_styled(right_edge - 1, "─", border.clone());
    line.set_str_styled(right_edge, "┤", border.clone());
}

fn line_proc_used(line: &mut LineBuilder, data: &DiskData, border: &Style, right_edge: usize) {
    line.set_str_styled(0, "Used:", theme::fg(theme::MAIN_FG));
    line.set_str_styled(5, &format!("{}", data.proc_used), theme::fg(theme::TITLE));
    line.set_str_styled(10, "■■■■■", theme::fg(theme::USED_MID));
    line.set_str_styled(18, "0 Byte", theme::fg(theme::MAIN_FG));
    line.set_str_styled(right_edge, "│", border.clone());
}

fn line_efi(line: &mut LineBuilder, data: &DiskData, border: &Style, right_edge: usize) {
    line.set_str_styled(0, "efi", theme::fg(theme::TITLE));
    line.set_str_styled(3, &bar_repeat('─', 13), border.clone());
    line.set_str_styled(16, data.efi_total, theme::fg(theme::TITLE));
    line.set_str_styled(20, "─", border.clone());
    line.set_str_styled(21, "MiB", theme::fg(theme::MAIN_FG));
    line.set_str_styled(right_edge - 1, "─", border.clone());
    line.set_str_styled(right_edge, "┤", border.clone());
}

fn line_io(line: &mut LineBuilder, border: &Style, right_edge: usize) {
    line.set_str_styled(0, "IO%", theme::fg(theme::MAIN_FG));
    line.set_str_styled(3, " ", theme::fg(theme::MAIN_FG));
    line.set_str_styled(4, &bar_repeat('⣀', 20), theme::fg(theme::GRAPH_TEXT));
    line.set_str_styled(right_edge, "│", border.clone());
}

pub fn render_with_size(data: &DiskData, width: usize, height: usize) -> RectBuilder {
    let width = width.max(26);
    let height = height.max(11);
    let mut rect = RectBuilder::new(width, height);
    let border = theme::fg(theme::MEM_BOX);
    let right_edge = width.saturating_sub(1);
    let footer_y = height.saturating_sub(1);

    if let Some(line) = rect.line_mut(0) {
        line.set_str_styled(0, "┐", border.clone());
        line.set_str_styled(1, "disks", theme::fg_bold(theme::TITLE));
        line.set_str_styled(6, "┌─────────────┐", border.clone());
        line.set_str_styled(21, "io", theme::fg(theme::TITLE));
        if right_edge >= 23 {
            let dash_len = right_edge.saturating_sub(23).saturating_sub(1);
            line.set_str_styled(23, &format!("┌{}╮", bar_repeat('─', dash_len)), border.clone());
        }
    }

    if let Some(line) = rect.line_mut(1) {
        line_root(line, data, &border, right_edge);
    }
    if let Some(line) = rect.line_mut(2) {
        line_io(line, &border, right_edge);
    }
    if let Some(line) = rect.line_mut(3) {
        line_disk_used(line, data, &border, right_edge);
    }
    if let Some(line) = rect.line_mut(4) {
        line_swap(line, data, &border, right_edge);
    }
    if let Some(line) = rect.line_mut(5) {
        line_swap_used(line, data, &border, right_edge);
    }
    if let Some(line) = rect.line_mut(6) {
        line_proc(line, &border, right_edge);
    }
    if let Some(line) = rect.line_mut(7) {
        line_proc_used(line, data, &border, right_edge);
    }
    if let Some(line) = rect.line_mut(8) {
        line_efi(line, data, &border, right_edge);
    }
    if let Some(line) = rect.line_mut(9) {
        line_io(line, &border, right_edge);
    }

    let extra_start = 10;
    for y in extra_start..footer_y {
        if let Some(line) = rect.line_mut(y) {
            line_io(line, &border, right_edge);
        }
    }

    if let Some(line) = rect.line_mut(footer_y) {
        line.set_str_styled(0, &bar_repeat('─', right_edge), border.clone());
        line.set_str_styled(right_edge, "╯", border.clone());
    }

    rect
}
