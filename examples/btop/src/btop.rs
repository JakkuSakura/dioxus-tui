use dioxus::prelude::*;
use dioxus_html::input_data::keyboard_types::Code;
use dioxus_tui::TuiContext;

struct CpuCore {
    idx: u8,
    usage: u8,
}

struct DiskRow {
    name: &'static str,
    used: u16,
    total: u16,
}

impl DiskRow {
    fn percent_used(&self) -> u8 {
        (self.used as u32 * 100 / self.total.max(1) as u32) as u8
    }
}

struct NetRow {
    name: &'static str,
    down_mb: f32,
    up_mb: f32,
    total_down_gb: f32,
    total_up_gb: f32,
}

struct ProcessRow {
    pid: u32,
    user: &'static str,
    cpu: f32,
    mem_gb: f32,
    time: &'static str,
    cmd: &'static str,
}

const SPARK: [char; 8] = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];

fn meter_blocks(percent: u8, width: usize) -> String {
    let filled = (percent as usize * width) / 100;
    let mut out = String::with_capacity(width);
    for i in 0..width {
        out.push(if i < filled { '█' } else { '░' });
    }
    out
}

fn sparkline(points: &[u8], width: usize) -> String {
    points
        .iter()
        .take(width)
        .map(|p| {
            let idx = (*p as usize * (SPARK.len() - 1)) / 100;
            SPARK[idx]
        })
        .collect()
}

pub fn app() -> Element {
    let tui: TuiContext = consume_context();

    let cpu_total = 1u8;
    let cpu_temp = 38u8;
    let cpu = [
        CpuCore {
            idx: 0,
            usage: 1,
        },
        CpuCore {
            idx: 1,
            usage: 0,
        },
        CpuCore {
            idx: 2,
            usage: 0,
        },
        CpuCore {
            idx: 3,
            usage: 0,
        },
        CpuCore {
            idx: 4,
            usage: 0,
        },
        CpuCore {
            idx: 5,
            usage: 1,
        },
        CpuCore {
            idx: 6,
            usage: 1,
        },
        CpuCore {
            idx: 7,
            usage: 0,
        },
    ];

    let disks = [
        DiskRow {
            name: "root",
            used: 87,
            total: 100,
        },
        DiskRow {
            name: "swap",
            used: 0,
            total: 8,
        },
        DiskRow {
            name: "efi",
            used: 1,
            total: 1,
        },
    ];

    let net = NetRow {
        name: "ens10f0np0",
        down_mb: 2.27,
        up_mb: 0.73,
        total_down_gb: 1009.0,
        total_up_gb: 250.0,
    };

    let procs = [
        ProcessRow {
            pid: 9050,
            user: "root",
            cpu: 0.3,
            mem_gb: 67.0,
            time: "05:22:10",
            cmd: "/usr/bin/kvm -id 102",
        },
        ProcessRow {
            pid: 6762,
            user: "root",
            cpu: 0.0,
            mem_gb: 0.01,
            time: "02:11:33",
            cmd: "/usr/sbin/tailscaled",
        },
        ProcessRow {
            pid: 9719,
            user: "root",
            cpu: 0.0,
            mem_gb: 6.8,
            time: "01:33:14",
            cmd: "/usr/bin/kvm -id 105",
        },
        ProcessRow {
            pid: 3549518,
            user: "jakku",
            cpu: 0.0,
            mem_gb: 0.01,
            time: "00:02:01",
            cmd: "btop",
        },
    ];

    let cpu_history = [5, 3, 2, 4, 3, 2, 3, 4, 3, 2, 5, 4, 3, 2, 4, 3];
    let net_down = [5, 10, 35, 80, 60, 40, 20, 10, 5, 8, 6, 10, 20, 30, 40, 15];
    let net_up = [2, 4, 6, 10, 8, 5, 3, 2, 1, 2, 4, 6, 8, 10, 12, 6];
    let core_lines: Vec<String> = cpu
        .iter()
        .map(|core| format!("{:>2} {} {:>2}%", core.idx, meter_blocks(core.usage, 10), core.usage))
        .collect();
    let proc_lines: Vec<String> = procs
        .iter()
        .map(|p| {
            format!(
                "{:>5}  {:<6} {:>4.1}  {:>5.2}G  {:<8} {}",
                p.pid, p.user, p.cpu, p.mem_gb, p.time, p.cmd
            )
        })
        .collect();

    rsx! {
        div {
            width: "100%",
            height: "100%",
            display: "flex",
            flex_direction: "column",
            background_color: "#0b0f14",
            color: "#c0caf5",
            padding: "0.5ch",
            box_sizing: "border-box",

            tabindex: "0",
            onkeydown: move |e| match e.code() {
                Code::KeyQ | Code::Escape => tui.quit(),
                _ => {}
            },

            div {
                display: "flex",
                flex_direction: "row",
                justify_content: "space-between",
                align_items: "center",
                padding: "0.2ch 0.5ch",
                background_color: "#121621",
                border_style: "solid",
                border_width: "1px",
                border_color: "#2f354d",
                font_family: "monospace",

                span { color: "#7aa2f7", "cpu" }
                span { color: "#a9b1d6", "menu" }
                span { color: "#a9b1d6", "preset" }
                span { color: "#565f89", "2000ms" }
                span { color: "#a9b1d6", "17:50:35" }
            }

            div {
                display: "flex",
                flex_direction: "column",
                gap: "0.6ch",
                padding_top: "0.6ch",
                min_height: "0px",
                flex_grow: "1",

                Panel { title: "CPU", accent: "#7aa2f7", wide: Some(true),
                    div {
                        display: "flex",
                        flex_direction: "column",
                        gap: "0.2ch",
                        div {
                            display: "flex",
                            flex_direction: "row",
                            justify_content: "space-between",
                            span { color: "#7dcfff", "EPYC 9965" }
                            span { color: "#a9b1d6", "{cpu_total}%" }
                            span { color: "#9ece6a", "{cpu_temp}°C" }
                        }
                        div {
                            color: "#9ece6a",
                            font_family: "monospace",
                            "{meter_blocks(cpu_total, 52)}"
                        }
                        div {
                            display: "flex",
                            flex_direction: "row",
                            flex_wrap: "wrap",
                            gap: "0.2ch",
                            for line in core_lines.iter() {
                                div {
                                    flex_basis: "16ch",
                                    min_width: "16ch",
                                    font_family: "monospace",
                                    color: "#a9b1d6",
                                    "{line}"
                                }
                            }
                        }
                        div { color: "#565f89", "history {sparkline(&cpu_history, 16)}" }
                    }
                }

                div {
                    display: "flex",
                    flex_direction: "row",
                    flex_wrap: "wrap",
                    gap: "0.6ch",

                    Panel { title: "Memory", accent: "#bb9af7",
                        div {
                            display: "flex",
                            flex_direction: "column",
                            gap: "0.2ch",
                            div { color: "#a9b1d6", "Total: 376 GiB" }
                            div { color: "#a9b1d6", "Used: 119 GiB" }
                            div { color: "#9ece6a", font_family: "monospace", "{meter_blocks(32, 24)}" }
                            div { color: "#a9b1d6", "Available: 257 GiB" }
                            div { color: "#a9b1d6", "Cached: 197 GiB" }
                            div { color: "#565f89", "{sparkline(&cpu_history, 16)}" }
                        }
                    }

                    Panel { title: "Disks", accent: "#9ece6a",
                        div {
                            display: "flex",
                            flex_direction: "column",
                            gap: "0.2ch",
                            for disk in disks.iter() {
                                div {
                                    display: "flex",
                                    flex_direction: "row",
                                    justify_content: "space-between",
                                    span { color: "#7dcfff", "{disk.name}" }
                                    span { color: "#a9b1d6", "{disk.used} / {disk.total}" }
                                }
                                div {
                                    color: "#9ece6a",
                                    font_family: "monospace",
                                    "{meter_blocks(disk.percent_used(), 24)}"
                                }
                            }
                            div { color: "#565f89", "io 0%" }
                        }
                    }

                    Panel { title: "Processes", accent: "#ff9e64", wide: Some(true),
                        div {
                            display: "flex",
                            flex_direction: "column",
                            gap: "0.1ch",
                            div {
                                color: "#a9b1d6",
                                font_family: "monospace",
                                "PID    USER   CPU%  MEM     TIME     COMMAND"
                            }
                            for line in proc_lines.iter() {
                                div {
                                    color: "#c0caf5",
                                    font_family: "monospace",
                                    "{line}"
                                }
                            }
                        }
                    }
                }

                Panel { title: "Network", accent: "#2ac3de", wide: Some(true),
                    div {
                        display: "flex",
                        flex_direction: "column",
                        gap: "0.2ch",
                        div { color: "#7dcfff", "{net.name}" }
                        div { color: "#a9b1d6", "Download {net.down_mb} MiB/s  Total {net.total_down_gb} GiB" }
                        div { color: "#a9b1d6", "Upload   {net.up_mb} MiB/s  Total {net.total_up_gb} GiB" }
                        div {
                            font_family: "monospace",
                            color: "#2ac3de",
                            "{sparkline(&net_down, 32)}"
                        }
                        div {
                            font_family: "monospace",
                            color: "#7dcfff",
                            "{sparkline(&net_up, 32)}"
                        }
                    }
                }
            }

            div {
                color: "#565f89",
                padding_top: "0.4ch",
                font_family: "monospace",
                "F1 Help  F2 Menu  F3 Search  F4 Filter  F5 Tree  F6 Sort  F7 Nice  F8 Kill  F9 Signals  F10 Quit"
            }
        }
    }
}

#[component]
fn Panel(title: &'static str, accent: &'static str, children: Element, wide: Option<bool>) -> Element {
    let is_wide = wide.unwrap_or(false);
    let basis = if is_wide { "100%" } else { "30ch" };

    rsx! {
        div {
            flex_basis: "{basis}",
            flex_grow: "1",
            min_width: "24ch",
            background_color: "#111521",
            border_style: "solid",
            border_width: "1px",
            border_color: "#2f354d",
            padding: "0.5ch",
            box_sizing: "border-box",

            div {
                display: "flex",
                flex_direction: "row",
                justify_content: "space-between",
                align_items: "center",
                padding_bottom: "0.2ch",
                border_bottom: "1px solid #24283b",

                span { color: "{accent}", "{title}" }
                span { color: "#565f89", "mock" }
            }
            div { padding_top: "0.2ch", {children} }
        }
    }
}
