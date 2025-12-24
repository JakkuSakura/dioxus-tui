use dioxus_tui::builders::RectBuilder;

use crate::components::ComponentBlock;
use crate::data::TopbarData;
use crate::render::{bar_repeat, pad_right};
use crate::theme;

const BASE_WIDTH: usize = 120;
const HEIGHT: usize = 1;

fn tab_style(tab: &str, active: &str) -> dioxus_tui::builders::Style {
    if tab == active {
        let mut style = theme::fg_bg(theme::SELECTED_FG, theme::SELECTED_BG);
        style.bold = true;
        style
    } else {
        theme::fg(theme::INACTIVE_FG)
    }
}

pub fn render_with_width(data: &TopbarData, width: usize) -> RectBuilder {
    let width = width.max(1);
    let mut rect = RectBuilder::new(width, HEIGHT);
    let line = rect.line_mut(0).expect("topbar line");
    let border = theme::fg(theme::DIV_LINE);

    let mut x = 0;
    line.set_str_styled(x, "╭─┐", border.clone());
    x += 3;

    let tab = format!("¹{}", data.tabs[0]);
    line.set_str_styled(x, &tab, tab_style(data.tabs[0], data.active_tab));
    x += tab.chars().count();

    line.set_str_styled(x, "┌──┐", border.clone());
    x += 4;

    let tab = data.tabs[1];
    line.set_str_styled(x, tab, tab_style(tab, data.active_tab));
    x += tab.chars().count();

    line.set_str_styled(x, "┌┐", border.clone());
    x += 2;

    let tab = data.tabs[2];
    line.set_str_styled(x, tab, tab_style(tab, data.active_tab));
    x += tab.chars().count();

    line.set_str_styled(x, " *┌", border.clone());
    x += 3;

    let time = pad_right(data.time, 8);
    let interval = format!("{}ms", data.interval_ms);
    let base_len = x + 1 + 30 + time.chars().count() + 1 + 40 + 3 + interval.chars().count() + 5;
    let extra = width.saturating_sub(base_len);
    let bar1_len = 30;
    let bar2_len = 40 + extra;

    let bar = bar_repeat('─', bar1_len);
    line.set_str_styled(x, &bar, border.clone());
    x += bar.chars().count();

    line.set_str_styled(x, "┐", border.clone());
    x += 1;
    line.set_str_styled(x, &time, theme::fg_bold(theme::TITLE));
    x += time.chars().count();

    line.set_str_styled(x, "┌", border.clone());
    x += 1;

    let bar = bar_repeat('─', bar2_len);
    line.set_str_styled(x, &bar, border.clone());
    x += bar.chars().count();

    line.set_str_styled(x, "┐- ", border.clone());
    x += 3;
    line.set_str_styled(x, &interval, theme::fg(theme::TITLE));
    x += interval.chars().count();

    line.set_str_styled(x, " +┌─╮", border);

    rect
}

pub fn render(data: &TopbarData) -> ComponentBlock {
    ComponentBlock {
        x: 0,
        y: 0,
        rect: render_with_width(data, BASE_WIDTH),
    }
}
