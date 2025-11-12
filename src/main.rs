// 需要：1.webhook接口 2.llama.cpp和hf调用 3.线程模式和切换探讨

use std::io::{self, BufRead, Write};
use std::process::Command;
use std::time::Duration;
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{backend::CrosstermBackend, Terminal};

mod app;
mod ui;
mod ollama;
mod agent;
mod shortcuts;
mod utils;

use app::{App, Mode, Task};
use shortcuts::handle_shortcut;
use ui::draw_ui;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 禁用终端 Ctrl+Q 快捷键（可扩展：根据终端类型动态配置）
    let _ = Command::new("stty").args(&["quit", "undef"]).output();

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    stdout.execute(EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();

    let result = (|| {
        loop {
            // 接收后台响应（多线程模式：当前 mpsc，可换 tokio::select!）
            if let Ok(resp) = app.rx.try_recv() {
                if let Some(last) = app.history.last_mut() {
                    last.output = resp.clone();
                }
                app.save_to_json(&resp);
                app.waiting = false;
            }

            draw_ui(&mut terminal, &app)?;

            if let Event::Key(key) = event::read()? {
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    if handle_shortcut(&mut app, key.code, &mut terminal)? {
                        continue;
                    }
                }

                match key.code {
                    KeyCode::Enter => {
                        if app.input.starts_with("quote:") {
                            if let Ok(idx) = app.input[6..].trim().parse::<usize>() {
                                if let Some(quote) = app.quote(idx) {
                                    app.input = format!("(quoted) {}", quote);
                                }
                            }
                        } else if !app.waiting {
                            app.send();
                        }
                    }
                    KeyCode::Char(c) => app.input.push(c),
                    KeyCode::Backspace => { app.input.pop(); }
                    KeyCode::Esc => app.input.clear(),
                    _ => {}
                }
            }
        }
        Ok::<(), Box<dyn std::error::Error>>(())
    })();

    // 恢复终端
    disable_raw_mode()?;
    io::stdout().execute(LeaveAlternateScreen)?;
    let _ = Command::new("stty").args(&["quit", "^Q"]).output();  // 恢复 Ctrl+Q

    result?;
    Ok(())
}
