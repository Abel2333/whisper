# Secure Module

This module provides AES-256-GCM encryption and decryption utilities for securely storing API keys and other sensitive data.

## Module Structure

- **`mod.rs`** - Module exports and `load_key_from_env()` helper
- **`aes.rs`** - Core AES-256-GCM encryption/decryption implementation

## Overview

The secure module is used in two main contexts:
1. **Configuration loading** - Decrypting API keys from `config.toml`
2. **Encryptor binary** - Standalone tool for encrypting/decrypting text

## Encryption Scheme: AES-256-GCM

- **Algorithm:** AES-256 in Galois/Counter Mode (GCM)
- **Key size:** 256 bits (32 bytes)
- **Nonce size:** 96 bits (12 bytes)
- **Authentication:** Built-in AEAD (Authenticated Encryption with Associated Data)

### Why AES-256-GCM?

- **Confidentiality:** Strong encryption with 256-bit keys
- **Integrity:** Authentication tag prevents tampering
- **Performance:** Hardware-accelerated on modern CPUs
- **Standard:** NIST-approved, widely supported

## Usage

### Encrypting Data

```rust
use whisper::secure::aes::{encrypt, decrypt};

// 32-byte key (256 bits)
let key_bytes: [u8; 32] = /* your key */;

// Encrypt plaintext
let plaintext = "sk-your-api-key-here";
let encrypted = encrypt(plaintext, &key_bytes)?;
// Returns Base64-encoded string: nonce (12 bytes) + ciphertext + auth tag
```

**Output format:** Base64( nonce || ciphertext || tag )
- Nonce is prepended to allow decryption
- Authentication tag ensures integrity

### Decrypting Data

```rust
use whisper::secure::aes::decrypt;

let encrypted = "base64-encoded-ciphertext";
let key_bytes: [u8; 32] = /* same key */;

let plaintext = decrypt(&encrypted, &key_bytes)?;
// Returns original plaintext if key is correct and data is valid
```

### Loading Key from Environment

```rust
use whisper::secure::load_key_from_env;

// Load ENCRYPT_KEY from environment
let key_bytes = load_key_from_env("ENCRYPT_KEY")?;

// Use with encrypt/decrypt
let encrypted = encrypt("secret", &key_bytes)?;
```

Supported key formats:
- **Hex string** (64 hex chars = 32 bytes)
- **Base64 string** (44 chars with padding)
- **Raw bytes** (if passed directly)

## Command-line Tool

The `encryptor` binary provides a convenient interface:

### Encrypt a value

```bash
export ENCRYPT_KEY="$(openssl rand -hex 32)"
cargo run --bin encryptor -- encrypt --text "sk-my-api-key"
```

Output: Base64-encoded encrypted string

### Decrypt a value

```bash
cargo run --bin encryptor -- decrypt --text "base64-encrypted-string"
```

Output: Original plaintext

### Encrypt from file

```bash
cargo run --bin encryptor -- encrypt --input secrets.txt
```

## Key Management

### Generating a Key

**Recommended: Use `openssl` or similar cryptographic tools**

```bash
# Generate 32 random bytes as hex (64 characters)
openssl rand -hex 32

# Or as base64 (44 characters)
openssl rand -base64 32
```

**Store securely:**
```bash
# In .env file (not committed to git)
echo "ENCRYPT_KEY=$(openssl rand -hex 32)" >> .env

# Or export directly
export ENCRYPT_KEY="your-generated-key"
```

### Key Storage Best Practices

✅ **DO:**
- Store `ENCRYPT_KEY` in `.env` file (add to `.gitignore`)
- Use environment variables in production
- Use secure secret management (AWS Secrets Manager, HashiCorp Vault, etc.)
- Rotate keys periodically
- Use different keys for different environments (dev/staging/prod)

❌ **DON'T:**
- Commit `ENCRYPT_KEY` to git
- Hardcode keys in source code
- Share keys via email/chat
- Reuse keys across projects
- Use weak keys (like "password123")

## Security Considerations

### Nonce Generation

Each encryption operation generates a fresh random nonce using `OsRng` (OS-provided cryptographically secure RNG). This ensures:
- Nonce uniqueness (critical for GCM security)
- Unpredictability (prevents attacks)

**IMPORTANT:** Never reuse a (key, nonce) pair - this breaks GCM security!

### Key Size

Must be exactly 32 bytes (256 bits):
```rust
let key_bytes: [u8; 32] = /* ... */;
// Key length is enforced by type system
```

Attempting to use a different key size will cause a panic or error.

### Error Handling

```rust
match decrypt(encrypted, &key_bytes) {
    Ok(plaintext) => { /* success */ }
    Err(e) => {
        // Common errors:
        // - "Base64 decode failed" - Invalid Base64 encoding
        // - "Decryption failed" - Wrong key or tampered data
        // - UTF-8 conversion error - Decrypted bytes not valid UTF-8
    }
}
```

**Decryption failures can indicate:**
- Wrong encryption key
- Corrupted/tampered ciphertext
- Data encrypted with different key
- Not actually encrypted data

### Attack Resistance

AES-256-GCM provides:
- **Confidentiality:** Ciphertext reveals nothing about plaintext
- **Integrity:** Authentication tag detects any modification
- **Authenticity:** Ensures data came from key holder

**Not protected against:**
- Key compromise (secure your `ENCRYPT_KEY`!)
- Side-channel attacks (timing, power analysis) - use in trusted environments
- Replay attacks (if attacker can resubmit old encrypted values)

## Integration with Config Module

The config module uses this module to decrypt API keys:

```rust
// In config/read_config.rs
use crate::secure::{self, load_key_from_env};

let key_bytes = load_key_from_env("ENCRYPT_KEY")?;
for mut model in config.models {
    match secure::aes::decrypt(&model.api_key, &key_bytes) {
        Ok(decrypted) => {
            model.api_key = decrypted;
            valid_models.push(model);
        }
        Err(e) => {
            log::error!("Failed to decrypt api_key: {}", e);
            // Skip this model
        }
    }
}
```

See [../config/README.md](../config/README.md) for details.

## Testing

Tests in `aes.rs:tests`:
- `encrypt_then_decrypt_roundtrip` - Verifies encryption/decryption correctness
- `decrypt_rejects_bad_base64` - Ensures invalid input is rejected

Run tests:
```bash
cargo test --package whisper --lib secure::aes
```

## Dependencies

- `aes-gcm` (0.10.3) - AES-GCM implementation
- `base64` (0.22.1) - Base64 encoding/decoding
- `anyhow` (1.0.99) - Error handling

## See Also

- [Configuration Module](../config/README.md) - How encrypted keys are loaded
- [AES-GCM Specification](https://nvlpubs.nist.gov/nistpubs/Legacy/SP/nistspecialpublication800-38d.pdf) - NIST SP 800-38D
- [Encryptor Binary](../bin/encryptor.rs) - Command-line tool implementation
