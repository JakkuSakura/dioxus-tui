use crate::components::ComponentBlock;
use crate::data::NetData;
use crate::render::{bar_repeat, LineBuilder};

const WIDTH: usize = 54;

pub fn render(data: &NetData) -> ComponentBlock {
    let mut lines = Vec::new();

    let mut header = LineBuilder::new(WIDTH);
    header.set_str(0, "╭─┐³");
    header.set_str(4, "net");
    header.set_str(7, "┌─────────┐sync┌┐auto┌┐zero┌┐<b ");
    header.set_str(39, data.interface);
    header.set_str(49, " n>┌╮");
    lines.push(header.finish());

    let mut line = LineBuilder::new(WIDTH);
    line.set_char(0, '│');
    line.set_str(1, data.graph_top);
    line.set_str(26, "╭─┐download┌──────────────╮");
    line.set_char(53, '│');
    lines.push(line.finish());

    let mut line = LineBuilder::new(WIDTH);
    line.set_char(0, '│');
    line.set_str(1, data.graph_mid);
    line.set_str(26, &format!("│▼ {} ({})││", data.down_rate, data.down_rate_mib));
    lines.push(line.finish());

    let mut line = LineBuilder::new(WIDTH);
    line.set_char(0, '│');
    line.set_str(1, data.graph_solid);
    line.set_str(26, &format!("│▼ Total:         {}││", data.down_total));
    lines.push(line.finish());

    let mut line = LineBuilder::new(WIDTH);
    line.set_char(0, '│');
    line.set_str(1, data.graph_bottom);
    line.set_str(26, &format!("│▲ {}  ({})││", data.up_rate, data.up_rate_mib));
    lines.push(line.finish());

    let mut line = LineBuilder::new(WIDTH);
    line.set_char(0, '│');
    line.set_str(1, &bar_repeat(' ', 25));
    line.set_str(26, &format!("│▲ Total:          {}││", data.up_total));
    lines.push(line.finish());

    let mut line = LineBuilder::new(WIDTH);
    line.set_char(0, '│');
    line.set_str(1, data.graph_footer);
    line.set_str(26, "╰─┘upload└────────────────╯");
    line.set_char(53, '│');
    lines.push(line.finish());

    let mut footer = LineBuilder::new(WIDTH);
    footer.set_char(0, '╰');
    footer.set_str(1, &bar_repeat('─', WIDTH - 2));
    footer.set_char(WIDTH - 1, '╯');
    lines.push(footer.finish());

    ComponentBlock {
        x: 0,
        y: 20,
        lines,
    }
}
