# Agent Module

This module implements the core agent infrastructure for Whisper, built on top of the Rig framework with MCP (Model Context Protocol) support.

## Module Structure

- **`mod.rs`** - Module exports
- **`model_adaptor.rs`** - Agent creation and model provider adapters
- **`session.rs`** - Type-state session builder and streaming response handling
- **`cli_chat.rs`** - Terminal frontend implementation
- **`dyn_embedding_wrapper.rs`** - Dynamic embedding model wrapper for vector store integration
- **`tools/`** - Built-in agent tools (see [tools/README.md](tools/README.md))

## Architecture

### Type-State Session Builder

The `SessionBuilder` uses compile-time type states to ensure proper configuration before building a session:

```rust
// Start with no implementation
SessionBuilder::new()
    // Provide an agent (transitions to AgentImpl state)
    .agent(agent)
    // Configure multi-turn depth
    .multi_turn_depth(4)
    // Enable usage tracking
    .show_usage()
    // Build the session
    .build()
```

**Type-state transitions:**
- `SessionBuilder<NoImplProvided>` → initial state
- `SessionBuilder<AgentImpl<M>>` → after calling `.agent(agent)`
- `SessionBuilder<ChatImpl<T>>` → after calling `.chat(chatbot)`
- `Session<T>` → final built session

This pattern prevents runtime errors by enforcing configuration completeness at compile time.

### Response Sink Abstraction

The `ResponseSink` trait abstracts output handling, allowing different frontends without changing agent logic:

```rust
pub trait ResponseSink {
    fn chat_start(&mut self) -> ...;
    fn user_start(&mut self) -> ...;
    fn output_start(&mut self) -> ...;
    fn output_text(&mut self, content: &dyn Display) -> ...;
    fn output_reason_start(&mut self) -> ...;
    fn output_reason_end(&mut self) -> ...;
    fn output_finished(&mut self, usage: &Option<Usage>) -> ...;
    fn chat_finished(&mut self) -> ...;
    fn output_error(&mut self, e: &dyn Display) -> ...;
}
```

**Current implementations:**
- `CliFrontend` in `cli_chat.rs` - Terminal-based UI with colored output

### Input Source Abstraction

The `InputSource` trait abstracts user input:

```rust
pub trait InputSource {
    fn read_input(&mut self) -> Pin<Box<dyn Future<Output = Result<Option<String>, SinkError>>>>;
}
```

`CliFrontend` also implements this, reading from stdin.

### Chat Session Flow

The `ChatSession` trait defines the core interaction loop:

```rust
pub trait ChatSession {
    fn request<'a, S: ResponseSink + 'a>(
        &'a mut self,
        prompt: &'a str,
        chat_log: Vec<Message>,
        sink: &'a mut S,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<String>> + 'a>>;
}
```

**Implementations:**
- `ChatImpl<T>` - Simple chat without tool support
- `AgentImpl<M>` - Full agent with tools, reasoning, and multi-turn support

## Multi-turn Agent Flow

1. **User input** is read via `InputSource::read_input()`
2. **Request processing** starts via `ChatSession::request()`
3. **Streaming response** from the agent is categorized into:
   - `Text` - Normal assistant response text
   - `Reasoning` - Model's internal reasoning (if supported)
   - `ToolCall` - Function calls to tools
   - `FinalResponse` - Contains usage statistics
4. **Each stream item** is routed to appropriate `ResponseSink` methods:
   - Text → `output_text()`
   - Reasoning → `output_reason_start()` + `output_text()` + `output_reason_end()`
   - Tool calls → `output_text()` with call description
5. **Incremental updates** are handled by `extract_increment_and_update()` to avoid duplicate output
6. **Turn completion** - User and assistant messages are appended to `chat_log`
7. **Loop continues** until input source returns `None`

## Model Configuration

Supports 1-2 models per agent, validated at startup:

### Single Model (Required)
```toml
[[models]]
model_type = "completion"
# ... other config
```

### Dual Model (Optional - Enables RAG)
```toml
[[models]]
model_type = "completion"
# ... completion model config

[[models]]
model_type = "embedding"
# ... embedding model config
```

When an embedding model is provided along with a `ToolSet` and documents, the agent uses RAG (Retrieval-Augmented Generation) for dynamic tool selection.

## Supported Providers

Configured in `model_adaptor.rs`:

- **OpenAI** - GPT models, text-embedding models
- **Ollama** - Local models via Ollama server
- **DeepSeek** - DeepSeek models

Each provider is instantiated via Rig's client builders and wrapped in `Arc<dyn CompletionModelDyn>` or `Arc<dyn EmbeddingModelDyn>`.

## Creating an Agent

```rust
use whisper::agent::{model_adaptor::{AgentSettings, create_agent}};

// 1. Create agent settings
let settings = AgentSettings::try_new(
    model_configs,
    tool_set,          // Optional ToolSet for MCP tools
    preamble,          // System prompt
    temperature,
)?;

// 2. Create the agent
let agent = create_agent::<ToolSchema, Vec<ToolSchema>>(
    settings,
    docs,  // Optional documents for RAG
).await?;

// 3. Build a session
let session = SessionBuilder::new()
    .agent(agent)
    .multi_turn_depth(4)
    .show_usage()
    .build();

// 4. Run with a frontend
let mut frontend = CliFrontend::new();
session.run(&mut frontend).await?;
```

## Error Handling

- `SinkError` - Errors from `ResponseSink` or `InputSource` operations
  - `Io(io::Error)` - Underlying I/O errors
  - `Output(String)` - Output logic errors
  - `Other(String)` - Uncategorized errors

- Agent creation and request errors are wrapped in `anyhow::Error` for easy propagation

## Testing

See tests in:
- `model_adaptor.rs:tests` - Model configuration validation
- `session.rs:tests` - Session builder and streaming logic (if present)
- `tests/model_flow.rs` - Integration tests

## See Also

- [Tools Documentation](tools/README.md) - Built-in tools and how to add new ones
- [Configuration Module](../config/README.md) - How model configs are loaded
- [docs/ARCHITECTURE.md](../../docs/ARCHITECTURE.md) - Overall system architecture
