# Chat

A simple natural language to shell command translator using Ollama with an LLM.

Currently it supports only zsh and MacOS.

## Demo

![Demo of chat in action](./assets/demo.gif)

## Pre-requisites

[Ollama must be installed](https://ollama.com/) and up and running.
Run Ollama in the background:

```zsh
ollama serve &

# next command is only necessary once
ollama pull <llm_model_name> 
```

## Build and Run

Build the project by running:

```zsh
cargo build --release
```

Run the chat from cargo:

```zsh
cargo run
```

Install the chatbot by placing the binary from `target/release/chat` in your path.

## Configuration

The model can be configured via (in priority order):
1. `CHAT_MODEL` environment variable
2. Config file (`~/Library/Application Support/chat/config.toml` on macOS, `~/.config/chat/config.toml` on Linux):
   ```toml
   model = "qwen3-coder:latest"
   ```
3. Default: `qwen3-coder:latest`

## Usage

Run the TUI:

```zsh
chat
```

**Keyboard shortcuts:**
- `Enter` - Submit query / Select command
- `↑/↓` - Navigate commands or history
- `r` - Regenerate options
- `x` - Delete history entry
- `Esc` - Cancel / Exit

Query history is saved automatically and shown as a filterable dropdown while typing.

## Future Work

- Add a non-interactive mode and execute the commands directly
- Make it configurable to support different additional tools installed on the machine
- Add support for more shells
- Add support for other OSs

## Know Issues

These are known issue and will be fixed in the future:

- Sometimes the model will not return only the command but also some explanation.
- Very complex workflows might fail to generate the correct command.
