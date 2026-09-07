#!/usr/bin/env sh
set -eu
codex="${CODEX_BIN:-codex}"; source="${GITFATHER_SOURCE:-$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)}"; data="${CODEX_HOME:-$HOME/.codex}/plugins/data/the-gitfather-the-gitfather"; binary=""
while [ $# -gt 0 ]; do case "$1" in --binary) binary=$2; shift 2;; --codex) codex=$2; shift 2;; --source) source=$2; shift 2;; *) echo "unknown option: $1" >&2; exit 2;; esac; done
if [ -z "$binary" ]; then binary=$(mktemp); sum=$(mktemp); trap 'rm -f "$binary" "$sum"' EXIT; url=https://github.com/lmtlssss/The-GitFather/releases/latest/download/the-gitfather-x86_64-unknown-linux-gnu; curl -fsSL "$url" -o "$binary"; curl -fsSL "$url.sha256" -o "$sum"; (cd "$(dirname "$binary")" && sha256sum -c "$sum"); fi
[ -f "$binary" ] || { echo "binary not found" >&2; exit 1; }; mkdir -p "$data"; tmp="$data/.the-gitfather.tmp.$$"; install -m 0755 "$binary" "$tmp"; mv -f "$tmp" "$data/the-gitfather"
"$codex" plugin marketplace add "$source" >/dev/null 2>&1 || true; "$codex" plugin add the-gitfather@the-gitfather >/dev/null; "$data/the-gitfather" trust
printf '%s\n' "the-gitfather installed at $data/the-gitfather"
