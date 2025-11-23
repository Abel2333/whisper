# Config Module

This module handles configuration loading from `config.toml` and environment variables, with built-in support for encrypted API keys.

## Module Structure

- **`mod.rs`** - Module exports
- **`read_config.rs`** - Configuration loading and decryption logic

## Configuration Format

### config.toml Structure

```toml
# Model configurations (1-2 models supported)
[[models]]
base_url = "https://api.openai.com/v1"
api_key = "ENCRYPTED_API_KEY_HERE"
provider = "openai"                    # "openai", "ollama", or "deepseek"
model_name = "gpt-4o-mini"
model_type = "completion"              # "completion" or "embedding"
context_size = 4096

[[models]]
base_url = "https://api.openai.com/v1"
api_key = "ENCRYPTED_EMBEDDING_KEY"
provider = "openai"
model_name = "text-embedding-3-large"
model_type = "embedding"
context_size = 512

# MCP Server configurations (optional)
[[mcp_servers]]
name = "DocsIndex"
protocol = "sse"                       # "sse" or "stdio"
url = "http://localhost:8080/sse"

[[mcp_servers]]
name = "Scripts"
protocol = "stdio"
command = "python3"
args = ["scripts/tool_server.py"]
envs = { API_KEY = "${YOUR_API_KEY}" }
```

## Configuration Loading Process

### 1. Environment Setup

```bash
# .env file or shell export
export ENCRYPT_KEY="your-hex-or-base64-key"
```

The `ENCRYPT_KEY` is required for decrypting API keys stored in `config.toml`.

### 2. Load Configuration

```rust
use whisper::config::read_config::load_config;

let app_config = load_config()?;
```

**Loading process:**
1. Load `.env` file via `dotenvy::dotenv()`
2. Build config from sources:
   - `config.toml` file (optional)
   - Environment variables with prefix `WHISPER__` (e.g., `WHISPER__MODELS__0__API_KEY`)
3. Load `ENCRYPT_KEY` from environment
4. Decrypt all `api_key` fields in model configs
5. Skip models with failed decryption (logged as errors)
6. Return validated `AppConfig`

### 3. Configuration Types

```rust
pub struct AppConfig {
    pub models: Vec<ModelConfig>,
}

pub struct ModelConfig {
    pub base_url: String,
    pub api_key: String,           // Decrypted at load time
    pub provider: String,          // Normalized to lowercase
    pub model_name: String,
    pub model_type: ModelType,
    pub context_size: u32,
}

pub enum ModelType {
    Embedding,
    Completion,
    Chat,  // Not yet supported
}
```

## Encrypting API Keys

Use the `encryptor` binary to encrypt your API keys before adding them to `config.toml`:

```bash
# Set your encryption key
export ENCRYPT_KEY="your-key-here"

# Encrypt an API key
cargo run --bin encryptor -- encrypt --text "sk-your-actual-api-key"

# Output: encrypted_base64_string
# Copy this output to config.toml
```

See [../secure/README.md](../secure/README.md) for details on the encryption implementation.

## Environment Variable Overrides

You can override any configuration value using environment variables with the `WHISPER__` prefix and double underscores as separators:

```bash
# Override first model's API key
export WHISPER__MODELS__0__API_KEY="encrypted-key"

# Override first model's provider
export WHISPER__MODELS__0__PROVIDER="ollama"

# Override first model's base URL
export WHISPER__MODELS__0__BASE_URL="http://localhost:11434"
```

Array indices start at 0. Nested fields use double underscores.

## Model Configuration Rules

Validated in `agent::model_adaptor::AgentSettings::validate()`:

1. **At least one model required** - Must have 1-2 models
2. **Single model mode** - Must be a `completion` model
3. **Dual model mode** - Exactly one `completion` + one `embedding` model
4. **Provider support:**
   - Completion models: `openai`, `ollama`, `deepseek`
   - Embedding models: `openai`, `ollama`

## Error Handling

### Configuration Errors
- `config::ConfigError` - File parsing, type conversion, or source errors
- Models with failed decryption are skipped with error logs (not fatal)

### Common Issues

**ENCRYPT_KEY not set:**
```
Error: Get key bytes error!
```
**Solution:** Export `ENCRYPT_KEY` environment variable

**Invalid encrypted key:**
```
Failed to decrypt api_key for model 'gpt-4o-mini': <error>. Skipping entry.
```
**Solution:** Re-encrypt the API key with the correct `ENCRYPT_KEY`

**No valid models after decryption:**
```
AgentSettings requires at least one model config
```
**Solution:** Check logs for decryption errors, fix API keys

## Testing

See tests in `read_config.rs:tests`:
- `provider_is_lowercased` - Ensures provider names are normalized

## Example: Complete Setup

```bash
# 1. Copy example config
cp config.example.toml config.toml

# 2. Generate and set encryption key (first time only)
export ENCRYPT_KEY="$(openssl rand -hex 32)"
echo "ENCRYPT_KEY=$ENCRYPT_KEY" >> .env

# 3. Encrypt your API keys
cargo run --bin encryptor -- encrypt --text "sk-your-openai-key"
# Copy output to config.toml [[models]] api_key field

# 4. Edit config.toml with your encrypted keys
vim config.toml

# 5. Run the chatbot
cargo run --bin cli_chatbot -- --config config.toml
```

## See Also

- [Encryption Module](../secure/README.md) - AES-256-GCM implementation details
- [Agent Module](../agent/README.md) - How model configs are used to create agents
- [config.example.toml](../../config.example.toml) - Example configuration file
