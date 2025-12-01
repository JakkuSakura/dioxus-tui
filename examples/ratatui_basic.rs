use crossterm::{execute, terminal::{EnterAlternateScreen, LeaveAlternateScreen}};
use ratatui::{backend::CrosstermBackend, layout::{Layout, Constraint, Direction, Alignment}, widgets::{Block, Borders, Paragraph}, style::{Style, Color}, Terminal};
use std::io;

fn main() -> anyhow::Result<()> {
    // Enter alternate screen for a clean demo
    execute!(io::stdout(), EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;

    terminal.draw(|frame| {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Min(1),
            ])
            .split(frame.size());

        let header = Paragraph::new("Ratatui demo")
            .style(Style::default().fg(Color::Yellow))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).title("Title"));

        let info = Paragraph::new("This is a simple ratatui layout without Dioxus.")
            .block(Block::default().borders(Borders::ALL).title("Info"));

        let body = Paragraph::new("Press Ctrl+C to exit.")
            .block(Block::default().borders(Borders::ALL).title("Body"));

        frame.render_widget(header, chunks[0]);
        frame.render_widget(info, chunks[1]);
        frame.render_widget(body, chunks[2]);
    })?;

    // Wait so output remains visible in many terminals
    std::thread::sleep(std::time::Duration::from_secs(3));

    execute!(io::stdout(), LeaveAlternateScreen)?;
    Ok(())
}
