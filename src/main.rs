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
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};
use std::io;
use tokio::io::{stdout, AsyncWriteExt};
use tokio_stream::StreamExt;

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
    loop {
        terminal.draw(|frame| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([Constraint::Min(3)])
                .split(frame.area());

            let block = Block::default()
                .title(" Chat - Press 'q' or Ctrl+C to quit ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan));

            let paragraph = Paragraph::new("Welcome to Chat TUI!")
                .block(block)
                .style(Style::default().fg(Color::White));

            frame.render_widget(paragraph, chunks[0]);
        })?;

        // Handle input events
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Char('c')
                            if key.modifiers.contains(event::KeyModifiers::CONTROL) =>
                        {
                            break;
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    Ok(())
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
