#!/usr/bin/env sh
set -eu
data="${CODEX_HOME:-$HOME/.codex}/plugins/data/the-gitfather-the-gitfather"
if [ -x "$data/the-gitfather" ]; then "$data/the-gitfather" untrust >/dev/null 2>&1 || true; fi
rm -f "$data/the-gitfather"
printf '%s\n' "the-gitfather executable removed; state preserved"
