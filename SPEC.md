# the graphfather — build contract

## v0.2.1 explicit handoff bridge

CompactVeteran may provide a bounded Scope field naming a planned GraphFather
session. SessionStart records an alias only for that explicit origin; unknown or
unplanned origins remain independent. Aliases preserve the canonical SQLite
ledger, receipts, marks, revisions, and Markdown plan across handoffs and
env-free resume. Working-directory equality is never an identity signal.

## v0.2 live-plan amendment (latest person authority)

the person approved a living Markdown plan per conversation, reconciled on every
user turn by the agent already handling that turn. write only actual changes;
preserve valid work and proof. do not modify Codex's native /plan, add model calls,
or infer semantic scope changes in a Rust hook. this amendment supersedes the
fixed-blueprint limitation below. release as v0.2.0.

keep SQLite as the transactional revision/receipt ledger. publish its accepted
plan and progress as a private, readable Markdown file at
PLUGIN_DATA/plans/<sha256-session-id>.md. the agent maintains it through the CLI,
not by independently editing two authorities. status and lifecycle hooks return
the absolute plan path and revision. Markdown includes objective, components and
dependencies, layer progress, exact next action and recent superseded revisions.
atomic replacement, 0600 files/0700 directories; unchanged content keeps mtime.
recover a missing/stale Markdown projection from committed state on access.
serialize publication with state writers so an older writer cannot overwrite a
newer plan. old v0.1 documents load with defaulted new fields; retain all records.

extend blueprint with optional dependencies: {component: [prerequisite IDs]}.
reject unknown IDs, self edges, duplicates and cycles. omitted means no edges.

add `revise FILE` (also `-` stdin) accepting:
```
{"expected_revision":1,"reason":"add the export view",
 "blueprint":{"objective":"build the app","components":["api","ui","export"],
 "layers":["scaffold","behavior"],"next":"scaffold export",
 "dependencies":{"ui":["api"],"export":["api"]}},
 "invalidate":{"ui":"behavior"}}
```
revision is an optimistic concurrency guard. reject stale revisions atomically.
validate everything before mutation. identical blueprint with no invalidation is
a no-op: no new revision/generation/event or Markdown rewrite. cursor unchanged
is also a no-op. initial plan gets revision 1; old documents may start at 0.

preserve marks for surviving component/layer IDs unless explicitly invalidated.
invalidate names the earliest affected layer; clear that layer and later marks
for the component and transitive dependents. added components start at scaffold.
changed dependency edges invalidate the changed component and dependents from
scaffold; use old and new edges to account for removal. removed components/layers
are removed from current marks but remain in the archived prior state. new layers start
unmarked across all components. choose the earliest incomplete layer after an
amendment; if none, proof. preserve unresolved surviving issues and repair flow
when no earlier construction is required; do not silently lose an open issue.
next-action-only revision preserves phase and proof. semantic/scope revisions
invalidate global whole proof and reopen completion, but retain unaffected marks.
archive prior plan/state with bounded reason in the existing events table.

add optional global `--revision N` for guarded writes (mark, cursor, advance,
issue, changed, reset and check reservation). lifecycle command context supplies
the current revision; agent/worker instructions require using it and refreshing
after a conflict. keep old callers compatible when omitted. validate inside the
write transaction, not a racy pre-read. recording a running check's later result
must still succeed after a steer, but stale results cannot bless current proof.

extend check with optional `--component ID` for smoke/narrow/safety, never whole.
retain per-component input epochs so an unaffected scoped receipt still blocks
an unchanged repeat after another component changes. affected components and
their dependents advance epochs. legacy/unscoped receipts use global generation.
unscoped changed/observed code patches conservatively invalidate all input epochs
because the plugin cannot infer their semantic impact. keep that limit explicit.
do not permit an in-flight old whole check to bless a later revision.

UserPromptSubmit tells the existing agent to read the pinned plan and reconcile
the latest instruction before continuing. questions/no scope changes need no
revision. steers use revise, not reset; reset remains for a genuinely new task.
all lifecycle context remains bounded with command/session/plan/revision before
long user-authored fields. native five-hook topology and trust ownership stay.

assembly: runtime/commands/Markdown + hook/skill instructions + packaging/docs;
compile smoke only until joined. integrated proof then covers real CLI plan and
Markdown, mid-build additions/removals, dependency invalidation, layer changes,
completion reopening, no-op bytes/mtime/events, concurrent stale revisions,
scoped receipt preservation, old-state migration, publication recovery, hook
context and the existing lifecycle suite. then release CI, checksum-verified
upgrade, actual native five-hook trust and unrelated-config preservation.

## authority

repository: https://github.com/lmtlssss/The-GraphFather

approved description:
“i’m gonna make him an offer he can’t refuse.” — for Codex agents with commitment issues.

the person's requirement: every Codex conversation is a project. complete the
entire scaffold, then each ordered layer across the entire project; test the
joined system, then make narrow repairs. preserve an exact cursor and completed
work. ordinary progress must not depend on repeated recap, indexing or test loops.

manual style follows the actual READMEs and layouts of CompactVeteran,
RecentlyDivorced and The-Last-AirBlender: lowercase prose, correct brand casing,
plain function, small technical schematic, operational sections.

## v0.1 scope

a Rust command/hook binary, local SQLite state, a bundled instruction skill,
Codex lifecycle/tool hooks, install/uninstall scripts, CI and one system proof.
Linux x86_64 release first, matching the existing Rust plugins. no model/network
calls in routine operation. no transcript reading, launcher replacement, auto Git
commits, daemon, dashboard, fabricated config keys or changes to other plugins.

the plugin enforces its state transitions and recognized check commands. hooks
are guardrails, not a security boundary or proof that an agent's semantic claims
are true. hosted tools/specialized opt-outs and arbitrary shell programs are not
fully observable. document these limits instead of promising absolute prevention.

## layout

```
.agents/plugins/marketplace.json
.github/workflows/{ci,release}.yml
README.md  LICENSE  install.sh  uninstall.sh
scripts/prove-system.sh
plugins/the-graphfather/
  .codex-plugin/plugin.json
  hooks/hooks.json
  skills/build/SKILL.md
  runtime/Cargo.toml  runtime/Cargo.lock
  runtime/src/{main,state,hooks}.rs
```

marketplace and plugin identifier: the-graphfather. executable: the-graphfather.
installed executable/data: ${CODEX_HOME}/plugins/data/the-graphfather-the-graphfather.
hooks use quoted ${PLUGIN_DATA}/the-graphfather, never the Desktop source path.

## state

one record per conversation session ID, shared by its subagents. hook events
supply session_id; CLI takes --session ID or CODEX_THREAD_ID. never merge unrelated
conversations merely because cwd matches. SQLite transactions serialize updates.
use schema version 1, busy timeout, private data directory/file permissions.
--data-dir or GRAPHFATHER_DATA overrides PLUGIN_DATA/default for isolated tests.
do not store full prompts/tool output/secrets; only bounded explicit project data,
command receipt metadata and append-only state events. cap stdin at 128 KiB.

blueprint JSON:
```
{"objective":"build the thing","components":["state","cli","hooks"],
 "layers":["scaffold","behavior","recovery"],"next":"wire all interfaces"}
```
require nonempty objective, unique nonempty component/layer IDs, scaffold first,
bounded lengths/counts (64 components, 16 layers). marks bind layer+component to
nonempty evidence. phase = blueprint | build | proof | repair | complete.
cursor contains current layer, incomplete components, exact next action.
do not silently replace an existing blueprint. explicit reset requires a reason
and archives the old state in the event log before starting a new project.

## command interface

global options: --data-dir DIR, --session ID (before subcommand).

- status: JSON state (empty blueprint when not initialized).
- plan FILE: load blueprint, refuse replacement; creates build at scaffold.
- cursor TEXT: record exact next action without advancing phase.
- mark COMPONENT EVIDENCE: complete this component in CURRENT build layer;
  in repair, resolve only an explicitly opened issue for that component.
- advance: only when every component in current layer is marked; next layer or
  proof after the final layer. in repair, requires every opened issue resolved
  and returns to proof for final integrated confirmation.
- issue COMPONENT REASON: only from proof/repair; opens a narrow repair target,
  invalidates the previous successful whole-proof receipt, enters repair.
- check KIND LABEL -- COMMAND [ARGS...]: KIND smoke|whole|narrow|safety.
  execute argv directly, inherit normal process environment/cwd, capture exit
  status; store bounded metadata, not output. stream output to caller. smoke is
  allowed during build; whole only in proof; narrow only in repair with an open
  issue; safety needs a nonempty explicit reason via --reason before --.
  successful same-kind/label/argv check at same generation is not rerun: reject
  with the prior receipt and next cursor. failed checks may retry after a real
  repair/generation change; never loop unchanged failures. record both outcomes.
- changed REASON: explicit externally changed inputs; increments generation,
  invalidates successful proof, records reason. do not use to evade repeats.
- finish: only proof with successful whole check at current generation; complete.
- reset REASON: archive state; return to blueprint (explicit new task only).
- hook: bounded JSON stdin, dispatch by hook_event_name, JSON stdout.
- doctor: verify writable state plus schema, print concise checks.
- trust / untrust: use stock Codex app-server hooks/list + config/batchWrite to
  change ONLY hooks whose pluginId is the-graphfather@the-graphfather. use returned
  keys/currentHash, never guessed hashes. bounded timeout/owned-child cleanup.

state mutation commands return compact JSON including the cursor. unknown or
invalid input exits nonzero without partial changes. tests use disposable dirs.

## hooks

SessionStart, UserPromptSubmit, SubagentStart: concise project/cursor
context plus the installed binary path and session ID. initialize only an empty
record, never infer a blueprint from raw text. max ~1500 characters per injection.
small tasks use a small blueprint; never require the person to author it.

PreToolUse (all):
- observe Bash (including unified exec), apply_patch and Agent/spawn_agent.
- recognized direct test commands require check wrapper; explain exact command.
  distinguish argv execution from quoted search/text, and handle common env/cd,
  shell -c, npm/npx/cargo/pytest/vitest/playwright wrappers conservatively.
- never block check's child process through recursive hooks; own CLI validates
  kind/phase. allow configuration/status commands and normal discovery/build.
- block apply_patch in complete until explicit changed/reset; do not auto-reset.
- provide construction-phase reminders for delegation; no automatic model edits.

PostToolUse: successful apply_patch advances generation/invalidate prior proof.
do not persist patch bodies. unchanged reads/polls must not count as progress.
do not intercept compaction or interfere with CompactVeteran's existing hooks.

native wire contracts: PreToolUse deny is hookSpecificOutput.permissionDecision
with hookEventName=PreToolUse. other events use matching hookEventName and
additionalContext. PostToolUse cannot undo effects. unsupported input must never
emit an invalid hook response. error diagnostics stderr; no secret payload echo.

## operating instruction

whole-project layers supersede room-by-room finishing. record justified safety
exceptions narrowly; writing local payment code alone is not a blanket exemption
for exhaustive construction-time tests. first whole proof is diagnostic; retain
all failures, then repair affected parts and rerun the agreed whole proof once.
never replace proof with static strings that merely mirror implementation.

read the cursor and referenced changed evidence on resume, not the entire prior
conversation. refresh semantic indexes only when the next decision requires it,
not after docs/test-formatting commits. bounded worker briefs use fresh context
where supported. after two inadequate handoffs, change approach/executor within
allowed roles rather than continuing a tutoring loop. preserve user authority,
rollback, credentials, hardware trust and explicit delivery/physical-test gates.

## construction layers and acceptance

1. scaffold: state, CLI, hooks, packaging, skill, documentation all present.
2. behavior: transition rules, persistence, hook wiring and installation joined.
3. recovery: concurrent updates, invalid inputs, resume, repeat checks, safe
   uninstall and compatibility joined. compile/syntax smoke during these layers.
4. integrated proof: cargo tests + CLI/hook process scenarios + plugin validator
   + isolated install/uninstall + actual stock Codex hook consumption. observe
   blocking before an early test and continuity on resume. no live PartOfThis edits.
5. narrow repairs, one final system pass, commit and GitHub CI/release, install
   tested binary through plugin data path and trust only its hooks.

proof must cover two sessions in one cwd, whole-layer barriers, failed and passed
check receipts, generation invalidation, issue/repair/final proof, stdin bounds,
invalid JSON, concurrent state access, hook wire shape, no raw prompt retention,
install rollback and preservation of unrelated configuration/plugin state.
