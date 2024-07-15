use ollama_rs::{generation::completion::request::GenerationRequest, Ollama};
use tokio::io::{stdout, AsyncWriteExt};
use tokio_stream::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let prompt = r#"
 Your are Chatty, a natural language to zsh shell command translation engine for MacOS.
 You are an expert in zsh on MacOs and solve the problem at the end of the prompt with a zsh command.

 Obey the following rules to get the correct zsh command:
 * Your output is a single zsh command, you may pipe commands together.
 * The zsh command must solve the question.
 * Be consice and show just the final command in plain text.
 * Only show a single answer.
 * Do not create invalid syntax or cause syntax errors.
 * Remove any explanation or comments.
 * Do not surround the command with any additional characters like quotes or ```.
 * The answer must work on MacOS.
 * The answer must work on zsh.
 * You may use the following additional commands: jq, bat, fd, fzf, rg, xh

The problem is: \
"#.to_string();
    let ollama = Ollama::default();

    let mut stdout = stdout();

    loop {
        stdout.write_all(b"\n> ").await?;
        stdout.flush().await?;

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;

        let input = input.trim_end();
        if input.eq_ignore_ascii_case("exit") {
            break;
        }

        let prompt = prompt.to_owned() + input;

        let request = GenerationRequest::new("codestral:latest".into(), prompt);

        let mut stream = ollama.generate_stream(request).await?;

        let mut solution = "".to_string();
        while let Some(Ok(res)) = stream.next().await {
            for ele in res {
                solution += ele.response.as_str();
            }
        }
        println!("{}", solution);
    }

    Ok(())
}
