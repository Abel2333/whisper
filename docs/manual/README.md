# Manual Documentation

This directory hosts hand-written English documentation for the Whisper agent tooling.

## Guidelines

- Keep every document in English to match the repository standard.
- Capture architecture notes, tool specs, and operational guides that should version with the code.
- Link out to generated API references rather than copying them here.

## Generated Docs

`cargo doc` output or any other auto-generated Rust documentation should not be checked into version control. Generated files live under `target/doc/` (or any ad-hoc output directory) and are now ignored via `.gitignore`.
