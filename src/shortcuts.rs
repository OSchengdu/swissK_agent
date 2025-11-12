use crossterm::event::KeyCode;
use ratatui::Terminal;

use crate::app::{App, Mode};

pub fn handle_shortcut<B: ratatui::backend::Backend>(
    app: &mut App,
    code: KeyCode,
    terminal: &mut Terminal<B>,
) -> Result<bool, Box<dyn std::error::Error>> {
    match code {
        KeyCode::Char('m') => {
            app.mode.next();  // 切换模式
            Ok(true)
        }
        KeyCode::Char('s') => {
            // 切换会话（可扩展：添加列表选择）
            let mut new_session = String::new();
            terminal.draw(|f| {
                f.render_widget(
                    ratatui::widgets::Paragraph::new("Enter new session name:").block(ratatui::widgets::Block::default().borders(ratatui::widgets::Borders::ALL)),
                    f.area(),
                );
            })?;
            loop {
                if let crossterm::event::Event::Key(k) = crossterm::event::read()? {
                    match k.code {
                        KeyCode::Enter => break,
                        KeyCode::Char(c) => new_session.push(c),
                        KeyCode::Backspace => { new_session.pop(); }
                        _ => {}
                    }
                    terminal.draw(|f| {
                        f.render_widget(
                            ratatui::widgets::Paragraph::new(format!("Session: {}", new_session)).block(ratatui::widgets::Block::default().borders(ratatui::widgets::Borders::ALL)),
                            f.area(),
                        );
                    })?;
                }
            }
            app.session = if new_session.trim().is_empty() { "default".to_string() } else { new_session.trim().to_string() };
            Ok(true)
        }
        KeyCode::Char('h') => {
            app.show_history = !app.show_history;  // 查看历史
            Ok(true)
        }
        KeyCode::Char('q') => {
            // 优雅退出（已修复）
            Err("Quit".into())  // 跳出循环
        }
        // 扩展新快捷键：e.g., KeyCode::Char('i') => { /* Insert mode */ }
        _ => Ok(false),
    }
}
