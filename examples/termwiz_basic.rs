use std::time::Duration;

use termwiz::{
    caps::Capabilities,
    color::ColorAttribute,
    surface::{Change, Position},
    terminal::{buffered::BufferedTerminal, SystemTerminal},
};

fn main() -> anyhow::Result<()> {
    let mut term = BufferedTerminal::new(SystemTerminal::new(Capabilities::new_from_env()?)?)?;
    term.terminal().set_raw_mode()?;
    term.terminal().enter_alternate_screen()?;

    term.add_change(Change::ClearScreen(ColorAttribute::Default));
    term.add_change(Change::CursorPosition {
        x: Position::Absolute(0),
        y: Position::Absolute(0),
    });
    term.add_change(Change::Text("Termwiz demo".to_string()));
    term.add_change(Change::CursorPosition {
        x: Position::Absolute(0),
        y: Position::Absolute(2),
    });
    term.add_change(Change::Text(
        "This is a simple termwiz layout without Dioxus.".to_string(),
    ));
    term.add_change(Change::CursorPosition {
        x: Position::Absolute(0),
        y: Position::Absolute(4),
    });
    term.add_change(Change::Text("Press Ctrl+C to exit.".to_string()));

    term.flush()?;
    std::thread::sleep(Duration::from_secs(3));

    term.terminal().leave_alternate_screen()?;
    Ok(())
}
