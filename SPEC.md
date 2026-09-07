# the gitfather — build contract

## authority

repository: https://github.com/lmtlssss/The-GitFather

approved description:
“i’m gonna make him an offer he can’t refuse.” — for codex agents with commitment issues.

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
plugins/the-gitfather/
  .codex-plugin/plugin.json
  hooks/hooks.json
  skills/build/SKILL.md
  runtime/Cargo.toml  runtime/Cargo.lock
  runtime/src/{main,state,hooks}.rs
```

marketplace and plugin identifier: the-gitfather. executable: the-gitfather.
installed executable/data: ${CODEX_HOME}/plugins/data/the-gitfather-the-gitfather.
hooks use quoted ${PLUGIN_DATA}/the-gitfather, never the Desktop source path.

## state

one record per conversation session ID, shared by its subagents. hook events
supply session_id; CLI takes --session ID or CODEX_THREAD_ID. never merge unrelated
conversations merely because cwd matches. SQLite transactions serialize updates.
use schema version 1, busy timeout, private data directory/file permissions.
--data-dir or GITFATHER_DATA overrides PLUGIN_DATA/default for isolated tests.
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
  change ONLY hooks whose pluginId is the-gitfather@the-gitfather. use returned
  keys/currentHash, never guessed hashes. bounded timeout/owned-child cleanup.

state mutation commands return compact JSON including the cursor. unknown or
invalid input exits nonzero without partial changes. tests use disposable dirs.

## hooks

SessionStart, UserPromptSubmit, PostCompact, SubagentStart: concise project/cursor
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
Stop: concise advisory cursor only, never force an infinite continuation loop.
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
