use dioxus_tui::layout_helpers::{taffy_columns, ColumnSpec};

use crate::data::{ProcData, ProcRow};

#[derive(Clone, Copy)]
pub struct ProcColumns {
    pub pid: usize,
    pub name: usize,
    pub cmd: usize,
    pub user: usize,
    pub mem: usize,
    pub bar: usize,
    pub cpu: usize,
    pub tail: usize,
}

fn pad_or_trim(text: &str, width: usize) -> String {
    let mut out = String::new();
    for ch in text.chars().take(width) {
        out.push(ch);
    }
    let len = out.chars().count();
    if len < width {
        out.push_str(&" ".repeat(width - len));
    }
    out
}

fn row_lengths(row: &ProcRow) -> ProcColumns {
    ProcColumns {
        pid: format!("{:>7}", row.pid).chars().count(),
        name: row.name.chars().count(),
        cmd: row.cmd.chars().count(),
        user: row.user.chars().count(),
        mem: row.mem.chars().count(),
        bar: row.bar.chars().count(),
        cpu: row.cpu.chars().count(),
        tail: row.tail.chars().count(),
    }
}

fn max_columns(a: ProcColumns, b: ProcColumns) -> ProcColumns {
    ProcColumns {
        pid: a.pid.max(b.pid),
        name: a.name.max(b.name),
        cmd: a.cmd.max(b.cmd),
        user: a.user.max(b.user),
        mem: a.mem.max(b.mem),
        bar: a.bar.max(b.bar),
        cpu: a.cpu.max(b.cpu),
        tail: a.tail.max(b.tail),
    }
}

pub fn compute_columns(data: &ProcData, width: usize) -> ProcColumns {
    let mut cols = ProcColumns {
        pid: "Pid:".chars().count(),
        name: "Program:".chars().count(),
        cmd: "Command:".chars().count(),
        user: "User:".chars().count(),
        mem: "MemB".chars().count(),
        bar: 1,
        cpu: "Cpu%".chars().count(),
        tail: "↑".chars().count(),
    };

    for row in data.rows_top.iter().chain(data.rows_bottom.iter()) {
        cols = max_columns(cols, row_lengths(row));
    }

    let usable = width.saturating_sub(2);
    if usable == 0 {
        return cols;
    }
    let gaps = 7;
    let content_total = usable.saturating_sub(gaps);
    let specs = [
        ColumnSpec { min: cols.pid as u16, weight: cols.pid as f32 },
        ColumnSpec { min: cols.name as u16, weight: cols.name as f32 },
        ColumnSpec { min: cols.cmd as u16, weight: (cols.cmd as f32).max(8.0) },
        ColumnSpec { min: cols.user as u16, weight: cols.user as f32 },
        ColumnSpec { min: cols.mem as u16, weight: cols.mem as f32 },
        ColumnSpec { min: cols.bar as u16, weight: cols.bar as f32 },
        ColumnSpec { min: cols.cpu as u16, weight: cols.cpu as f32 },
        ColumnSpec { min: cols.tail as u16, weight: cols.tail as f32 },
    ];
    let widths = taffy_columns(content_total as u16, &specs);

    let mut iter = widths.into_iter();
    cols.pid = iter.next().unwrap_or(cols.pid as u16) as usize;
    cols.name = iter.next().unwrap_or(cols.name as u16) as usize;
    cols.cmd = iter.next().unwrap_or(cols.cmd as u16) as usize;
    cols.user = iter.next().unwrap_or(cols.user as u16) as usize;
    cols.mem = iter.next().unwrap_or(cols.mem as u16) as usize;
    cols.bar = iter.next().unwrap_or(cols.bar as u16) as usize;
    cols.cpu = iter.next().unwrap_or(cols.cpu as u16) as usize;
    cols.tail = iter.next().unwrap_or(cols.tail as u16) as usize;
    cols
}

pub fn format_header(columns: ProcColumns, width: usize) -> String {
    if width <= 2 {
        return String::new();
    }
    let mut line = String::new();
    line.push(' ');
    line.push_str(&pad_or_trim("Pid:", columns.pid));
    line.push(' ');
    line.push_str(&pad_or_trim("Program:", columns.name));
    line.push(' ');
    line.push_str(&pad_or_trim("Command:", columns.cmd));
    line.push(' ');
    line.push_str(&pad_or_trim("User:", columns.user));
    line.push(' ');
    line.push_str(&pad_or_trim("MemB", columns.mem));
    line.push(' ');
    line.push_str(&pad_or_trim("Cpu%", columns.cpu));
    line.push(' ');
    line.push_str(&pad_or_trim("↑", columns.tail));
    pad_or_trim(&line, width.saturating_sub(2))
}

pub fn format_row(row: &ProcRow, columns: ProcColumns, width: usize) -> String {
    if width <= 2 {
        return String::new();
    }
    let pid = format!("{:>7}", row.pid);
    let mut line = String::new();
    line.push(' ');
    line.push_str(&pad_or_trim(&pid, columns.pid));
    line.push(' ');
    line.push_str(&pad_or_trim(row.name, columns.name));
    line.push(' ');
    line.push_str(&pad_or_trim(row.cmd, columns.cmd));
    line.push(' ');
    line.push_str(&pad_or_trim(row.user, columns.user));
    line.push(' ');
    line.push_str(&pad_or_trim(row.mem, columns.mem));
    line.push(' ');
    line.push_str(&pad_or_trim(row.bar, columns.bar));
    line.push(' ');
    line.push_str(&pad_or_trim(row.cpu, columns.cpu));
    line.push(' ');
    line.push_str(&pad_or_trim(row.tail, columns.tail));
    pad_or_trim(&line, width.saturating_sub(2))
}
