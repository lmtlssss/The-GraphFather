#!/usr/bin/env sh
set -eu
root=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
codex="${CODEX_BIN:-codex}"; source="https://github.com/lmtlssss/The-GraphFather"; binary=""; codex_dir="${CODEX_HOME:-$HOME/.codex}"
while [ $# -gt 0 ]; do case "$1" in --binary) binary=$2;shift 2;;--source) source=$2;shift 2;;--codex) codex=$2;shift 2;;--code-home) codex_dir=$2;shift 2;;*) echo "unknown option: $1" >&2;exit 2;;esac;done
data="$codex_dir/plugins/data/the-graphfather-the-graphfather"; old="$data/the-graphfather"; backup="$data/.the-graphfather.backup.$$"; made=0
cleanup(){ if [ -n "${tmp:-}" ]; then rm -f "$tmp" "${sum:-}"; fi; }
rollback(){ if [ -e "$backup" ];then mv -f "$backup" "$old";elif [ "$made" = 1 ];then rm -f "$old";fi; }
trap cleanup EXIT HUP INT TERM
if [ -z "$binary" ];then tmp=$(mktemp);sum=$(mktemp);url=https://github.com/lmtlssss/The-GraphFather/releases/latest/download/the-graphfather-x86_64-unknown-linux-gnu;curl -fsSL "$url" -o "$tmp";curl -fsSL "$url.sha256" -o "$sum";expected=$(awk 'NF{print $1;exit}' "$sum");actual=$(sha256sum "$tmp"|awk '{print $1}');[ "$expected" = "$actual" ]||{ echo "release checksum mismatch" >&2;exit 1;};binary=$tmp;fi
[ -f "$binary" ]||{ echo "binary not found" >&2;exit 1;}
mkdir -p "$data";chmod 700 "$data";candidate="$data/.the-graphfather.candidate.$$";install -m 0755 "$binary" "$candidate";version=$($candidate --version 2>/dev/null || true);case "$version" in "the-graphfather "[0-9]*.[0-9]*.[0-9]*) ;; *) rm -f "$candidate";echo "invalid the-graphfather version: $version" >&2;exit 1;; esac
if [ -e "$old" ];then mv "$old" "$backup";else made=1;fi
if ! mv "$candidate" "$old" || { case "$source" in http://*|https://*) CODEX_HOME="$codex_dir" "$codex" plugin marketplace add "$source" --ref "v${version#the-graphfather }" && CODEX_HOME="$codex_dir" "$codex" plugin marketplace upgrade the-graphfather ;; *) CODEX_HOME="$codex_dir" "$codex" plugin marketplace add "$source" ;; esac; } || ! CODEX_HOME="$codex_dir" "$codex" plugin add the-graphfather@the-graphfather || ! CODEX_HOME="$codex_dir" CODEX_BIN="$codex" "$old" --data-dir "$data" trust;then rm -f "$candidate";rollback;echo "install rolled back" >&2;exit 1;fi
rm -f "$backup";printf '%s\n' "the-graphfather installed at $old"
