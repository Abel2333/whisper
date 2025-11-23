# Repository Guidelines

This guide keeps contributions consistent across the agent-oriented CLI tools in this repository. Follow each section before submitting code.

## Project Structure & Module Organization

- `src/main.rs` wires configuration, logging, and the top-level agent runtime.
- `src/lib.rs` exports shared primitives; `src/agent/` hosts embedding, session, and adaptor logic; each module should keep a focused API surface.
- `src/mcp/` contains transport/tool adapters for Rig MCP clients.
- `src/secure/` provides AES-based helpers used by `src/bin/encryptor.rs`.
- `src/bin/cli_chatbot.rs` and `src/bin/encryptor.rs` deliver runnable binaries; keep CLI parsing isolated inside each binary.
- `config.toml` plus `.env` variables drive secrets; add new knobs in `src/config/` and document defaults inline.

## Build, Test, and Development Commands

- `cargo fmt --all` formats Rust code; run before every commit.
- `cargo clippy --all-targets --all-features -D warnings` enforces lints matching CI.
- `cargo test --all -- --nocapture` executes unit/integration suites and streams logs for debugging.
- `cargo run --bin cli_chatbot -- --config config.toml` runs the chat agent against the sample config.
- `cargo run --bin encryptor -- --input samples/plain.txt` exercises the secure pipeline; replace the input path as needed.

## Coding Style & Naming Conventions

- Follow Rust 2024 idioms: 4-space indentation, snake_case modules/files, UpperCamelCase types, SCREAMING_SNAKE_CASE constants.
- Prefer explicit `pub(crate)` visibility and constructor functions over exposing fields.
- Keep tracing spans contextual (`info_span!("session", user_id = ...)`) and prefer `anyhow::Result` for fallible entry points.

## Testing Guidelines

- Co-locate tests in `mod tests` blocks next to the code or create `tests/` folders when cross-module orchestration is needed.
- When touching crypto or transport code, add regression tests that cover both success and failure paths plus malformed input scenarios.
- Aim for a minimum of one targeted test per new public function; use `#[tokio::test]` for async flows.

## Commit & Pull Request Guidelines

- Match the existing concise, imperative commit style ("Add security to protact api keys", "Change the structure of chat"), keeping subject lines under 60 characters.
- Reference issue numbers in the body, describe observable behavior changes, and call out configuration or schema updates explicitly.
- PRs must include: summary, testing evidence (`cargo test` output or screenshots for binaries), and any follow-up tasks or TODOs.

## Security & Configuration Tips

- Never commit real credentials; load them with `dotenvy` or environment overrides.
- Keep AES keys and salts in `config.toml` only as placeholders, and validate at startup before wiring transports.
