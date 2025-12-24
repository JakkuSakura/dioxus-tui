use crate::components::ComponentBlock;
use crate::data::ProcData;
use crate::render::LineBuilder;

const WIDTH: usize = 66;

fn header_row() -> String {
    let mut line = LineBuilder::new(WIDTH);
    line.set_str(0, "╭─┐⁴");
    line.set_str(4, "proc");
    line.set_str(8, "┌┐filter┌─────────┐per-core┌┐reverse┌┐tree┌┐< threads >┌─╮");
    line.finish()
}

fn columns_row() -> String {
    let mut line = LineBuilder::new(WIDTH);
    line.set_char(0, '│');
    line.set_char(WIDTH - 1, '│');
    line.set_str(5, "Pid:");
    line.set_str(10, "Program:");
    line.set_str(19, "Command:");
    line.set_str(41, "User:");
    line.set_str(48, "MemB");
    line.set_str(59, "Cpu%");
    line.set_str(64, "↑");
    line.finish()
}

fn row_line(row: &crate::data::ProcRow) -> String {
    let mut line = LineBuilder::new(WIDTH);
    line.set_char(0, '│');
    line.set_char(WIDTH - 1, '│');
    line.set_str(2, &format!("{:>7}", row.pid));
    line.set_str(10, row.name);
    line.set_str(19, row.cmd);
    line.set_str(41, row.user);
    line.set_str(48, &format!("{:>4}", row.mem));
    line.set_char(52, ' ');
    line.set_str(53, row.bar);
    line.set_str(58, "  ");
    line.set_str(60, row.cpu);
    line.set_char(63, ' ');
    line.set_str(64, row.tail);
    line.finish()
}

pub fn render(data: &ProcData) -> ComponentBlock {
    let mut lines = Vec::new();
    lines.push(header_row());
    lines.push(columns_row());
    for row in data.rows_top {
        lines.push(row_line(row));
    }

    ComponentBlock {
        x: 54,
        y: 9,
        lines,
    }
}
