#!/usr/bin/env bash
# Copies Claude Code chats/settings from .claude-backup/ into the l562-claude volume.
# Runs from postCreateCommand; only once per volume and never overwrites existing files.
#
# Make a backup (inside the container):
#   mkdir -p .claude-backup && cp -r /root/.claude/projects .claude-backup/ && cp /root/.claude.json .claude-backup/
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SRC="$ROOT/.claude-backup"
DST="${CLAUDE_CONFIG_DIR:-$HOME/.claude}"
MARK="$DST/.restored-from-backup"

[[ -d "$SRC" ]] || exit 0
[[ -e "$MARK" ]] && exit 0

mkdir -p "$DST"
cp -r --update=none "$SRC"/. "$DST"/
touch "$MARK"
echo "Claude Code data restored from $SRC into $DST"
