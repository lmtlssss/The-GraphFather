---
name: build
description: Use the-gitfather to keep a bounded project cursor and prove lifecycle checks.
---

# build

the-gitfather treats each Codex conversation as one project. keep the cursor exact and advance layers only after every component is marked.

```text
blueprint -> build -> proof -> repair -> complete
```

use `plan FILE`, `cursor TEXT`, `mark COMPONENT EVIDENCE`, and `advance` for construction. wrap checks with `check KIND LABEL -- COMMAND`. use `issue COMPONENT REASON` only after a failed whole proof. `changed REASON` records an external change. `trust` and `untrust` modify only this plugin's hooks. `status` and `doctor` are safe at any time.

hooks are guardrails, not a security boundary; hosted tools, specialized opt-outs, and arbitrary shell work are not fully observable.
