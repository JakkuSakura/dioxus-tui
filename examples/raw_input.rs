use anyhow::Result;
use dioxus_tui::capabilities::detect;
use std::io::{self, Write};
use termwiz::input::{InputEvent, KeyCode, Modifiers};
use termwiz::terminal::{new_terminal, Terminal as _};

struct TerminalGuard<T: termwiz::terminal::Terminal> {
    term: T,
}

impl<T: termwiz::terminal::Terminal> TerminalGuard<T> {
    fn new(mut term: T) -> Result<Self> {
        term.set_raw_mode()?;
        Ok(Self { term })
    }

    fn term_mut(&mut self) -> &mut T {
        &mut self.term
    }
}

impl<T: termwiz::terminal::Terminal> Drop for TerminalGuard<T> {
    fn drop(&mut self) {
        let _ = self.term.set_cooked_mode();
    }
}

fn describe_event(event: &InputEvent) -> String {
    match event {
        InputEvent::Key(key) => format!("key: {:?} modifiers={:?}", key.key, key.modifiers),
        InputEvent::Mouse(mouse) => format!(
            "mouse: ({}, {}) buttons={:?} modifiers={:?}",
            mouse.x, mouse.y, mouse.mouse_buttons, mouse.modifiers
        ),
        InputEvent::PixelMouse(mouse) => format!(
            "pixel mouse: ({}, {}) buttons={:?} modifiers={:?}",
            mouse.x_pixels, mouse.y_pixels, mouse.mouse_buttons, mouse.modifiers
        ),
        InputEvent::Resized { cols, rows } => format!("resized: {cols}x{rows}"),
        InputEvent::Paste(text) => format!("paste: {text:?}"),
        InputEvent::Wake => "wake".to_string(),
    }
}

fn should_exit(event: &InputEvent) -> bool {
    match event {
        InputEvent::Key(key) => match key.key {
            KeyCode::Char('q' | 'Q') if key.modifiers == Modifiers::NONE => true,
            KeyCode::Char('c' | 'C') if key.modifiers.contains(Modifiers::CTRL) => true,
            _ => false,
        },
        _ => false,
    }
}

fn main() -> Result<()> {
    let caps = detect()?.termwiz;
    let term = new_terminal(caps)?;
    let mut session = TerminalGuard::new(term)?;

    let mut stdout = io::stdout();
    writeln!(
        stdout,
        "Raw input demo (no Dioxus). Press keys or use the mouse; 'q' or Ctrl-C exits."
    )?;
    writeln!(
        stdout,
        "Note: mouse events require terminal mouse reporting support."
    )?;
    stdout.flush()?;

    loop {
        if let Some(event) = session.term_mut().poll_input(None)? {
            if should_exit(&event) {
                break;
            }
            writeln!(stdout, "{}", describe_event(&event))?;
            stdout.flush()?;
        }
    }

    Ok(())
}
