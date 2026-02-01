# AGENTS.md

This file provides guidance to AI coding agents when working with code in this repository.

## Project Overview

Chat is a natural language to zsh shell command translator for macOS. It uses Ollama with the ai models to convert English descriptions into shell commands.

## Build and Run

```zsh
# Build
cargo build --release

# Run
cargo run

# Lint (strict pedantic mode enabled via #![warn(clippy::all, clippy::pedantic)])
cargo clippy
```

**Prerequisites:** Ollama must be running (`ollama serve`) with the necessary mode (use `ollama pull <image_name>` to pull).

## Architecture

Single-file Rust application (`src/main.rs`) using:

- **clap** for CLI argument parsing with subcommand support
- **ollama-rs** for LLM interaction with streaming responses
- **tokio** async runtime for I/O operations
- **color-eyre** for error handling

The main loop reads user input, prepends a system prompt that instructs the LLM to output only zsh commands, streams the response from Ollama, and prints the result. Type "exit" to quit.

The system prompt encourages use of modern CLI tools: jq, bat, eza, fd, fzf, rg, xh.
