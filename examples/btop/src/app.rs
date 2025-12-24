use dioxus::prelude::*;
use dioxus_html::input_data::keyboard_types::Code;
use dioxus_tui::{on_draw, use_viewport, DrawContext, TuiContext};

use crate::components::{cpu_panel, disk_panel, mem_panel, net_panel, proc_panel_bottom, proc_panel_top, topbar};
use crate::data::MOCK_DATA;
use crate::theme;

const SCREEN_WIDTH: usize = 120;
const SCREEN_HEIGHT: usize = 28;

fn compose_screen(blocks: &[crate::components::ComponentBlock]) -> String {
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
    let viewport = use_viewport();
    let viewport = viewport.read();
    let width = viewport.width.max(1) as usize;
    let height = viewport.height.max(1) as usize;

    let topbar_data = MOCK_DATA.topbar;
    let cpu_data = MOCK_DATA.cpu;
    let mem_data = MOCK_DATA.mem;
    let disk_data = MOCK_DATA.disk;
    let net_data = MOCK_DATA.net;
    let proc_data = MOCK_DATA.proc;

    let topbar_html = topbar::render(&topbar_data).rect.render(0, 0);
    let cpu_html = cpu_panel::render(&cpu_data).rect.render(0, 0);
    let mem_html = mem_panel::render(&mem_data).rect.render(0, 0);
    let disk_html = disk_panel::render(&disk_data).rect.render(0, 0);
    let proc_top_html = proc_panel_top::render(&proc_data).rect.render(0, 0);
    let net_html = net_panel::render(&net_data).rect.render(0, 0);
    let proc_bottom_html = proc_panel_bottom::render(&proc_data).rect.render(0, 0);

    rsx! {
        div {
            width: "{width}ch",
            height: "{height}ch",
            background_color: theme::MAIN_BG,
            color: theme::MAIN_FG,
            font_size: "16px",
            line_height: "16px",
            padding: "0",
            margin: "0",
            box_sizing: "border-box",
            display: "flex",
            flex_direction: "column",

            tabindex: "0",
            onkeydown: move |e| match e.code() {
                Code::KeyQ | Code::Escape => tui.quit(),
                _ => {}
            },

            div {
                height: "1ch",
                flex_basis: "1ch",
                flex_grow: "0",
                flex_shrink: "0",
                width: "100%",
                position: "relative",
                "data-draw-id": on_draw(move |ctx: &mut DrawContext| {
                    let rect = topbar::render_with_width(&topbar_data, ctx.rect.width as usize);
                    rect.draw_to(ctx);
                }),
                {topbar_html}
            }

            div {
                height: "8ch",
                flex_basis: "8ch",
                flex_grow: "0",
                flex_shrink: "0",
                width: "100%",
                position: "relative",
                "data-draw-id": on_draw(move |ctx: &mut DrawContext| {
                    let rect = cpu_panel::render_with_width(&cpu_data, ctx.rect.width as usize);
                    rect.draw_to(ctx);
                }),
                {cpu_html}
            }

            div {
                height: "11ch",
                flex_basis: "11ch",
                flex_grow: "0",
                flex_shrink: "0",
                width: "100%",
                display: "flex",
                flex_direction: "row",
                align_items: "stretch",
                min_height: "0px",

                div {
                    width: "28ch",
                    flex_basis: "28ch",
                    height: "11ch",
                    position: "relative",
                    "data-draw-id": on_draw(move |ctx: &mut DrawContext| {
                        let rect = mem_panel::render_with_size(
                            &mem_data,
                            ctx.rect.width as usize,
                            ctx.rect.height as usize,
                        );
                        rect.draw_to(ctx);
                    }),
                    {mem_html}
                }

                div {
                    width: "26ch",
                    flex_basis: "26ch",
                    height: "11ch",
                    position: "relative",
                    "data-draw-id": on_draw(move |ctx: &mut DrawContext| {
                        let rect = disk_panel::render_with_size(
                            &disk_data,
                            ctx.rect.width as usize,
                            ctx.rect.height as usize,
                        );
                        rect.draw_to(ctx);
                    }),
                    {disk_html}
                }

                div {
                    flex_grow: "1",
                    flex_shrink: "1",
                    min_width: "0px",
                    height: "11ch",
                    position: "relative",
                    "data-draw-id": on_draw(move |ctx: &mut DrawContext| {
                        let rect = proc_panel_top::render_with_size(
                            &proc_data,
                            ctx.rect.width as usize,
                            ctx.rect.height as usize,
                        );
                        rect.draw_to(ctx);
                    }),
                    {proc_top_html}
                }
            }

            div {
                width: "100%",
                flex_grow: "1",
                flex_shrink: "1",
                min_height: "0px",
                display: "flex",
                flex_direction: "row",
                align_items: "flex-start",

                div {
                    width: "54ch",
                    flex_basis: "54ch",
                    height: "8ch",
                    position: "relative",
                    "data-draw-id": on_draw(move |ctx: &mut DrawContext| {
                        let rect = net_panel::render_with_size(
                            &net_data,
                            ctx.rect.width as usize,
                            ctx.rect.height as usize,
                        );
                        rect.draw_to(ctx);
                    }),
                    {net_html}
                }

                div {
                    flex_grow: "1",
                    flex_shrink: "1",
                    min_width: "0px",
                    height: "100%",
                    position: "relative",
                    "data-draw-id": on_draw(move |ctx: &mut DrawContext| {
                        let rect = proc_panel_bottom::render_with_size(
                            &proc_data,
                            ctx.rect.width as usize,
                            ctx.rect.height as usize,
                        );
                        rect.draw_to(ctx);
                    }),
                    {proc_bottom_html}
                }
            }
        }
    }
}

pub fn app() -> Element {
    rsx! { App {} }
}
