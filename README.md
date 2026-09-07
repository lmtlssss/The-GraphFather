# the-gitfather

“i’m gonna make him an offer he can’t refuse.” — for codex agents with commitment issues.

the-gitfather stores a local cursor for each Codex conversation and guards lifecycle checks. it does not read transcripts, call a model, replace the launcher, commit to git, or run a daemon.

```text
blueprint -> build -> proof -> repair -> complete
```

## install

```sh
./install.sh --binary ./the-gitfather
```

without `--binary`, the script downloads the latest Linux x86_64 release. the executable is installed atomically under `${CODEX_HOME:-$HOME/.codex}/plugins/data/the-gitfather-the-gitfather/`.

## usage

```sh
the-gitfather status
the-gitfather plan blueprint.json
the-gitfather check smoke compile -- cargo check --locked
```

use `--session ID` or `CODEX_THREAD_ID`, and `--data-dir DIR` for isolated tests. `trust` and `untrust` change only this plugin's hooks.

## state

private SQLite state stores bounded project fields, receipts, and events; stdin is capped at 128 KiB. hooks are guardrails, not proof of semantic truth, and hosted or arbitrary shell work is not fully observable.

## uninstall

`./uninstall.sh` removes this executable and registration while preserving state and unrelated configuration.

## build

`cargo build --release --locked --manifest-path plugins/the-gitfather/runtime/Cargo.toml`

Linux x86_64 releases include a sha256 checksum.
