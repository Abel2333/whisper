# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Whisper is a Rust-based playground for experimenting with Rig MCP (Model Context Protocol) powered agents. It bundles a multi-turn CLI chat runtime and an AES-256-GCM encryption helper for exercising both inference and secure transport paths from a single crate.

**Current version:** 0.2.0

## Development Commands

### Building and Testing
```bash
# Format code (run before every commit)
cargo fmt --all

# Run linter with CI-level strictness
cargo clippy --all-targets --all-features -D warnings

# Run all tests with output
cargo test --all -- --nocapture

# Run a single test
cargo test test_name -- --nocapture
```

### Running Binaries
```bash
# Run the chat agent (requires config.toml with credentials)
cargo run --bin cli_chatbot -- --config config.toml

# Encrypt text using AES-256-GCM (requires ENCRYPT_KEY env var)
cargo run --bin encryptor -- encrypt --text "secret"

# Decrypt text
cargo run --bin encryptor -- decrypt --text "encrypted_value"
```

### Documentation
```bash
# Regenerate API docs (shipped in docs/ folder)
cargo doc --no-deps
rm -rf docs && mkdir -p docs
cp -r target/doc/. docs/
```

## Architecture Overview

### Module Organization

- **`src/lib.rs`** - Exports shared primitives across the workspace
- **`src/agent/`** - Core agent infrastructure:
  - `model_adaptor.rs` - Creates Rig agents from model configs; supports OpenAI, Ollama, DeepSeek providers
  - `session.rs` - Type-state builder pattern for chat/agent sessions; implements streaming response handling via `ChatSession` trait
  - `cli_chat.rs` - Terminal frontend implementing `ResponseSink` and `InputSource` traits
  - `dyn_embedding_wrapper.rs` - Wraps dynamic embedding models for Rig's vector store integration
  - `tools/` - Built-in agent tools (currently contains calculator)
- **`src/config/`** - Configuration loading from `config.toml` and environment variables
  - `read_config.rs` - Parses model configs, decrypts API keys using AES-256-GCM at startup
- **`src/secure/`** - AES-256-GCM encryption/decryption utilities
  - `aes.rs` - Core crypto implementation used by both config loader and encryptor binary
- **`src/bin/`** - Runnable binaries:
  - `cli_chatbot.rs` - Boots agent runtime from config, streams responses, logs to `logs/`
  - `encryptor.rs` - Standalone encrypt/decrypt tool using `src/secure/` modules

### Key Architectural Patterns

**Type-state Session Builder:**
The `SessionBuilder` uses compile-time type states to ensure an agent or chat implementation is provided before building a session:
```rust
SessionBuilder::new()
    .agent(agent)
    .multi_turn_depth(4)
    .show_usage()
    .build()
```

**Response Sink Abstraction:**
The `ResponseSink` trait in `session.rs` abstracts output handling (text, reasoning, errors, usage stats). `CliFrontend` implements this for terminal display. This allows swapping frontends without changing agent logic.

**Model Configuration:**
Supports 1-2 models per agent:
- Single completion model (required)
- Completion + embedding model (optional, enables dynamic tool selection via RAG)

Model configs are validated at startup in `model_adaptor.rs:validate()`.

**Encrypted Credentials:**
API keys in `config.toml` are stored encrypted. The `ENCRYPT_KEY` environment variable is loaded at startup and used to decrypt keys before passing them to provider clients.

### Agent Multi-turn Flow

1. User input is read via `InputSource::read_input()`
2. `ChatSession::request()` streams the agent's response
3. Stream items are categorized as Text, Reasoning, ToolCall, or FinalResponse
4. Each item type is routed to appropriate `ResponseSink` methods
5. Completed turns are appended to `chat_log` for multi-turn context
6. Loop continues until input source is exhausted

## Configuration

### Setup
```bash
# Copy example config and add your credentials
cp config.example.toml config.toml

# Set encryption key (hex or base64)
export ENCRYPT_KEY="your-key-here"
```

### Model Configuration Structure
Each `[[models]]` entry requires:
- `base_url` - Provider API endpoint
- `api_key` - Encrypted API key (use encryptor binary to encrypt)
- `provider` - One of: "openai", "ollama", "deepseek"
- `model_name` - Model identifier (e.g., "gpt-4o-mini")
- `model_type` - "completion" or "embedding"
- `context_size` - Max tokens (for embeddings: max batch size)

### MCP Server Configuration
`[[mcp_servers]]` entries support:
- `protocol` - "sse" or "stdio"
- `name` - Server identifier
- `url` - For SSE servers
- `command`, `args`, `envs` - For stdio servers

## Coding Conventions

### Style (from AGENTS.md)
- Rust 2024 edition idioms
- 4-space indentation, snake_case modules/files, UpperCamelCase types, SCREAMING_SNAKE_CASE constants
- Prefer explicit `pub(crate)` visibility and constructor functions over exposed fields
- Use contextual tracing spans: `info_span!("session", user_id = ...)`
- Prefer `anyhow::Result` for fallible entry points

### Testing
- Co-locate tests in `mod tests` blocks next to the code
- For cross-module orchestration, create `tests/` folder entries
- Add regression tests for crypto/transport code covering success, failure, and malformed input
- Aim for minimum one test per new public function
- Use `#[tokio::test]` for async flows

### Commit Style
- Concise, imperative style: "Add security to protect api keys", "Change the structure of chat"
- Keep subject lines under 60 characters
- Reference issue numbers in body
- Describe observable behavior changes
- Call out configuration or schema updates explicitly

## Security Notes

- Never commit real credentials; use `dotenvy` or environment overrides
- Keep AES keys and salts in `config.toml` only as placeholders
- Validate encryption keys at startup before wiring transports
- API keys are decrypted in `read_config.rs` after loading from config; models with failed decryption are skipped with log warnings

## Logging

- Logs are written to `logs/cli_chatbot_*.log` with daily rotation
- Keep up to 5 log files, max 10MB each
- Use `flexi_logger` with `opt_format`
- Log level controlled by `RUST_LOG` env var (defaults to "info")
