#!/usr/bin/env sh
set -eu
root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
python3 -m json.tool "$root/plugins/the-gitfather/.codex-plugin/plugin.json" >/dev/null
python3 -m json.tool "$root/plugins/the-gitfather/hooks/hooks.json" >/dev/null
bin="${GITFATHER_BIN:-$root/plugins/the-gitfather/runtime/target/debug/the-gitfather}"
[ -x "$bin" ] || { echo "build first or set GITFATHER_BIN" >&2; exit 1; }
tmp=$(mktemp -d); trap 'rm -rf "$tmp"' EXIT
cat > "$tmp/blueprint.json" <<'JSON'
{"objective":"prove","components":["state"],"layers":["scaffold","behavior"],"next":"wire"}
JSON
run() { "$bin" --data-dir "$tmp" --session "$2" "$@" >/dev/null; }
"$bin" --data-dir "$tmp" --session one plan "$tmp/blueprint.json" >/dev/null
"$bin" --data-dir "$tmp" --session one mark state evidence >/dev/null
"$bin" --data-dir "$tmp" --session one advance >/dev/null
"$bin" --data-dir "$tmp" --session two status >/dev/null
printf '%s\n' '{"hook_event_name":"SessionStart","session_id":"one"}' | "$bin" --data-dir "$tmp" hook >/dev/null
printf '%s\n' "system proof passed: isolated sessions, plan, mark, advance, and hook process"
