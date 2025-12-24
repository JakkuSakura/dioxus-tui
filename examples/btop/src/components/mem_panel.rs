use crate::components::ComponentBlock;
use crate::data::MemData;
use crate::render::{bar_repeat, LineBuilder};

const WIDTH: usize = 28;

fn line_total(data: &MemData) -> String {
    let mut line = LineBuilder::new(WIDTH);
    line.set_char(0, '│');
    line.set_str(2, "Total:");
    line.set_str(18, data.total_gib);
    line.set_str(22, "GiB");
    line.set_str(26, "├─");
    line.finish()
}

fn line_used(data: &MemData) -> String {
    let mut line = LineBuilder::new(WIDTH);
    line.set_str(0, "├─Used:");
    line.set_repeat(7, '─', 11);
    line.set_str(18, data.used_gib);
    line.set_char(21, '─');
    line.set_str(22, "GiB");
    line.set_str(25, "─┤");
    line.finish()
}

fn line_bar(ch: char, percent: u8) -> String {
    let mut line = LineBuilder::new(WIDTH);
    line.set_char(0, '│');
    line.set_char(1, ' ');
    line.set_str(2, &bar_repeat(ch, 19));
    line.set_str(22, &format!("{:>2}%", percent));
    line.set_char(25, ' ');
    line.set_char(26, '│');
    line.finish()
}

fn line_available(data: &MemData) -> String {
    let mut line = LineBuilder::new(WIDTH);
    line.set_str(0, "├─Available:");
    line.set_repeat(12, '─', 6);
    line.set_str(18, data.available_gib);
    line.set_char(21, '─');
    line.set_str(22, "GiB");
    line.set_str(25, "─├─");
    line.finish()
}

fn line_cached(data: &MemData) -> String {
    let mut line = LineBuilder::new(WIDTH);
    line.set_str(0, "├─Cached:");
    line.set_repeat(9, '─', 9);
    line.set_str(18, data.cached_gib);
    line.set_char(21, '─');
    line.set_str(22, "GiB");
    line.set_str(25, "─├─");
    line.finish()
}

fn line_free(data: &MemData) -> String {
    let mut line = LineBuilder::new(WIDTH);
    line.set_str(0, "├─Free:");
    line.set_repeat(7, '─', 10);
    line.set_str(17, data.free_gib);
    line.set_char(21, '─');
    line.set_str(22, "GiB");
    line.set_str(25, "─├─");
    line.finish()
}

pub fn render(data: &MemData) -> ComponentBlock {
    let mut lines = Vec::new();

    let mut header = LineBuilder::new(WIDTH);
    header.set_str(0, "╭─┐²");
    header.set_str(4, "mem");
    header.set_str(7, "┌");
    header.set_str(8, &bar_repeat('─', 18));
    header.set_str(26, "┬─");
    lines.push(header.finish());

    lines.push(line_total(data));
    lines.push(line_used(data));
    lines.push(line_bar('⣤', data.used_pct));
    lines.push(line_available(data));
    lines.push(line_bar('⣶', data.available_pct));
    lines.push(line_cached(data));
    lines.push(line_bar('⣤', data.cached_pct));
    lines.push(line_free(data));
    lines.push(line_bar('⣀', data.free_pct));

    let mut footer = LineBuilder::new(WIDTH);
    footer.set_str(0, "╰");
    footer.set_str(1, &bar_repeat('─', 25));
    footer.set_str(26, "┴─");
    lines.push(footer.finish());

    ComponentBlock {
        x: 0,
        y: 9,
        lines,
    }
}
