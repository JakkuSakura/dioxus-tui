use dioxus_tui::builders::{LineBuilder, RectBuilder, Style};

use crate::components::proc_table;
use crate::data::{ProcData, ProcRow};
use crate::theme;

fn row_line(line: &mut LineBuilder, row: &ProcRow, border: &Style, width: usize, columns: proc_table::ProcColumns) {
    line.set_str_styled(0, "│", border.clone());
    line.set_str_styled(width - 1, "│", border.clone());
    let content = proc_table::format_row(row, columns, width);
    line.set_str_styled(1, &content, theme::fg(theme::MAIN_FG));
}

fn footer_line(line: &mut LineBuilder, border: &Style, width: usize) {
    let left = "╰┘↑ select ↓└┘info ↵└┘terminate└┘kill└┘signals└";
    let right = "┘9/4384└╯";
    let dash_len = width
        .saturating_sub(left.chars().count() + right.chars().count())
        .max(1);
    let footer = format!("{left}{}{right}", "─".repeat(dash_len));
    line.set_str_styled(0, &footer, border.clone());
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
    let body_rows = height.saturating_sub(1);
    for idx in 0..body_rows {
        if let Some(line) = rect.line_mut(idx) {
            if data.rows_bottom.is_empty() {
                blank_row(line, &border, width);
            } else {
                let row = &data.rows_bottom[idx % data.rows_bottom.len()];
                row_line(line, row, &border, width, columns);
            }
        }
    }

    if let Some(line) = rect.line_mut(height - 1) {
        footer_line(line, &border, width);
    }

    rect
}
