use dioxus::prelude::*;
use dioxus_html::input_data::keyboard_types::Code;
use dioxus_tui::{on_draw, DrawContext, TuiContext};

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
            style: "width: 100%; height: 100%; display: flex; flex-direction: column;",
            background_color: theme::MAIN_BG,
            color: theme::MAIN_FG,
            padding: "0",
            margin: "0",
            box_sizing: "border-box",

            tabindex: "0",
            onkeydown: move |e| match e.code() {
                Code::KeyQ | Code::Escape => tui.quit(),
                _ => {}
            },

            div {
                style: "flex: 0 0 1ch; height: 1ch; width: 100%;",
                "data-draw-id": on_draw(move |ctx: &mut DrawContext| {
                    if std::env::var("BTOP_DEBUG_LAYOUT").is_ok() {
                        eprintln!("topbar rect: {:#?}", ctx.rect);
                    }
                    let rect = topbar::render_with_width(&topbar_data, ctx.rect.width as usize);
                    rect.draw_to(ctx);
                }),
                div { style: "position: relative; width: 100%; height: 100%;", {topbar_html} }
            }

            div {
                style: "flex: 1 1 0; min-height: 0; width: 100%; display: flex; flex-direction: column;",

                div {
                    style: "flex: 8 0 0; min-height: 8ch; width: 100%;",
                    "data-draw-id": on_draw(move |ctx: &mut DrawContext| {
                        if std::env::var("BTOP_DEBUG_LAYOUT").is_ok() {
                            eprintln!("cpu rect: {:#?}", ctx.rect);
                        }
                        let rect = cpu_panel::render_with_size(
                            &cpu_data,
                            ctx.rect.width as usize,
                            ctx.rect.height as usize,
                        );
                        rect.draw_to(ctx);
                    }),
                    div { style: "position: relative; width: 100%; height: 100%;", {cpu_html} }
                }

                div {
                    style: "flex: 11 0 0; min-height: 11ch; width: 100%; display: flex; flex-direction: row;",

                    div {
                        style: "flex: 28 0 0; min-width: 10ch; height: 100%;",
                        "data-draw-id": on_draw(move |ctx: &mut DrawContext| {
                            let rect = mem_panel::render_with_size(
                                &mem_data,
                                ctx.rect.width as usize,
                                ctx.rect.height as usize,
                            );
                            rect.draw_to(ctx);
                        }),
                        div { style: "position: relative; width: 100%; height: 100%;", {mem_html} }
                    }

                    div {
                        style: "flex: 26 0 0; min-width: 10ch; height: 100%;",
                        "data-draw-id": on_draw(move |ctx: &mut DrawContext| {
                            let rect = disk_panel::render_with_size(
                                &disk_data,
                                ctx.rect.width as usize,
                                ctx.rect.height as usize,
                            );
                            rect.draw_to(ctx);
                        }),
                        div { style: "position: relative; width: 100%; height: 100%;", {disk_html} }
                    }

                    div {
                        style: "flex: 66 1 0; min-width: 0ch; height: 100%;",
                        "data-draw-id": on_draw(move |ctx: &mut DrawContext| {
                            let rect = proc_panel_top::render_with_size(
                                &proc_data,
                                ctx.rect.width as usize,
                                ctx.rect.height as usize,
                            );
                            rect.draw_to(ctx);
                        }),
                        div { style: "position: relative; width: 100%; height: 100%;", {proc_top_html} }
                    }
                }

                div {
                    style: "flex: 8 1 0; min-height: 8ch; width: 100%; display: flex; flex-direction: row;",

                    div {
                        style: "flex: 54 0 0; min-width: 12ch; height: 100%;",
                        "data-draw-id": on_draw(move |ctx: &mut DrawContext| {
                            let rect = net_panel::render_with_size(
                                &net_data,
                                ctx.rect.width as usize,
                                ctx.rect.height as usize,
                            );
                            rect.draw_to(ctx);
                        }),
                        div { style: "position: relative; width: 100%; height: 100%;", {net_html} }
                    }

                    div {
                        style: "flex: 66 1 0; min-width: 0ch; height: 100%;",
                        "data-draw-id": on_draw(move |ctx: &mut DrawContext| {
                            let rect = proc_panel_bottom::render_with_size(
                                &proc_data,
                                ctx.rect.width as usize,
                                ctx.rect.height as usize,
                            );
                            rect.draw_to(ctx);
                        }),
                        div { style: "position: relative; width: 100%; height: 100%;", {proc_bottom_html} }
                    }
                }
            }
        }
    }
}

pub fn app() -> Element {
    rsx! { App {} }
}
