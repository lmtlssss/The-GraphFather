#!/usr/bin/env sh
set -eu
codex="${CODEX_BIN:-codex}"; codex_dir="${CODEX_HOME:-$HOME/.codex}"
while [ $# -gt 0 ];do case "$1" in --codex)codex=$2;shift 2;;--code-home)codex_dir=$2;shift 2;;*)echo "unknown option: $1" >&2;exit 2;;esac;done
data="$codex_dir/plugins/data/the-graphfather-the-graphfather"; bin="$data/the-graphfather"
if [ -x "$bin" ];then CODEX_HOME="$codex_dir" CODEX_BIN="$codex" "$bin" --data-dir "$data" untrust;fi
CODEX_HOME="$codex_dir" "$codex" plugin remove the-graphfather@the-graphfather
rm -f "$bin"
printf '%s\n' "the-graphfather executable and registration removed; state preserved"
