use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
};

use crate::app::App;

pub fn draw_ui<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &App,
) -> Result<(), Box<dyn std::error::Error>> {
    terminal.draw(|f| {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Min(1),
                Constraint::Length(3),
            ])
            .split(f.area());

        // Header（可扩展：添加更多状态显示）
        let mut header_spans = vec![
            Span::styled(" Ollama TUI ", Style::default().fg(Color::Yellow)),
            Span::raw(" | "),
            Span::styled(format!("Mode: {}", app.mode.as_str()), Style::default().fg(Color::Green)),
            Span::raw(" | "),
            Span::styled(format!("Session: {}", app.session), Style::default().fg(Color::Cyan)),
        ];
        if app.waiting {
            header_spans.push(Span::raw(" [thinking...]").style(Style::default().fg(Color::Red)));
        }
        let header = Line::from(header_spans);
        f.render_widget(Paragraph::new(header), chunks[0]);

        // Body（可扩展：添加滚动支持）
        if app.show_history {
            let items: Vec<ListItem> = app.history.iter().rev().enumerate().map(|(i, m)| {
                ListItem::new(format!("{}: {} → {}", i + 1, m.input, m.output))
            }).collect();
            f.render_widget(
                List::new(items).block(Block::default().borders(Borders::ALL).title("History Stack")),
                chunks[1],
            );
        } else {
            let log = if app.history.is_empty() {
                "No messages yet.".to_string()
            } else {
                let last = app.history.last().unwrap();
                format!("> {}\n{}", last.input, last.output)
            };
            f.render_widget(
                Paragraph::new(log).block(Block::default().borders(Borders::ALL)),
                chunks[1],
            );
        }

        // Input（可扩展：添加自动补全）
        let input = Paragraph::new(app.input.as_str())
            .style(Style::default().fg(Color::Yellow))
            .block(Block::default().borders(Borders::ALL).title("Input | Ctrl+M=Mode | Ctrl+S=Session | Ctrl+H=History | Ctrl+Q=Quit"));
        f.render_widget(input, chunks[2]);
    })?;
    Ok(())
}
