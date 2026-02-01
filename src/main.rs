#![warn(clippy::all, clippy::pedantic)]

use clap::{Parser, Subcommand};
use color_eyre::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ollama_rs::{generation::completion::request::GenerationRequest, Ollama};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};
use std::io;
use tokio::io::{stdout, AsyncWriteExt};
use tokio_stream::StreamExt;

/// Application state for the TUI
#[derive(Default)]
struct App {
    /// Current input text
    input: String,
    /// Current cursor position in the input
    cursor_position: usize,
    /// Whether a query has been submitted
    query_submitted: bool,
}

impl App {
    /// Insert a character at the cursor position
    fn insert_char(&mut self, c: char) {
        self.input.insert(self.cursor_position, c);
        self.cursor_position += 1;
    }

    /// Delete the character before the cursor (backspace)
    fn delete_char(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
            self.input.remove(self.cursor_position);
        }
    }

    /// Delete the character at the cursor (delete key)
    fn delete_char_forward(&mut self) {
        if self.cursor_position < self.input.len() {
            self.input.remove(self.cursor_position);
        }
    }

    /// Move cursor left
    fn move_cursor_left(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
        }
    }

    /// Move cursor right
    fn move_cursor_right(&mut self) {
        if self.cursor_position < self.input.len() {
            self.cursor_position += 1;
        }
    }

    /// Move cursor to start of input
    fn move_cursor_home(&mut self) {
        self.cursor_position = 0;
    }

    /// Move cursor to end of input
    fn move_cursor_end(&mut self) {
        self.cursor_position = self.input.len();
    }

    /// Submit the current query
    fn submit_query(&mut self) {
        if !self.input.trim().is_empty() {
            self.query_submitted = true;
        }
    }
}

#[derive(Parser)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    cmd: Option<Cmd>,
}
impl Cli {
    fn cmd(&self) -> Cmd {
        self.cmd.clone().unwrap_or(Cmd::Tui)
    }
}

#[derive(Subcommand, Debug, Clone)]
enum Cmd {
    /// Run the application interactively (default)
    Run,
    /// Run the TUI interface
    Tui,
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let args = Cli::parse();
    match args.cmd() {
        Cmd::Run => run().await,
        Cmd::Tui => run_tui(),
    }
}

/// Initialize the terminal for TUI mode
fn setup_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

/// Restore the terminal to its original state
fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}

/// Run the TUI application
fn run_tui() -> Result<()> {
    let mut terminal = setup_terminal()?;

    let result = run_tui_loop(&mut terminal);

    // Always restore terminal, even if there was an error
    restore_terminal(&mut terminal)?;

    result
}

/// Main TUI event loop
fn run_tui_loop(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    let mut app = App::default();

    loop {
        terminal.draw(|frame| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([Constraint::Length(3), Constraint::Min(1)])
                .split(frame.area());

            // Render input box
            render_input(&app, frame, chunks[0]);

            // Render status/help area
            let status_text = if app.query_submitted {
                format!("Query submitted: {}", app.input)
            } else {
                "Type your problem and press Enter to submit. Press Esc or Ctrl+C to quit.".to_string()
            };

            let status_block = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray));

            let status = Paragraph::new(status_text)
                .block(status_block)
                .style(Style::default().fg(Color::Gray));

            frame.render_widget(status, chunks[1]);
        })?;

        // Handle input events
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Esc => break,
                        KeyCode::Char('c')
                            if key.modifiers.contains(event::KeyModifiers::CONTROL) =>
                        {
                            break;
                        }
                        KeyCode::Enter => app.submit_query(),
                        KeyCode::Backspace => app.delete_char(),
                        KeyCode::Delete => app.delete_char_forward(),
                        KeyCode::Left => app.move_cursor_left(),
                        KeyCode::Right => app.move_cursor_right(),
                        KeyCode::Home => app.move_cursor_home(),
                        KeyCode::End => app.move_cursor_end(),
                        KeyCode::Char(c) => app.insert_char(c),
                        _ => {}
                    }
                }
            }
        }
    }

    Ok(())
}

/// Render the text input widget
fn render_input(app: &App, frame: &mut ratatui::Frame, area: ratatui::layout::Rect) {
    let input_block = Block::default()
        .title(" Describe your problem: ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    // Build the input text with cursor
    let input_text = if app.cursor_position < app.input.len() {
        // Cursor in middle of text
        let before = &app.input[..app.cursor_position];
        let cursor_char = &app.input[app.cursor_position..=app.cursor_position];
        let after = &app.input[app.cursor_position + 1..];

        Line::from(vec![
            Span::raw(before),
            Span::styled(
                cursor_char,
                Style::default()
                    .bg(Color::White)
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(after),
        ])
    } else {
        // Cursor at end of text (show block cursor)
        Line::from(vec![
            Span::raw(&app.input),
            Span::styled(
                " ",
                Style::default()
                    .bg(Color::White)
                    .fg(Color::Black),
            ),
        ])
    };

    let input = Paragraph::new(input_text)
        .block(input_block)
        .style(Style::default().fg(Color::White));

    frame.render_widget(input, area);
}

async fn run() -> Result<()> {
    let prompt = r"
 Your are Chatty, a natural language to zsh shell command translation engine for MacOS.
 You are an expert in zsh on MacOs and solve the problem at the end of the prompt with a zsh command.

 Obey the following rules to generate the correct zsh command:
 * Your output is a single zsh command, you may pipe commands together.
 * The zsh command must solve the question.
 * Be consice and show just the final command in plain text.
 * Only show a single answer.
 * Do not create invalid syntax or cause syntax errors.
 * Remove any explanation or comments.
 * Do not surround the command with any additional characters like quotes or ```.
 * The answer must work on MacOS.
 * The answer must work on zsh.
 * Try to use the following additional commands: jq, bat, eza, fd, fzf, rg, xh

The problem is: \
".to_string();

    let model_name = "qwen3-coder:latest";

    let ollama = Ollama::default();

    let mut stdout = stdout();

    let _ = ollama
        .show_model_info(model_name.to_string())
        .await
        .expect("Failed to get model info, is Ollama server running?");

    stdout
        .write_all(format!("Running model {model_name}\n").as_bytes())
        .await?;
    stdout.flush().await?;

    loop {
        stdout.write_all(b"\n> ").await?;
        stdout.flush().await?;

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;

        let input = input.trim_end();
        if input.eq_ignore_ascii_case("exit") {
            break;
        }

        let prompt = prompt.clone() + input;

        let request = GenerationRequest::new(model_name.into(), prompt);

        let mut stream = ollama.generate_stream(request).await?;

        let mut solution = String::new();
        while let Some(Ok(res)) = stream.next().await {
            for ele in res {
                solution += ele.response.as_str();
            }
        }
        println!("{solution}");
    }

    Ok(())
}
