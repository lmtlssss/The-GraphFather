# the graphfather

“i’m gonna make him an offer he can’t refuse.” — for Codex agents with commitment issues.

project structure and build sequencing for Codex conversations.

finish the scaffold. complete each layer across the whole project. test the
joined system, then repair the parts that failed.

```text
THE GRAPHFATHER
──────────────────────────────────────────────────────────────

conversation  ──►  blueprint  ──►  whole scaffold
                                      │
                                      ▼
                                 whole layers
                                      │
                                      ▼
                               integrated proof
                                      │
                                narrow repairs
                                      │
                                      ▼
                                  completion

session + cursor + receipts  ──►  local SQLite  ──►  resume
```

## install

Linux x86_64, with Codex installed.

```bash
curl -fsSL https://raw.githubusercontent.com/lmtlssss/The-GraphFather/main/install.sh | sh
```

inspect first:

```bash
curl -fsSLO https://raw.githubusercontent.com/lmtlssss/The-GraphFather/main/install.sh
less install.sh
sh install.sh
```

the installer verifies the release checksum, registers the plugin, and trusts
only its current hook definitions. existing Codex plugins and model settings stay
in place.

## usage

use Codex normally. each conversation receives its own project cursor.

the agent records the blueprint and next action. subagents use the parent
project. a short task gets a short plan; the person does not fill out forms.

```text
01  map the complete scope and ordered layers
02  assemble every component of the scaffold
03  complete the current layer across all components
04  advance only when the layer is complete
05  run the integrated check and retain failures
06  repair the affected components
07  confirm the whole system and finish
```

the cursor survives a new turn, resume, and compaction. CompactVeteran continues
to own its handoff behavior; RecentlyDivorced continues to own conversation labels.

## controls

hooks supply the installed command path and conversation ID to the agent.

```text
command                         action
──────────────────────────────────────────────────────────────
status                          read project state
plan FILE                       register the blueprint
cursor TEXT                     save the exact next action
mark COMPONENT EVIDENCE         complete work in this layer
advance                         cross the whole-layer gate
check KIND LABEL -- COMMAND     run and record a check
issue COMPONENT REASON          open a narrow repair
changed REASON                  invalidate outdated proof
finish                          require current whole proof
reset REASON                    archive state for a new task
doctor                          inspect local state storage
```

check kinds:

```text
smoke      syntax, type and build checks during construction
whole      the joined system, in the proof phase
narrow     affected work with an open repair issue
safety     a named imminent effect, with --reason
```

an identical check at the same input generation is not run again, even under a
different label. changed inputs require an explicit reason or an observed patch.
check receipts retain the argument hash and result, not raw command arguments.

## state

```text
${CODEX_HOME:-$HOME/.codex}/plugins/data/the-graphfather-the-graphfather/
    the-graphfather
    state.sqlite
```

state is private and local. routine hooks do not call a model or read transcripts.
separate conversations in the same directory keep separate records.

hooks guard supported tool calls and recognized check commands. they cannot prove
that an agent's completion claim is true or classify every arbitrary program.
hosted tools and specialized hook opt-outs remain outside that enforcement.

## uninstall

```bash
curl -fsSL https://raw.githubusercontent.com/lmtlssss/The-GraphFather/main/uninstall.sh | sh
```

the executable and plugin registration are removed. project state is retained.

## build

```bash
cargo build --release --locked --manifest-path plugins/the-graphfather/runtime/Cargo.toml
cargo test --locked --manifest-path plugins/the-graphfather/runtime/Cargo.toml
GRAPHFATHER_BIN=plugins/the-graphfather/runtime/target/release/the-graphfather scripts/prove-system.sh
```

install a local build:

```bash
./install.sh --source "$PWD" --binary plugins/the-graphfather/runtime/target/release/the-graphfather
```
