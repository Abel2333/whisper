# Git Workflow Guide

This document describes the Git Flow workflow and commit conventions used in this project.

## Branch Structure

### Permanent Branches

- **`main`** - Production-ready code. Only stable releases.
- **`develop`** - Integration branch. All features merge here first.

### Temporary Branches

#### Feature Branches
- **Naming:** `feature/<short-description>`
- **Branch from:** `develop`
- **Merge to:** `develop`
- **Purpose:** Develop new features

```bash
# Create feature branch
git checkout develop
git checkout -b feature/user-authentication

# Work on feature...

# Merge back to develop
git checkout develop
git merge --no-ff feature/user-authentication
git branch -d feature/user-authentication
```

#### Release Branches
- **Naming:** `release/<version>`
- **Branch from:** `develop`
- **Merge to:** `main` AND `develop`
- **Purpose:** Prepare for production release

```bash
# Create release branch
git checkout develop
git checkout -b release/0.3.0

# Bump version, update changelog, fix bugs...

# Merge to main
git checkout main
git merge --no-ff release/0.3.0
git tag -a v0.3.0 -m "Release version 0.3.0"

# Merge to develop
git checkout develop
git merge --no-ff release/0.3.0

# Delete release branch
git branch -d release/0.3.0
```

#### Hotfix Branches
- **Naming:** `hotfix/<issue-description>`
- **Branch from:** `main`
- **Merge to:** `main` AND `develop`
- **Purpose:** Emergency production fixes

```bash
# Create hotfix branch
git checkout main
git checkout -b hotfix/critical-security-fix

# Fix the issue...

# Merge to main
git checkout main
git merge --no-ff hotfix/critical-security-fix
git tag -a v0.2.1 -m "Hotfix: critical security fix"

# Merge to develop
git checkout develop
git merge --no-ff hotfix/critical-security-fix

# Delete hotfix branch
git branch -d hotfix/critical-security-fix
```

## Conventional Commits

We follow the [Conventional Commits](https://www.conventionalcommits.org/) specification.

### Commit Message Format

```
<type>(<scope>): <subject>

<body>

<footer>
```

### Types

- **`feat:`** - New feature
- **`fix:`** - Bug fix
- **`docs:`** - Documentation changes
- **`style:`** - Code style changes (formatting, missing semicolons, etc.)
- **`refactor:`** - Code refactoring (neither fixes a bug nor adds a feature)
- **`perf:`** - Performance improvements
- **`test:`** - Adding or updating tests
- **`build:`** - Build system or external dependencies (e.g., Cargo.toml)
- **`ci:`** - CI/CD configuration changes
- **`chore:`** - Other changes (maintenance, tooling, etc.)
- **`revert:`** - Reverts a previous commit

### Scopes (Optional)

Scopes indicate which part of the codebase is affected:

- `agent` - Agent module
- `config` - Configuration module
- `secure` - Security/encryption module
- `cli` - CLI interface
- `tools` - Agent tools
- `docs` - Documentation

### Examples

#### Good Commits

```
feat(agent): add RAG-based tool selection

Implement retrieval-augmented generation for dynamic tool selection
using embedding similarity. This allows the agent to choose relevant
tools based on the user's query context.

Closes #42

---

fix(cli): resolve Chinese character deletion display issue

Fixed backspace handling to properly clear wide characters (e.g., Chinese).
The issue was that backspace only cleared 1 display width, but Chinese
characters occupy 2 display widths.

- Add unicode-width dependency
- Calculate character display width before deletion
- Clear appropriate number of terminal cells

Fixes #38

---

docs: reorganize project documentation structure

Move documentation to docs/ and add module-level README files:
- docs/ARCHITECTURE.md - System architecture
- docs/CONTRIBUTING.md - Development guidelines
- src/*/README.md - Module-specific documentation

---

chore: add MIT license and CI/CD workflow

- Add LICENSE file
- Create GitHub Actions workflow for CI
- Update .gitignore to exclude config.toml
```

#### Bad Commits (Don't Do This)

```
❌ updated stuff
❌ fix bug
❌ WIP
❌ asdf
❌ Fixed the thing that was broken
❌ Added feature
```

### Subject Line Rules

1. **Use imperative mood** - "add" not "added" or "adds"
2. **Don't capitalize first letter** - `feat: add feature` not `Feat: Add feature`
3. **No period at the end** - `fix: resolve issue` not `fix: resolve issue.`
4. **Limit to 72 characters**
5. **Be specific and descriptive**

### Body Guidelines

- Separate subject from body with a blank line
- Wrap body at 72 characters
- Explain **what** and **why**, not **how**
- Use bullet points for multiple changes
- Reference issues and pull requests

### Footer

- **Breaking Changes:** `BREAKING CHANGE: <description>`
- **Issue References:** `Fixes #123`, `Closes #456`, `Refs #789`
- **Co-authors:** `Co-Authored-By: Name <email>`

## Workflow Examples

### Adding a New Feature

```bash
# 1. Start from develop
git checkout develop
git pull origin develop

# 2. Create feature branch
git checkout -b feature/mcp-server-integration

# 3. Work on feature with conventional commits
git add src/mcp/
git commit -m "feat(mcp): add SSE transport support

Implement Server-Sent Events transport for MCP servers.
This allows connecting to remote tool servers via HTTP.

- Add SSE client implementation
- Handle reconnection logic
- Add timeout configuration"

# 4. Continue working...
git commit -m "feat(mcp): add stdio transport support"
git commit -m "test(mcp): add transport integration tests"
git commit -m "docs(mcp): document MCP server configuration"

# 5. Merge to develop (you'll do this manually or via PR)
git checkout develop
git merge --no-ff feature/mcp-server-integration
git branch -d feature/mcp-server-integration

# 6. Push to remote
git push origin develop
```

### Creating a Release

```bash
# 1. Create release branch from develop
git checkout develop
git checkout -b release/0.3.0

# 2. Bump version
# Edit Cargo.toml: version = "0.3.0"
git add Cargo.toml
git commit -m "build: bump version to 0.3.0"

# 3. Update changelog
# Edit CHANGELOG.md
git add CHANGELOG.md
git commit -m "docs: update CHANGELOG for v0.3.0"

# 4. Merge to main
git checkout main
git merge --no-ff release/0.3.0
git tag -a v0.3.0 -m "Release version 0.3.0"

# 5. Merge back to develop
git checkout develop
git merge --no-ff release/0.3.0

# 6. Delete release branch
git branch -d release/0.3.0

# 7. Push everything
git push origin main develop --tags
```

## Git Aliases (Optional)

Add these to your `~/.gitconfig` for convenience:

```ini
[alias]
    # Git Flow shortcuts
    feature-start = "!f() { git checkout develop && git pull && git checkout -b feature/$1; }; f"
    feature-finish = "!f() { git checkout develop && git merge --no-ff feature/$1 && git branch -d feature/$1; }; f"

    # Conventional commit helpers
    feat = "!f() { git commit -m \"feat: $*\"; }; f"
    fix = "!f() { git commit -m \"fix: $*\"; }; f"
    docs = "!f() { git commit -m \"docs: $*\"; }; f"
    style = "!f() { git commit -m \"style: $*\"; }; f"
    refactor = "!f() { git commit -m \"refactor: $*\"; }; f"
    test = "!f() { git commit -m \"test: $*\"; }; f"
    chore = "!f() { git commit -m \"chore: $*\"; }; f"

    # Other useful aliases
    lg = log --graph --oneline --decorate --all
    st = status -sb
    co = checkout
    br = branch
    cm = commit
    unstage = reset HEAD --
```

Usage:
```bash
git feature-start user-auth
# ... work on feature ...
git add .
git feat "add user authentication with JWT"
git feature-finish user-auth
```

## Commit Message Template

Create `.gitmessage` in project root:

```
# <type>(<scope>): <subject>
#
# <body>
#
# <footer>

# Type: feat, fix, docs, style, refactor, perf, test, build, ci, chore, revert
# Scope: agent, config, secure, cli, tools, docs (optional)
# Subject: imperative mood, lowercase, no period, max 72 chars
#
# Body: explain what and why, wrap at 72 chars
#
# Footer: BREAKING CHANGE, Fixes #123, Closes #456
```

Configure Git to use it:
```bash
git config commit.template .gitmessage
```

## Tools

### Commitlint (Optional)

Install commitlint to enforce commit conventions:

```bash
npm install -g @commitlint/cli @commitlint/config-conventional
```

Create `commitlint.config.js`:
```javascript
module.exports = {
  extends: ['@commitlint/config-conventional']
};
```

Add to `.git/hooks/commit-msg`:
```bash
#!/bin/sh
npx commitlint --edit $1
```

### Git Flow CLI (Optional)

Install git-flow extension for easier branch management:

```bash
# macOS
brew install git-flow

# Linux
apt-get install git-flow
```

Initialize:
```bash
git flow init
# Accept defaults: main, develop, feature/, release/, hotfix/
```

Usage:
```bash
git flow feature start user-auth
# ... work ...
git flow feature finish user-auth
```

## References

- [Git Flow Model](https://nvie.com/posts/a-successful-git-branching-model/)
- [Conventional Commits](https://www.conventionalcommits.org/)
- [Semantic Versioning](https://semver.org/)
