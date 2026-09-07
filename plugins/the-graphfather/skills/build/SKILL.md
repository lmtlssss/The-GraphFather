---
name: build
description: Maintain a Codex task's blueprint, whole-project layers, exact cursor and check receipts with The GraphFather.
---

# build

use the absolute command path and session ID supplied by the hook. do not assume
the binary is on PATH or PLUGIN_DATA exists in the agent's shell. subagents use
the parent session ID supplied by their hook.

## begin or resume

read `status` first. continue its cursor; never reset completed work because a
new turn or compaction occurred. derive a small blueprint from the actual request
and existing sources. the person does not author it.

`plan FILE` reads JSON; `plan -` accepts the same JSON on stdin:

```json
{"objective":"the requested outcome","components":["storage","interface"],"layers":["scaffold","behavior","recovery"],"next":"join the storage and interface boundaries"}
```

IDs must be unique and nonempty; scaffold comes first. maximum: 64 components,
16 layers. scale the plan to the task. a short answer or repair needs no large
construction plan. preserve an existing plan unless the person changes scope.

## construct

complete the entire scaffold, then each layer across all components. never take
one component through finishing while the others lack the foundation.

`mark COMPONENT EVIDENCE` records actual completed work in the current layer;
`cursor TEXT` saves the precise next action. evidence identifies files, commits
or observed results. `advance` refuses an incomplete layer.

construction allows compile/type/build smoke, not exhaustive tests or polish
detours. use `check smoke LABEL -- COMMAND`. a named imminent safety effect uses
`check safety LABEL --reason REASON -- COMMAND`; merely editing unreleased local
payment code does not justify exhaustive construction-time tests.

## prove and repair

after all layers, run `check whole LABEL -- COMMAND`. retain the failures and
group them by cause. use `issue COMPONENT REASON`, repair that component, then
`check narrow LABEL -- COMMAND`. mark resolved work and advance to final whole
proof. `finish` requires successful current-generation whole proof. honor the
person's explicit provider, device and release gates.

do not rerun unchanged checks under new labels. successful observed patches
invalidate prior proof; `changed REASON` records real external changes the hooks
could not observe. filing an issue is not an input change.

## continuity

read only necessary cursor evidence on resume. refresh a semantic index when the
next decision needs it, not after every commit. delegate a bounded part of the
current layer with fresh context where supported. after two inadequate handoffs,
change approach within allowed roles rather than continuing a tutoring loop.

`reset REASON` archives the old state for an explicit new task. do not use reset
or changed to evade guards. never put credentials or raw prompts in project fields.

hooks guard supported calls and recognized commands, not every arbitrary program
or the truth of an agent's completion claim. routine operation stays local.
