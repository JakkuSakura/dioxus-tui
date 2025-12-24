use crate::components::ComponentBlock;
use crate::data::DiskData;
use crate::render::{bar_repeat, LineBuilder};

const WIDTH: usize = 26;

fn line_root(data: &DiskData) -> String {
    let mut line = LineBuilder::new(WIDTH);
    line.set_str(0, "root");
    line.set_repeat(4, '─', 5);
    line.set_str(9, "▼28K");
    line.set_repeat(13, '─', 3);
    line.set_str(16, data.root_used);
    line.set_char(20, '─');
    line.set_str(21, data.root_total);
    line.set_char(24, '─');
    line.set_char(25, '┤');
    line.finish()
}

fn line_disk_used(data: &DiskData) -> String {
    let mut line = LineBuilder::new(WIDTH);
    line.set_str(0, "Used:");
    line.set_str(6, &format!("{:>2}%", data.root_used_pct));
    line.set_str(10, "■■■■■");
    line.set_str(16, data.root_used_gib);
    line.set_str(21, "GiB");
    line.set_char(25, '│');
    line.finish()
}

fn line_swap(data: &DiskData) -> String {
    let mut line = LineBuilder::new(WIDTH);
    line.set_str(0, "swap");
    line.set_repeat(4, '─', 12);
    line.set_str(16, data.swap_total);
    line.set_char(20, '─');
    line.set_str(21, "GiB");
    line.set_char(24, '─');
    line.set_char(25, '┤');
    line.finish()
}

fn line_swap_used(data: &DiskData) -> String {
    let mut line = LineBuilder::new(WIDTH);
    line.set_str(0, "Used:");
    line.set_str(6, &format!("{:>2}%", data.swap_used_pct));
    line.set_str(10, "■■■■■");
    line.set_str(18, data.swap_used);
    line.set_str(20, "Byte");
    line.set_char(25, '│');
    line.finish()
}

fn line_proc() -> String {
    let mut line = LineBuilder::new(WIDTH);
    line.set_str(0, "proc");
    line.set_repeat(4, '─', 14);
    line.set_str(18, "0");
    line.set_char(19, '─');
    line.set_str(20, "Byte");
    line.set_char(24, '─');
    line.set_char(25, '┤');
    line.finish()
}

fn line_proc_used(data: &DiskData) -> String {
    let mut line = LineBuilder::new(WIDTH);
    line.set_str(0, "Used:");
    line.set_str(5, &format!("{}", data.proc_used));
    line.set_str(10, "■■■■■");
    line.set_str(18, "0 Byte");
    line.set_char(25, '│');
    line.finish()
}

fn line_efi(data: &DiskData) -> String {
    let mut line = LineBuilder::new(WIDTH);
    line.set_str(0, "efi");
    line.set_repeat(3, '─', 13);
    line.set_str(16, data.efi_total);
    line.set_char(20, '─');
    line.set_str(21, "MiB");
    line.set_char(24, '─');
    line.set_char(25, '┤');
    line.finish()
}

fn line_io() -> String {
    let mut line = LineBuilder::new(WIDTH);
    line.set_str(0, "IO%");
    line.set_char(3, ' ');
    line.set_str(4, &bar_repeat('⣀', 20));
    line.set_char(25, '│');
    line.finish()
}

pub fn render(data: &DiskData) -> ComponentBlock {
    let mut lines = Vec::new();

    let mut header = LineBuilder::new(WIDTH);
    header.set_char(0, '┐');
    header.set_str(1, "disks");
    header.set_str(6, "┌─────────────┐");
    header.set_str(21, "io");
    header.set_str(23, "┌─╮");
    lines.push(header.finish());

    lines.push(line_root(data));
    lines.push(line_io());
    lines.push(line_disk_used(data));
    lines.push(line_swap(data));
    lines.push(line_swap_used(data));
    lines.push(line_proc());
    lines.push(line_proc_used(data));
    lines.push(line_efi(data));
    lines.push(line_io());

    let mut footer = LineBuilder::new(WIDTH);
    footer.set_str(0, &bar_repeat('─', 25));
    footer.set_char(25, '╯');
    lines.push(footer.finish());

    ComponentBlock {
        x: 28,
        y: 9,
        lines,
    }
}
