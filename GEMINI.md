# Project Overview

This project, named "whisper," is a Rust-based playground for experimenting with AI agents. It includes two main components: a command-line chat application (`cli_chatbot`) and a utility for encryption/decryption (`encryptor`). The project is built using Rust and leverages the `rig-core` crate for agent functionalities. It uses `tokio` for asynchronous operations and `clap` for command-line argument parsing.

# Building and Running

## Prerequisites

-   Install the latest stable Rust toolchain: `rustup default stable`
-   Copy the example configuration file and set the `ENCRYPT_KEY` environment variable:
    ```bash
    cp config.example.toml config.toml
    export ENCRYPT_KEY="your-hex-or-base64-key"
    ```

## Running the Applications

-   **Chatbot:**
    ```bash
    cargo run --bin cli_chatbot -- --config config.toml
    ```
-   **Encryptor:**
    ```bash
    # Encrypt
    cargo run --bin encryptor -- encrypt --text "your secret text"

    # Decrypt
    cargo run --bin encryptor -- decrypt --text "your encrypted text"
    ```

## Testing

-   Run all tests:
    ```bash
    cargo test --all -- --nocapture
    ```

# Development Conventions

## Code Formatting

-   Format the code using `cargo fmt`:
    ```bash
    cargo fmt --all
    ```

## Linting

-   Check for warnings and errors using `clippy`:
    ```bash
    cargo clippy --all-targets --all-features -D warnings
    ```

## Documentation

-   Generate documentation:
    ```bash
    cargo doc --no-deps
    rm -rf docs && mkdir -p docs
    cp -r target/doc/. docs/
    ```
-   The generated documentation can be found in the `docs/` directory.

## Versioning

-   The project follows semantic versioning. The current version is 0.2.0.
