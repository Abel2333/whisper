#!/bin/bash
# Git configuration setup for Whisper project

echo "🔧 Setting up Git configuration for Whisper..."

# Set commit message template
echo "📝 Configuring commit message template..."
git config commit.template .gitmessage
echo "✅ Commit template configured (.gitmessage)"

# Optional: Set up git hooks
# (Uncomment if you want to add commit message validation)
# echo "🪝 Setting up git hooks..."
# cp scripts/commit-msg .git/hooks/commit-msg
# chmod +x .git/hooks/commit-msg

# Optional: Set useful aliases
echo "🔗 Setting up useful Git aliases..."
git config alias.lg "log --graph --oneline --decorate --all"
git config alias.st "status -sb"
git config alias.co "checkout"
git config alias.br "branch"
git config alias.cm "commit"
git config alias.unstage "reset HEAD --"

echo ""
echo "✨ Git configuration complete!"
echo ""
echo "Usage:"
echo "  git commit          # Opens editor with template"
echo "  git lg              # Pretty log graph"
echo "  git st              # Short status"
echo ""
echo "See docs/GIT_WORKFLOW.md for full workflow guide."
