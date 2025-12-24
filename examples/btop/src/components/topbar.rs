use dioxus_tui::builders::RectBuilder;

use crate::components::ComponentBlock;
use crate::data::TopbarData;
use crate::render::{bar_repeat, pad_right};
use crate::theme;

fn tab_style(tab: &str, active: &str) -> dioxus_tui::builders::Style {
    if tab == active {
        let mut style = theme::fg_bg(theme::SELECTED_FG, theme::SELECTED_BG);
        style.bold = true;
        style
    } else {
        theme::fg(theme::INACTIVE_FG)
    }
}

pub fn render(data: &TopbarData) -> ComponentBlock {
    let mut rect = RectBuilder::new(120, 1);
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

    let bar = bar_repeat('─', 30);
    line.set_str_styled(x, &bar, border.clone());
    x += bar.chars().count();

    line.set_str_styled(x, "┐", border.clone());
    x += 1;

    let time = pad_right(data.time, 8);
    line.set_str_styled(x, &time, theme::fg_bold(theme::TITLE));
    x += time.chars().count();

    line.set_str_styled(x, "┌", border.clone());
    x += 1;

    let bar = bar_repeat('─', 40);
    line.set_str_styled(x, &bar, border.clone());
    x += bar.chars().count();

    line.set_str_styled(x, "┐- ", border.clone());
    x += 3;

    let interval = format!("{}ms", data.interval_ms);
    line.set_str_styled(x, &interval, theme::fg(theme::TITLE));
    x += interval.chars().count();

    line.set_str_styled(x, " +┌─╮", border);

    ComponentBlock { x: 0, y: 0, rect }
}
