use dioxus_tui::builders::RectBuilder;
use dioxus_tui::layout_helpers::{taffy_columns, ColumnSpec};

use crate::data::NetData;
use crate::render::bar_repeat;
use crate::theme;

fn pad_or_trim(text: &str, width: usize) -> String {
    let mut out = String::new();
    for ch in text.chars().take(width) {
        out.push(ch);
    }
    let len = out.chars().count();
    if len < width {
        out.push_str(&bar_repeat(' ', width - len));
    }
    out
}

fn box_header(label: &str, box_width: usize) -> String {
    if box_width == 0 {
        return String::new();
    }
    let prefix = format!("╭─┐{label}┌");
    let prefix_len = prefix.chars().count();
    let dash_len = box_width.saturating_sub(prefix_len + 1);
    pad_or_trim(&format!("{prefix}{}╮", bar_repeat('─', dash_len)), box_width)
}

fn box_footer(label: &str, box_width: usize) -> String {
    if box_width == 0 {
        return String::new();
    }
    let prefix = format!("╰─┘{label}└");
    let prefix_len = prefix.chars().count();
    let dash_len = box_width.saturating_sub(prefix_len + 1);
    pad_or_trim(&format!("{prefix}{}╯", bar_repeat('─', dash_len)), box_width)
}

fn box_line(text: &str, box_width: usize) -> String {
    if box_width == 0 {
        return String::new();
    }
    let mut line = String::from("│");
    line.push_str(text);
    let len = line.chars().count();
    let pad = box_width.saturating_sub(len + 1);
    line.push_str(&bar_repeat(' ', pad));
    line.push('│');
    pad_or_trim(&line, box_width)
}

pub fn render_with_size(data: &NetData, width: usize, height: usize) -> RectBuilder {
    let width = width.max(2);
    let height = height.max(8);
    let mut rect = RectBuilder::new(width, height);
    let border = theme::fg(theme::NET_BOX);
    let right = width - 1;
    let footer_y = height.saturating_sub(2);
    let graph_min = [
        data.graph_top,
        data.graph_mid,
        data.graph_solid,
        data.graph_bottom,
        data.graph_footer,
    ]
    .iter()
    .map(|s| s.chars().count())
    .max()
    .unwrap_or(1)
    .max(1);
    let box_min = [
        format!("▼ {} ({})", data.down_rate, data.down_rate_mib).chars().count(),
        format!("▼ Total:         {}", data.down_total).chars().count(),
        format!("▲ {}  ({})", data.up_rate, data.up_rate_mib).chars().count(),
        format!("▲ Total:          {}", data.up_total).chars().count(),
        "download".chars().count() + 4,
        "upload".chars().count() + 4,
    ]
    .into_iter()
    .max()
    .unwrap_or(1)
    .max(1)
    + 2;

    let inner_width = width.saturating_sub(2).max(1) as u16;
    let cols = [
        ColumnSpec {
            min: graph_min as u16,
            weight: graph_min as f32,
        },
        ColumnSpec {
            min: box_min as u16,
            weight: box_min as f32,
        },
    ];
    let col_widths = taffy_columns(inner_width, &cols);
    let graph_width = col_widths.get(0).copied().unwrap_or(graph_min as u16) as usize;
    let box_width = col_widths.get(1).copied().unwrap_or(box_min as u16) as usize;
    let box_start = 1 + graph_width;

    if let Some(line) = rect.line_mut(0) {
        line.set_str_styled(0, "╭─┐³", border.clone());
        line.set_str_styled(4, "net", theme::fg_bold(theme::TITLE));
        let prefix = "┌─────────┐sync┌┐auto┌┐zero┌┐<b ";
        line.set_str_styled(7, prefix, border.clone());
        let iface_x = 7 + prefix.chars().count();
        line.set_str_styled(iface_x, data.interface, theme::fg(theme::TITLE));
        let suffix_x = iface_x + data.interface.chars().count();
        line.set_str_styled(suffix_x, " n>", border.clone());
        let header_start = suffix_x + 3;
        if right > header_start {
            let dash_len = right.saturating_sub(header_start + 1);
            line.set_str_styled(header_start, &format!("┌{}╮", bar_repeat('─', dash_len)), border.clone());
        } else {
            line.set_str_styled(header_start, "┌╮", border.clone());
        }
    }

    if let Some(line) = rect.line_mut(1) {
        line.set_str_styled(0, "│", border.clone());
        let graph = pad_or_trim(data.graph_top, graph_width);
        line.set_str_styled(1, &graph, theme::fg(theme::DOWNLOAD_MID));
        line.set_str_styled(box_start, &box_header("download", box_width), border.clone());
        line.set_str_styled(right, "│", border.clone());
    }

    if let Some(line) = rect.line_mut(2) {
        line.set_str_styled(0, "│", border.clone());
        let graph = pad_or_trim(data.graph_mid, graph_width);
        line.set_str_styled(1, &graph, theme::fg(theme::DOWNLOAD_MID));
        let content = format!("▼ {} ({})", data.down_rate, data.down_rate_mib);
        line.set_str_styled(box_start, &box_line(&content, box_width), theme::fg(theme::TITLE));
        line.set_str_styled(right, "│", border.clone());
    }

    if let Some(line) = rect.line_mut(3) {
        line.set_str_styled(0, "│", border.clone());
        let graph = pad_or_trim(data.graph_solid, graph_width);
        line.set_str_styled(1, &graph, theme::fg(theme::DOWNLOAD_MID));
        let content = format!("▼ Total:         {}", data.down_total);
        line.set_str_styled(box_start, &box_line(&content, box_width), theme::fg(theme::MAIN_FG));
        line.set_str_styled(right, "│", border.clone());
    }

    if let Some(line) = rect.line_mut(4) {
        line.set_str_styled(0, "│", border.clone());
        let graph = pad_or_trim(data.graph_bottom, graph_width);
        line.set_str_styled(1, &graph, theme::fg(theme::UPLOAD_MID));
        let content = format!("▲ {}  ({})", data.up_rate, data.up_rate_mib);
        line.set_str_styled(box_start, &box_line(&content, box_width), theme::fg(theme::TITLE));
        line.set_str_styled(right, "│", border.clone());
    }

    if let Some(line) = rect.line_mut(5) {
        line.set_str_styled(0, "│", border.clone());
        let graph = pad_or_trim("", graph_width);
        line.set_str_styled(1, &graph, theme::fg(theme::MAIN_FG));
        let content = format!("▲ Total:          {}", data.up_total);
        line.set_str_styled(box_start, &box_line(&content, box_width), theme::fg(theme::MAIN_FG));
        line.set_str_styled(right, "│", border.clone());
    }

    let extra_start = 6;
    if footer_y > extra_start {
        for y in extra_start..footer_y {
            if let Some(line) = rect.line_mut(y) {
                line.set_str_styled(0, "│", border.clone());
                let (graph, color) = match (y - extra_start) % 3 {
                    0 => (data.graph_mid, theme::DOWNLOAD_MID),
                    1 => (data.graph_solid, theme::DOWNLOAD_MID),
                    _ => (data.graph_bottom, theme::UPLOAD_MID),
                };
                let graph = pad_or_trim(graph, graph_width);
                line.set_str_styled(1, &graph, theme::fg(color));
                line.set_str_styled(right, "│", border.clone());
            }
        }
    }

    if let Some(line) = rect.line_mut(footer_y) {
        line.set_str_styled(0, "│", border.clone());
        let graph = pad_or_trim(data.graph_footer, graph_width);
        line.set_str_styled(1, &graph, theme::fg(theme::UPLOAD_MID));
        line.set_str_styled(box_start, &box_footer("upload", box_width), border.clone());
        line.set_str_styled(right, "│", border.clone());
    }

    if let Some(line) = rect.line_mut(height.saturating_sub(1)) {
        line.set_str_styled(0, "╰", border.clone());
        line.set_str_styled(1, &bar_repeat('─', width.saturating_sub(2)), border.clone());
        line.set_str_styled(right, "╯", border.clone());
    }

    rect
}
