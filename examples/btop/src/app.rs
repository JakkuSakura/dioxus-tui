use dioxus::prelude::*;
use dioxus_html::input_data::keyboard_types::Code;
use dioxus_tui::builders::Style;
use dioxus_tui::TuiContext;

use crate::components::{
    cpu_panel, disk_panel, mem_panel, net_panel, proc_panel_bottom, proc_panel_top, topbar, ComponentBlock,
};
use crate::data::MOCK_DATA;
use crate::theme;

const SCREEN_WIDTH: usize = 120;
const SCREEN_HEIGHT: usize = 28;

#[derive(Clone)]
struct Cell {
    ch: char,
    style: Style,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            ch: ' ',
            style: Style::default(),
        }
    }
}

fn compose_screen(blocks: &[ComponentBlock]) -> String {
    let mut grid = vec![vec![' '; SCREEN_WIDTH]; SCREEN_HEIGHT];

    for block in blocks {
        let lines = block.lines();
        for (row_idx, line) in lines.iter().enumerate() {
            let y = block.y + row_idx;
            if y >= SCREEN_HEIGHT {
                continue;
            }
            for (col_idx, ch) in line.chars().enumerate() {
                let x = block.x + col_idx;
                if x >= SCREEN_WIDTH {
                    continue;
                }
                grid[y][x] = ch;
            }
        }
    }

    grid.into_iter()
        .map(|row| row.into_iter().collect::<String>())
        .collect::<Vec<_>>()
        .join("\n")
}

fn compose_cells(blocks: &[ComponentBlock]) -> Vec<Vec<Cell>> {
    let mut grid = vec![vec![Cell::default(); SCREEN_WIDTH]; SCREEN_HEIGHT];

    for block in blocks {
        for span in block.positioned_spans() {
            let y = span.y;
            if y >= SCREEN_HEIGHT {
                continue;
            }
            for (idx, ch) in span.text.chars().enumerate() {
                let x = span.x + idx;
                if x >= SCREEN_WIDTH {
                    continue;
                }
                grid[y][x] = Cell {
                    ch,
                    style: span.style.clone(),
                };
            }
        }
    }

    grid
}

fn cells_to_spans(cells: &[Vec<Cell>]) -> Vec<Element> {
    let mut out = Vec::new();

    for (row_idx, row) in cells.iter().enumerate() {
        let mut current_style = row.first().map(|cell| cell.style.clone()).unwrap_or_default();
        let mut buffer = String::new();

        let flush = |out: &mut Vec<Element>, style: &Style, buf: &mut String| {
            if buf.is_empty() {
                return;
            }
            let css = style.to_css();
            let text = std::mem::take(buf);
            out.push(rsx! {
                span {
                    style: "{css}",
                    "{text}"
                }
            });
        };

        for cell in row {
            if cell.style == current_style {
                buffer.push(cell.ch);
            } else {
                flush(&mut out, &current_style, &mut buffer);
                current_style = cell.style.clone();
                buffer.push(cell.ch);
            }
        }
        flush(&mut out, &current_style, &mut buffer);

        if row_idx + 1 < cells.len() {
            out.push(rsx! {
                span { "\n" }
            });
        }
    }

    out
}

pub fn render_screen_text() -> String {
    let blocks = [
        topbar::render(&MOCK_DATA.topbar),
        cpu_panel::render(&MOCK_DATA.cpu),
        mem_panel::render(&MOCK_DATA.mem),
        disk_panel::render(&MOCK_DATA.disk),
        proc_panel_top::render(&MOCK_DATA.proc),
        net_panel::render(&MOCK_DATA.net),
        proc_panel_bottom::render(&MOCK_DATA.proc),
    ];
    compose_screen(&blocks)
}

#[component]
pub fn App() -> Element {
    let tui: TuiContext = consume_context();
    let blocks = [
        topbar::render(&MOCK_DATA.topbar),
        cpu_panel::render(&MOCK_DATA.cpu),
        mem_panel::render(&MOCK_DATA.mem),
        disk_panel::render(&MOCK_DATA.disk),
        proc_panel_top::render(&MOCK_DATA.proc),
        net_panel::render(&MOCK_DATA.net),
        proc_panel_bottom::render(&MOCK_DATA.proc),
    ];
    let cells = compose_cells(&blocks);
    let spans = cells_to_spans(&cells).into_iter();

    rsx! {
        div {
            width: "120ch",
            height: "28ch",
            background_color: theme::MAIN_BG,
            color: theme::MAIN_FG,
            padding: "0",
            margin: "0",
            box_sizing: "border-box",
            position: "relative",

            tabindex: "0",
            onkeydown: move |e| match e.code() {
                Code::KeyQ | Code::Escape => tui.quit(),
                _ => {}
            },

            pre {
                style: "white-space: pre; font-family: monospace; line-height: 1em; margin: 0; padding: 0; width: 120ch; height: 28ch; position: absolute; left: 0; top: 0;",
                "data-pre": "full",
                {spans}
            }
        }
    }
}

pub fn app() -> Element {
    rsx! { App {} }
}
