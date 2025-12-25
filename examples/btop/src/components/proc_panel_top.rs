use dioxus_tui::builders::{LineBuilder, RectBuilder, Style};

use crate::components::proc_table;
use crate::data::{ProcData, ProcRow};
use crate::theme;

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

fn columns_row(line: &mut LineBuilder, border: &Style, width: usize, columns: proc_table::ProcColumns) {
    line.set_str_styled(0, "│", border.clone());
    line.set_str_styled(width - 1, "│", border.clone());
    let content = proc_table::format_header(columns, width);
    line.set_str_styled(1, &content, theme::fg(theme::PROC_MISC));
}

fn row_line(line: &mut LineBuilder, row: &ProcRow, border: &Style, width: usize, columns: proc_table::ProcColumns) {
    line.set_str_styled(0, "│", border.clone());
    line.set_str_styled(width - 1, "│", border.clone());
    let content = proc_table::format_row(row, columns, width);
    line.set_str_styled(1, &content, theme::fg(theme::MAIN_FG));
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

    let columns = proc_table::compute_columns(data, width);

    if let Some(line) = rect.line_mut(0) {
        header_row(line, &border, width);
    }
    if let Some(line) = rect.line_mut(1) {
        columns_row(line, &border, width, columns);
    }

    let rows = height.saturating_sub(2);
    for idx in 0..rows {
        if let Some(line) = rect.line_mut(2 + idx) {
            if data.rows_top.is_empty() {
                blank_row(line, &border, width);
            } else {
                let row = &data.rows_top[idx % data.rows_top.len()];
                row_line(line, row, &border, width, columns);
            }
        }
    }

    rect
}
