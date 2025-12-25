use dioxus::prelude::*;
use dioxus_html::input_data::keyboard_types::Code;
use dioxus_tui::layout_helpers::{taffy_columns, ColumnSpec};
use dioxus_tui::{on_draw, DrawContext, TuiContext};

use crate::components::{cpu_panel, disk_panel, mem_panel, net_panel, proc_panel_bottom, proc_panel_top, topbar};
use crate::data::MOCK_DATA;
use crate::theme;

const SCREEN_WIDTH: usize = 120;
const SCREEN_HEIGHT: usize = 28;

struct ScreenLayout {
    topbar_h: usize,
    cpu_h: usize,
    mid_h: usize,
    bottom_h: usize,
    mem_w: usize,
    disk_w: usize,
    proc_w: usize,
    net_w: usize,
    proc_bottom_w: usize,
}

fn compute_layout(width: usize, height: usize) -> ScreenLayout {
    let topbar_h = 1usize;
    let remaining_h = height.saturating_sub(topbar_h) as u16;
    let row_specs = [
        ColumnSpec { min: 8, weight: 8.0 },
        ColumnSpec { min: 11, weight: 11.0 },
        ColumnSpec { min: 8, weight: 8.0 },
    ];
    let row_heights = taffy_columns(remaining_h.max(1), &row_specs);
    let cpu_h = row_heights.get(0).copied().unwrap_or(8) as usize;
    let mid_h = row_heights.get(1).copied().unwrap_or(11) as usize;
    let bottom_h = row_heights.get(2).copied().unwrap_or(8) as usize;

    let mid_specs = [
        ColumnSpec { min: 10, weight: 28.0 },
        ColumnSpec { min: 10, weight: 26.0 },
        ColumnSpec { min: 10, weight: 66.0 },
    ];
    let mid_cols = taffy_columns(width.max(1) as u16, &mid_specs);
    let mem_w = mid_cols.get(0).copied().unwrap_or(10) as usize;
    let disk_w = mid_cols.get(1).copied().unwrap_or(10) as usize;
    let proc_w = mid_cols.get(2).copied().unwrap_or(10) as usize;

    let bottom_specs = [
        ColumnSpec { min: 12, weight: 54.0 },
        ColumnSpec { min: 10, weight: 66.0 },
    ];
    let bottom_cols = taffy_columns(width.max(1) as u16, &bottom_specs);
    let net_w = bottom_cols.get(0).copied().unwrap_or(12) as usize;
    let proc_bottom_w = bottom_cols.get(1).copied().unwrap_or(10) as usize;

    ScreenLayout {
        topbar_h,
        cpu_h,
        mid_h,
        bottom_h,
        mem_w,
        disk_w,
        proc_w,
        net_w,
        proc_bottom_w,
    }
}

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
    let layout = compute_layout(SCREEN_WIDTH, SCREEN_HEIGHT);
    let mut y = 0;
    let mut blocks = Vec::new();

    blocks.push(crate::components::ComponentBlock {
        x: 0,
        y,
        rect: topbar::render_with_width(&MOCK_DATA.topbar, SCREEN_WIDTH),
    });
    y += layout.topbar_h;

    blocks.push(crate::components::ComponentBlock {
        x: 0,
        y,
        rect: cpu_panel::render_with_size(&MOCK_DATA.cpu, SCREEN_WIDTH, layout.cpu_h),
    });
    y += layout.cpu_h;

    blocks.push(crate::components::ComponentBlock {
        x: 0,
        y,
        rect: mem_panel::render_with_size(&MOCK_DATA.mem, layout.mem_w, layout.mid_h),
    });
    blocks.push(crate::components::ComponentBlock {
        x: layout.mem_w,
        y,
        rect: disk_panel::render_with_size(&MOCK_DATA.disk, layout.disk_w, layout.mid_h),
    });
    blocks.push(crate::components::ComponentBlock {
        x: layout.mem_w + layout.disk_w,
        y,
        rect: proc_panel_top::render_with_size(&MOCK_DATA.proc, layout.proc_w, layout.mid_h),
    });
    y += layout.mid_h;

    blocks.push(crate::components::ComponentBlock {
        x: 0,
        y,
        rect: net_panel::render_with_size(&MOCK_DATA.net, layout.net_w, layout.bottom_h),
    });
    blocks.push(crate::components::ComponentBlock {
        x: layout.net_w,
        y,
        rect: proc_panel_bottom::render_with_size(&MOCK_DATA.proc, layout.proc_bottom_w, layout.bottom_h),
    });

    let blocks = blocks;
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

    let fallback_layout = compute_layout(SCREEN_WIDTH, SCREEN_HEIGHT);
    let topbar_html = topbar::render_with_width(&topbar_data, SCREEN_WIDTH).render(0, 0);
    let cpu_html = cpu_panel::render_with_size(&cpu_data, SCREEN_WIDTH, fallback_layout.cpu_h).render(0, 0);
    let mem_html = mem_panel::render_with_size(&mem_data, fallback_layout.mem_w, fallback_layout.mid_h).render(0, 0);
    let disk_html = disk_panel::render_with_size(&disk_data, fallback_layout.disk_w, fallback_layout.mid_h).render(0, 0);
    let proc_top_html = proc_panel_top::render_with_size(&proc_data, fallback_layout.proc_w, fallback_layout.mid_h).render(0, 0);
    let net_html = net_panel::render_with_size(&net_data, fallback_layout.net_w, fallback_layout.bottom_h).render(0, 0);
    let proc_bottom_html = proc_panel_bottom::render_with_size(&proc_data, fallback_layout.proc_bottom_w, fallback_layout.bottom_h).render(0, 0);

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
                    let rect = topbar::render_with_width(&topbar_data, ctx.rect.width as usize);
                    rect.draw_to(ctx);
                }),
                div { style: "position: relative; width: 100%; height: 100%;", {topbar_html} }
            }

            div {
                style: "flex: 1 1 0; min-height: 0; width: 100%; display: flex; flex-direction: column; margin-top: 1ch;",

                div {
                    style: "flex: 8 0 0; min-height: 8ch; width: 100%;",
                    "data-draw-id": on_draw(move |ctx: &mut DrawContext| {
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
