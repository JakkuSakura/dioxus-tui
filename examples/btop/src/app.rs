use dioxus::prelude::*;
use dioxus_html::input_data::keyboard_types::Code;
use dioxus_tui::TuiContext;

use crate::components::{
    cpu_panel, disk_panel, mem_panel, net_panel, proc_panel_bottom, proc_panel_top, topbar, ComponentBlock,
};
use crate::data::MOCK_DATA;

const SCREEN_WIDTH: usize = 120;
const SCREEN_HEIGHT: usize = 28;

fn compose_screen(blocks: &[ComponentBlock]) -> String {
    let mut grid = vec![vec![' '; SCREEN_WIDTH]; SCREEN_HEIGHT];

    for block in blocks {
        for (row_idx, line) in block.lines.iter().enumerate() {
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

    let text = render_screen_text();

    rsx! {
        div {
            width: "120ch",
            height: "28ch",
            background_color: "#0b0f14",
            color: "#c0caf5",
            padding: "0",
            margin: "0",
            box_sizing: "border-box",

            tabindex: "0",
            onkeydown: move |e| match e.code() {
                Code::KeyQ | Code::Escape => tui.quit(),
                _ => {}
            },

            pre {
                style: "white-space: pre; font-family: monospace; line-height: 1em; margin: 0; padding: 0; width: 120ch; height: 28ch; position: absolute; left: 0; top: 0;",
                "data-pre": "full",
                "{text}"
            }
        }
    }
}

pub fn app() -> Element {
    rsx! { App {} }
}
