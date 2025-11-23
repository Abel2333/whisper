# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Git Flow workflow documentation in `docs/GIT_WORKFLOW.md`
- Conventional Commits message template in `.gitmessage`
- MIT LICENSE file
- GitHub Actions CI workflow (fmt, clippy, test, build)
- Comprehensive documentation structure:
  - `docs/ARCHITECTURE.md` - System architecture
  - `docs/CONTRIBUTING.md` - Development guidelines
  - `docs/CLAUDE.md` - AI assistant guide
  - Module-level README files for agent, config, secure, tools

### Fixed
- Chinese character deletion display issue in CLI
  - Characters now properly cleared based on their display width
  - Added unicode-width dependency

### Changed
- Reorganized documentation from flat structure to hierarchical (docs/ + src/*/README.md)
- Moved AGENTS.md to docs/CONTRIBUTING.md for standard naming
- Enhanced Cargo.toml with metadata (description, keywords, categories)
- Updated .gitignore to exclude config.toml and backup files
- Moved examples to examples/ directory

### Removed
- Unused dependencies (async-trait, serde_toml) from documentation branch
- config.toml from version control (now only config.example.toml is tracked)
- Backup files (*.bak) from repository

## [0.2.0] - 2025-01-XX

### Added
- Multi-turn CLI chat runtime with streaming responses
- AES-256-GCM encryption helper for secure credential storage
- Calculator tool with expression parsing and evaluation
- Support for OpenAI, Ollama, and DeepSeek providers
- Configuration system with encrypted API keys
- Structured logging to `logs/` directory

### Security
- API keys stored encrypted in config.toml
- Credentials never committed to version control

## [0.1.0] - Initial Release

### Added
- Initial project structure
- Basic agent framework integration with Rig
- Configuration loading system
- MCP server support foundation

---

## Version Format

- **Major.Minor.Patch** (e.g., 1.2.3)
- **Major** - Breaking changes
- **Minor** - New features (backwards compatible)
- **Patch** - Bug fixes (backwards compatible)

## Categories

- **Added** - New features
- **Changed** - Changes in existing functionality
- **Deprecated** - Soon-to-be removed features
- **Removed** - Removed features
- **Fixed** - Bug fixes
- **Security** - Security fixes
