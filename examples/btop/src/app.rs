use dioxus::prelude::*;
use dioxus_html::input_data::keyboard_types::Code;
use dioxus_tui::{on_draw, DrawContext, TuiContext};

use crate::components::{
    cpu_panel, disk_panel, mem_panel, net_panel, proc_panel_bottom, proc_panel_top, topbar, ComponentBlock,
};
use crate::data::MOCK_DATA;
use crate::theme;

const SCREEN_WIDTH: usize = 120;
const SCREEN_HEIGHT: usize = 28;

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

    let nodes = blocks.iter().enumerate().map(|(idx, block)| {
        let rect = block.rect.clone();
        let html = block.rect.render(0, 0);
        let style = format!(
            "position: absolute; left: {}ch; top: {}ch; width: {}ch; height: {}ch;",
            block.x,
            block.y,
            block.rect.width(),
            block.rect.height()
        );
        rsx! {
            div {
                key: "block-{idx}",
                style: "{style}",
                on_draw: on_draw(move |ctx: &mut DrawContext| {
                    rect.draw_to(ctx);
                }),
                {html}
            }
        }
    });

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

            {nodes}
        }
    }
}

pub fn app() -> Element {
    rsx! { App {} }
}
