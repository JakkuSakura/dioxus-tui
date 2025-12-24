use crate::components::ComponentBlock;
use crate::data::{CpuCoreData, CpuData};
use crate::render::{bar_repeat, LineBuilder};

const WIDTH: usize = 120;
const OUTER_LEFT: usize = 0;
const OUTER_RIGHT: usize = 119;
const INNER_LEFT: usize = 48;
const INNER_RIGHT: usize = 118;

fn core_cell(core: &CpuCoreData) -> String {
    format!("{:<2} {} {:>2}%", core.idx, core.bar, core.percent)
}

fn row_from_indices(cores: &[CpuCoreData], idxs: &[usize]) -> String {
    let mut parts = Vec::new();
    for idx in idxs {
        if let Some(core) = cores.get(*idx) {
            parts.push(core_cell(core));
        }
    }
    parts.join("│")
}

pub fn render(data: &CpuData) -> ComponentBlock {
    let mut lines = Vec::new();

    // Line 0
    let mut line = LineBuilder::new(WIDTH);
    line.set_char(OUTER_LEFT, '│');
    line.set_char(OUTER_RIGHT, '│');
    line.set_str(INNER_LEFT, "╭─┐");
    line.set_str(INNER_LEFT + 3, &format!("{}-", data.model));
    line.set_str(65, &format!("┌{}┐", bar_repeat('─', 43)));
    line.set_str(110, data.freq);
    line.set_str(117, "┌╮");
    lines.push(line.finish());

    // Line 1
    let mut line = LineBuilder::new(WIDTH);
    line.set_char(OUTER_LEFT, '│');
    line.set_char(INNER_LEFT, '│');
    line.set_char(INNER_RIGHT, '│');
    line.set_char(OUTER_RIGHT, '│');
    line.set_str(INNER_LEFT + 1, "CPU ");
    line.set_str(INNER_LEFT + 5, &bar_repeat('■', 48));
    line.set_str(INNER_LEFT + 56, &format!("{}%", data.total_percent));
    line.set_str(INNER_LEFT + 59, "⣀⣀⣀⣀⣀");
    line.set_str(INNER_LEFT + 66, &format!("{:>2}°C", data.temp_c));
    lines.push(line.finish());

    // Line 2
    let mut line = LineBuilder::new(WIDTH);
    line.set_char(OUTER_LEFT, '│');
    line.set_char(INNER_LEFT, '│');
    line.set_char(INNER_RIGHT, '│');
    line.set_char(OUTER_RIGHT, '│');
    let row = row_from_indices(data.cores, &[0, 4, 8, 12, 16]);
    line.set_str(INNER_LEFT + 1, &row);
    lines.push(line.finish());

    // Line 3
    let mut line = LineBuilder::new(WIDTH);
    line.set_char(OUTER_LEFT, '│');
    line.set_char(OUTER_RIGHT, '│');
    line.set_repeat(1, '⣀', INNER_LEFT - 1);
    line.set_char(INNER_LEFT, '│');
    line.set_char(INNER_RIGHT, '│');
    let row = row_from_indices(data.cores, &[1, 5, 9, 13, 17]);
    line.set_str(INNER_LEFT + 1, &row);
    lines.push(line.finish());

    // Line 4
    let mut line = LineBuilder::new(WIDTH);
    line.set_char(OUTER_LEFT, '│');
    line.set_char(OUTER_RIGHT, '│');
    line.set_repeat(1, '⠉', INNER_LEFT - 1);
    line.set_char(INNER_LEFT, '│');
    line.set_char(INNER_RIGHT, '│');
    let row = row_from_indices(data.cores, &[2, 6, 10, 14, 18]);
    line.set_str(INNER_LEFT + 1, &row);
    lines.push(line.finish());

    // Line 5
    let mut line = LineBuilder::new(WIDTH);
    line.set_char(OUTER_LEFT, '│');
    line.set_char(OUTER_RIGHT, '│');
    line.set_repeat(1, ' ', INNER_LEFT - 1);
    line.set_char(INNER_LEFT, '│');
    line.set_char(INNER_RIGHT, '│');
    let mut row = row_from_indices(data.cores, &[3, 7, 11, 15]);
    let load = format!("L {} {} {} ⣀  0%", data.load.0, data.load.1, data.load.2);
    row.push('│');
    row.push_str(&load);
    line.set_str(INNER_LEFT + 1, &row);
    lines.push(line.finish());

    // Line 6
    let mut line = LineBuilder::new(WIDTH);
    line.set_char(OUTER_LEFT, '│');
    line.set_char(OUTER_RIGHT, '│');
    line.set_str(2, data.uptime);
    line.set_str(INNER_LEFT, &format!("╰{}╯", bar_repeat('─', INNER_RIGHT - INNER_LEFT - 1)));
    lines.push(line.finish());

    // Line 7
    let mut line = LineBuilder::new(WIDTH);
    line.set_char(0, '╰');
    line.set_repeat(1, '─', WIDTH - 2);
    line.set_char(WIDTH - 1, '╯');
    lines.push(line.finish());

    ComponentBlock {
        x: 0,
        y: 1,
        lines,
    }
}
