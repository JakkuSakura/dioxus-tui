use anyhow::Result;
use dioxus_tui::capabilities::detect;
use std::io::{self, Write};
use termwiz::input::{InputEvent, KeyCode, Modifiers};
use termwiz::surface::Change;
use termwiz::terminal::{new_terminal, Terminal as _};

struct TerminalGuard<T: termwiz::terminal::Terminal> {
    term: T,
    pixel_mouse_enabled: bool,
}

impl<T: termwiz::terminal::Terminal> TerminalGuard<T> {
    fn new(mut term: T) -> Result<Self> {
        term.set_raw_mode()?;
        term.render(&[Change::Text("\x1b[?1016h".to_string())])?;
        term.flush()?;
        Ok(Self {
            term,
            pixel_mouse_enabled: true,
        })
    }

    fn term_mut(&mut self) -> &mut T {
        &mut self.term
    }
}

impl<T: termwiz::terminal::Terminal> Drop for TerminalGuard<T> {
    fn drop(&mut self) {
        if self.pixel_mouse_enabled {
            let _ = self
                .term
                .render(&[Change::Text("\x1b[?1016l".to_string())]);
            let _ = self.term.flush();
        }
        let _ = self.term.set_cooked_mode();
    }
}

fn describe_mouse(event: &InputEvent) -> Option<String> {
    match event {
        InputEvent::Mouse(mouse) => Some(format!(
            "mouse: ({}, {}) buttons={:?} modifiers={:?}",
            mouse.x, mouse.y, mouse.mouse_buttons, mouse.modifiers
        )),
        InputEvent::PixelMouse(mouse) => Some(format!(
            "pixel mouse: ({}, {}) buttons={:?} modifiers={:?}",
            mouse.x_pixels, mouse.y_pixels, mouse.mouse_buttons, mouse.modifiers
        )),
        InputEvent::Resized { cols, rows } => Some(format!("resized: {cols}x{rows}")),
        _ => None,
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
        "Raw mouse demo (no Dioxus). Move/click/scroll; 'q' or Ctrl-C exits."
    )?;
    writeln!(
        stdout,
        "Note: mouse reporting depends on terminal capabilities."
    )?;
    stdout.flush()?;

    loop {
        if let Some(event) = session.term_mut().poll_input(None)? {
            if should_exit(&event) {
                break;
            }
            if let Some(line) = describe_mouse(&event) {
                writeln!(stdout, "{line}")?;
                stdout.flush()?;
            }
        }
    }

    Ok(())
}
