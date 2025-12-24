use crate::components::ComponentBlock;
use crate::data::{ProcData, ProcRow};
use crate::render::LineBuilder;

const WIDTH: usize = 66;

fn row_line(row: &ProcRow) -> String {
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

fn footer_line() -> String {
    let mut line = LineBuilder::new(WIDTH);
    line.set_str(0, "╰┘↑ select ↓└┘info ↵└┘terminate└┘kill└┘signals└──────────┘9/4384└╯");
    line.finish()
}

pub fn render(data: &ProcData) -> ComponentBlock {
    let mut lines = Vec::new();
    for row in data.rows_bottom.iter() {
        lines.push(row_line(row));
    }
    lines.push(footer_line());

    ComponentBlock {
        x: 54,
        y: 20,
        lines,
    }
}
