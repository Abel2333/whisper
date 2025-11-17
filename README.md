# Whisper

Rust-based playground for experimenting with Rig MCP powered agents. The
workspace bundles a multi-turn CLI chat runtime and an AES-256-GCM helper so
you can exercise both the inference and secure transport paths from a single
crate.

## Components

- **cli_chatbot** – boots the agent runtime from `config.toml`, streams
  responses in the terminal, and records structured logs in `logs/`.
- **encryptor** – provides `encrypt` / `decrypt` subcommands that use the
  utilities in `src/secure/` and an `ENCRYPT_KEY` loaded from the environment.

## Getting Started

1. Install the latest stable Rust toolchain (`rustup default stable`).
2. Copy the sample config and replace placeholders with your credentials:

   ```bash
   cp config.example.toml config.toml
   export ENCRYPT_KEY="hex-or-base64-key"
   ```

3. Run the chat agent:

   ```bash
   cargo run --bin cli_chatbot -- --config config.toml
   ```

4. Exercise the encryptor:

   ```bash
   cargo run --bin encryptor -- encrypt --text "secret"
   ```

## Development Workflow

- `cargo fmt --all`
- `cargo clippy --all-targets --all-features -D warnings`
- `cargo test --all -- --nocapture`

These match the repository guidelines in `AGENTS.md` and should be run before
committing changes.

## Documentation

- Pre-generated API docs live in `docs/` (open `docs/whisper/index.html`). This
  folder can be published directly with GitHub Pages if desired.
- Regenerate docs after code changes:

  ```bash
  cargo doc --no-deps
  rm -rf docs && mkdir -p docs
  cp -r target/doc/. docs/
  ```

## Versioning

- Current crate version: **0.2.0**. This release refreshes the rustdoc output
  and ships it with the repository so downstream tools can consume the agent
  API without building docs locally.
