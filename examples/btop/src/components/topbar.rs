use crate::components::ComponentBlock;
use crate::data::TopbarData;
use crate::render::{bar_repeat, pad_right};

pub fn render(data: &TopbarData) -> ComponentBlock {
    let line = format!(
        "╭─┐¹{}┌──┐{}┌┐{} *┌{}┐{}┌{}┐- {}ms +┌─╮",
        data.tabs[0],
        data.tabs[1],
        data.tabs[2],
        bar_repeat('─', 30),
        data.time,
        bar_repeat('─', 40),
        data.interval_ms
    );
    let line = pad_right(&line, 120);

    ComponentBlock {
        x: 0,
        y: 0,
        lines: vec![line],
    }
}
