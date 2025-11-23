# Whisper Architecture

This document provides a high-level overview of the Whisper project architecture and how its components interact.

## Overview

Whisper is a Rust-based playground for experimenting with **Rig MCP (Model Context Protocol)** powered agents. It demonstrates:

- Multi-turn conversational AI with tool support
- Secure credential management via AES-256-GCM encryption
- Pluggable model providers (OpenAI, Ollama, DeepSeek)
- Extensible tool system
- RAG (Retrieval-Augmented Generation) for dynamic tool selection

## System Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        cli_chatbot Binary                       │
│  ┌──────────────┐    ┌──────────────┐    ┌─────────────────┐  │
│  │ Config Load  │───▶│ Agent Create │───▶│ Session.run()   │  │
│  │ + Decrypt    │    │ (model_adaptor)│   │ (session.rs)    │  │
│  └──────────────┘    └──────────────┘    └─────────────────┘  │
│         │                    │                     │           │
│         │                    │                     ▼           │
│         │                    │            ┌─────────────────┐  │
│         │                    │            │  CliFrontend    │  │
│         │                    │            │  (cli_chat.rs)  │  │
│         │                    │            └─────────────────┘  │
└─────────┼────────────────────┼──────────────────────────────────┘
          │                    │
          ▼                    ▼
┌──────────────────┐  ┌─────────────────────────────────────────┐
│  Config Module   │  │         Agent Module                    │
│                  │  │  ┌──────────────────────────────────┐   │
│  ┌────────────┐  │  │  │     Rig Framework Agent          │   │
│  │config.toml │  │  │  │  ┌────────────┐  ┌────────────┐  │   │
│  │ (encrypted)│  │  │  │  │ Completion │  │ Embedding  │  │   │
│  └────────────┘  │  │  │  │   Model    │  │   Model    │  │   │
│        │         │  │  │  └────────────┘  └────────────┘  │   │
│        ▼         │  │  │         │               │        │   │
│  ┌────────────┐  │  │  │         ▼               ▼        │   │
│  │ AES Decrypt│  │  │  │  ┌─────────────────────────┐    │   │
│  │ (secure/)  │  │  │  │  │    Tool System          │    │   │
│  └────────────┘  │  │  │  │  - Calculator           │    │   │
│        │         │  │  │  │  - MCP Tools (future)   │    │   │
│        ▼         │  │  │  │  - RAG Tool Selection   │    │   │
│  ┌────────────┐  │  │  │  └─────────────────────────┘    │   │
│  │ ModelConfig│  │  │  └──────────────────────────────────┘   │
│  └────────────┘  │  └─────────────────────────────────────────┘
└──────────────────┘
```

## Module Dependency Graph

```
┌─────────────┐
│   main.rs   │ (placeholder, not used)
└─────────────┘

┌──────────────────────────────────────────────────────────────┐
│                    Binary: cli_chatbot                       │
└──────────────────────────────────────────────────────────────┘
         │
         ├─────▶ config::read_config ───▶ secure::aes
         │                                      │
         └─────▶ agent::model_adaptor          │
                        │                       │
                        ├──▶ agent::session     │
                        ├──▶ agent::cli_chat    │
                        ├──▶ agent::tools       │
                        └──▶ agent::dyn_embedding_wrapper

┌──────────────────────────────────────────────────────────────┐
│                    Binary: encryptor                         │
└──────────────────────────────────────────────────────────────┘
         │
         └─────▶ secure::aes
```

**Key dependencies:**
- `config` → `secure` (for decryption)
- `agent` → *standalone* (no dependency on config or secure)
- Both binaries → respective modules

## Data Flow

### 1. Configuration Loading Flow

```
.env file + config.toml
         │
         ▼
   ┌───────────────┐
   │ dotenvy::load │  Load ENCRYPT_KEY
   └───────────────┘
         │
         ▼
   ┌─────────────────────────┐
   │ config::Config::build() │  Parse TOML + env vars
   └─────────────────────────┘
         │
         ▼
   ┌────────────────────────┐
   │ load_key_from_env()    │  Get encryption key
   └────────────────────────┘
         │
         ▼
   ┌────────────────────────┐
   │ for each model config: │
   │   secure::aes::decrypt │  Decrypt API keys
   └────────────────────────┘
         │
         ▼
   ┌────────────────┐
   │ AppConfig with │
   │ decrypted keys │
   └────────────────┘
```

### 2. Agent Creation Flow

```
AppConfig.models
         │
         ▼
   ┌──────────────────────────┐
   │ AgentSettings::try_new() │  Validate model configs
   └──────────────────────────┘
         │
         ▼
   ┌──────────────────────────┐
   │ create_agent()           │
   └──────────────────────────┘
         │
         ├──▶ create_completion_model()  (provider-specific client)
         ├──▶ create_embed_model()       (if embedding model present)
         │
         ▼
   ┌──────────────────────────┐
   │ AgentBuilder::new()      │  Rig framework
   │   .preamble()            │
   │   .temperature()         │
   │   .tool(Calculator)      │  Add built-in tools
   │   .dynamic_tools()       │  RAG-based tool selection (if embedding)
   │   .build()               │
   └──────────────────────────┘
         │
         ▼
   Agent<CompletionModelHandle>
```

### 3. Chat Session Flow

```
User Input (stdin)
         │
         ▼
   ┌─────────────────────────┐
   │ CliFrontend::read_input │
   └─────────────────────────┘
         │
         ▼
   ┌──────────────────────────────┐
   │ Session::run()               │
   │   loop {                     │
   │     ChatSession::request()   │◀─── Streaming response
   │       │                      │
   │       ▼                      │
   │     Agent.stream_prompt()    │
   │       .with_history()        │
   │       .multi_turn()          │
   │   }                          │
   └──────────────────────────────┘
         │
         ▼
   ┌─────────────────────────────────┐
   │ Response Stream Processing:     │
   │                                 │
   │  StreamItem match {             │
   │    Text        → output_text()  │
   │    Reasoning   → output_reason  │
   │    ToolCall    → output_text()  │
   │    FinalResp   → usage stats    │
   │  }                              │
   └─────────────────────────────────┘
         │
         ▼
   ┌────────────────────────┐
   │ CliFrontend displays   │
   │ to terminal (colored)  │
   └────────────────────────┘
         │
         ▼
   chat_log ← append (user + assistant messages)
         │
         └──▶ Next turn with history
```

## Key Architectural Patterns

### 1. Type-State Builder Pattern

**Location:** `agent/session.rs`

The `SessionBuilder` uses Rust's type system to enforce proper configuration at compile time:

```rust
SessionBuilder<NoImplProvided>        // Initial state
  → SessionBuilder<AgentImpl<M>>      // After .agent()
  → Session<AgentImpl<M>>             // After .build()
```

**Benefits:**
- Compile-time enforcement of required configuration
- IDE autocomplete guides correct usage
- No runtime validation needed

### 2. Trait-based Abstraction

**ResponseSink & InputSource Traits** (`session.rs`)

Decouples agent logic from I/O:
- Agent doesn't know about terminal/GUI/web
- Easy to swap frontends (CLI, TUI, web server)
- Testable without real I/O

**ChatSession Trait** (`session.rs`)

Unifies chat and agent interfaces:
- `ChatImpl<T>` - Simple chat without tools
- `AgentImpl<M>` - Full agent with tools/reasoning/multi-turn

### 3. Provider Abstraction

**Location:** `agent/model_adaptor.rs`

Each provider (OpenAI, Ollama, DeepSeek) is wrapped in:
- `Arc<dyn CompletionModelDyn>` for completion models
- `Arc<dyn EmbeddingModelDyn>` for embedding models

**Benefits:**
- Runtime provider selection based on config
- Easy to add new providers
- Type-erased for flexible composition

### 4. Encrypted Configuration

**Location:** `config/` + `secure/`

API keys are stored encrypted in `config.toml`:
1. Keys encrypted offline using `encryptor` binary
2. Stored as Base64 in config file
3. Decrypted at startup using `ENCRYPT_KEY` from environment
4. Never appear in logs or error messages

**Benefits:**
- Config file can be committed (with encrypted keys)
- Secrets protected at rest
- Easy key rotation (re-encrypt with new key)

### 5. Incremental Streaming Updates

**Location:** `session.rs:extract_increment_and_update()`

Handles streaming responses where the model sends:
- Full text so far: "Hello" → "Hello world" → "Hello world!"
- Only increments are displayed to avoid duplication

**Algorithm:**
- Compare new text with previous
- If new text starts with previous, extract delta
- Display only the delta

## Component Details

### Agent Module (`src/agent/`)

Core components:
- **`model_adaptor.rs`** - Creates agents from model configs
  - Provider clients (OpenAI, Ollama, DeepSeek)
  - Model validation (1-2 models, correct types)
  - Tool integration (built-in + MCP + RAG)
- **`session.rs`** - Session management and streaming
  - Type-state builder
  - ResponseSink/InputSource abstractions
  - ChatSession trait with Chat/Agent implementations
- **`cli_chat.rs`** - Terminal frontend
  - ANSI colored output
  - stdin/stdout handling
  - Implements ResponseSink + InputSource
- **`tools/`** - Agent tools
  - Calculator (arithmetic expressions)
  - Future: Browser, MCP tools

See [../src/agent/README.md](../src/agent/README.md) for details.

### Config Module (`src/config/`)

Configuration loading and validation:
- **`read_config.rs`** - Loads from `config.toml` and environment
  - Supports `WHISPER__` prefixed env vars
  - Decrypts API keys at load time
  - Skips invalid models with logging

See [../src/config/README.md](../src/config/README.md) for details.

### Secure Module (`src/secure/`)

AES-256-GCM encryption:
- **`aes.rs`** - Encrypt/decrypt functions
  - Random nonce generation per encryption
  - Base64 encoding for storage
  - AEAD for authenticity + confidentiality

See [../src/secure/README.md](../src/secure/README.md) for details.

### Binaries (`src/bin/`)

**`cli_chatbot.rs`** - Main chat application
1. Initialize logging (flexi_logger to `logs/`)
2. Load config with decryption
3. Create agent from config
4. Build session with CliFrontend
5. Run session loop until exit

**`encryptor.rs`** - Encryption utility
- Encrypt/decrypt text or files
- Used to prepare API keys for `config.toml`

## External Dependencies

### Rig Framework

Core AI agent framework:
- `rig-core` (0.21.0) - Agent, completion, embeddings, tools
- Provides abstractions over different AI providers
- Built-in RAG support with vector stores

### RMCP (Rust MCP)

Model Context Protocol support:
- `rmcp` (0.6.4) - MCP client and transports
- Enables external tool servers via SSE or stdio
- Planned for future tool integration

### Encryption

- `aes-gcm` (0.10.3) - NIST-approved AEAD cipher
- `rand` (0.9.2) - Cryptographic RNG for nonces

### Async Runtime

- `tokio` (1.47.1) - Async runtime with full features
- All I/O and agent operations are async

See `Cargo.toml` for complete dependency list.

## Extensibility Points

### Adding New Tools

1. Implement tool in `src/agent/tools/your_tool/`
2. Implement Rig's `Tool` trait
3. Register in `model_adaptor.rs:create_agent()`

See [../src/agent/tools/README.md](../src/agent/tools/README.md)

### Adding New Providers

1. Add dependency to `Cargo.toml`
2. Add match arm in `create_completion_model()` or `create_embed_model()`
3. Follow existing provider patterns (OpenAI, Ollama, DeepSeek)

### Adding New Frontends

1. Implement `ResponseSink` + `InputSource` traits
2. Pass to `Session::run()`

Examples:
- Web server (HTTP streaming)
- TUI (terminal UI with ratatui)
- GUI (egui, tauri)

### Adding MCP Server Support

1. Configure in `config.toml` under `[[mcp_servers]]`
2. Load servers in `create_agent()`
3. Register as tools with agent builder

## Logging Strategy

- **Library code:** Use `log` macros (`info!`, `debug!`, `error!`)
- **Binary initialization:** Set up `flexi_logger` in `main()`
- **Log output:** `logs/cli_chatbot_*.log` with daily rotation
- **Control level:** `RUST_LOG` environment variable

**Log locations:**
- Config loading: `info`, `error` for decryption failures
- Agent creation: `info`, `warn` for missing optional features
- Session loop: `info` for lifecycle, `debug` for user prompts
- Encryption: `debug` for operations, `error` for failures

## Security Considerations

### Credential Security

✅ **Protected:**
- API keys encrypted at rest in `config.toml`
- Encryption key (`ENCRYPT_KEY`) only in environment
- Decrypted keys never logged

❌ **Risks:**
- `ENCRYPT_KEY` exposure compromises all API keys
- Logs are written to disk (may contain sensitive prompts)
- In-memory keys accessible via debugger/core dumps

### Mitigation Strategies

- Keep `ENCRYPT_KEY` in `.env` (add to `.gitignore`)
- Rotate encryption keys periodically
- Review logs before sharing
- Use secure secret management in production

## Testing Strategy

### Unit Tests

- **Config:** Model config parsing and validation
- **Secure:** Encryption roundtrip, error cases
- **Agent:** Model adapter validation logic

Run: `cargo test --lib`

### Integration Tests

- **`tests/model_flow.rs`** - End-to-end agent workflows

Run: `cargo test --test model_flow`

### Manual Testing

- `cargo run --bin cli_chatbot` - Interactive chat testing
- `cargo run --bin encryptor` - Encryption utility testing

## Future Enhancements

Planned features (see issue tracker):

- [ ] MCP server integration (multiple tool servers)
- [ ] Browser tool implementation
- [ ] RAG document ingestion pipeline
- [ ] Web UI frontend
- [ ] Streaming API server
- [ ] Tool result caching
- [ ] Conversation persistence (SQLite)

## References

- **Rig Framework:** [Documentation](https://docs.rs/rig-core)
- **Model Context Protocol:** [MCP Specification](https://modelcontextprotocol.io)
- **AES-GCM:** [NIST SP 800-38D](https://nvlpubs.nist.gov/nistpubs/Legacy/SP/nistspecialpublication800-38d.pdf)

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for:
- Code style guidelines
- Testing requirements
- Commit conventions
- PR process
