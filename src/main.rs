#![warn(clippy::all, clippy::pedantic)]

use clap::{Parser, Subcommand};
use color_eyre::Result;
use ollama_rs::{generation::completion::request::GenerationRequest, Ollama};
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
        self.cmd.clone().unwrap_or(Cmd::Run)
    }
}

#[derive(Subcommand, Debug, Clone)]
enum Cmd {
    /// Run the application interactively (default)
    Run,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Cli::parse();
    match args.cmd() {
        Cmd::Run => run().await,
    }
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
