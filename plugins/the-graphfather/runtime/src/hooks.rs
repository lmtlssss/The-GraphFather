use crate::state::Store;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    io::{self, Read},
    path::Path,
};
pub fn run(data: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let mut body = Vec::new();
    io::stdin().take(131073).read_to_end(&mut body)?;
    if body.len() > 131072 {
        return Err("hook input exceeds 128 KiB".into());
    }
    let input: Value = serde_json::from_slice(&body)?;
    let event = input
        .get("hook_event_name")
        .and_then(Value::as_str)
        .ok_or("hook_event_name required")?;
    let id = input
        .get("session_id")
        .and_then(Value::as_str)
        .map(str::to_owned)
        .or_else(|| std::env::var("CODEX_THREAD_ID").ok())
        .ok_or("session_id required")?;
    let mut store = Store::open(data, &id)?;
    if event == "SessionStart" {
        if let Some(origin) = handoff_origin() {
            store.register_handoff(&origin)?;
        }
    }
    let canonical_id = store.canonical_id().to_owned();
    let state = store.status()?;
    if event == "PreToolUse" {
        let tool = input.get("tool_name").and_then(Value::as_str).unwrap_or("");
        let command = input
            .pointer("/tool_input/command")
            .and_then(Value::as_str)
            .or_else(|| input.pointer("/tool_input/cmd").and_then(Value::as_str))
            .unwrap_or("");
        if is_recognized_test(command) && !contains_wrapper(command) {
            return deny(
                "recognized test command: wrap it with the-graphfather check KIND LABEL -- COMMAND",
            );
        };
        if tool == "apply_patch" && state["phase"] == "blueprint" {
            return deny("plan a blueprint before editing");
        };
        if tool == "apply_patch" && state["phase"] == "complete" {
            return deny("project is complete; use changed or reset first");
        };
        return Ok(());
    }
    if event == "PostToolUse"
        && input.get("tool_name").and_then(Value::as_str) == Some("apply_patch")
        && patch_succeeded(input.get("tool_response"))
        && !state["blueprint"].is_null()
    {
        let _ = store.changed("successful apply_patch".into())?;
        return Ok(());
    }
    let bp = &state["blueprint"];
    let objective = bp["objective"].as_str().unwrap_or("unplanned");
    let remaining = state
        .pointer("/cursor/incomplete_components")
        .and_then(Value::as_array)
        .map(|x| x.len())
        .unwrap_or(0);
    let exe = std::env::current_exe()
        .ok()
        .and_then(|p| p.into_os_string().into_string().ok())
        .unwrap_or_else(|| "the-graphfather".into());
    let skill = std::env::var("PLUGIN_ROOT")
        .map(|p| format!("{p}/skills/build/SKILL.md"))
        .unwrap_or_else(|_| "plugin skill: skills/build/SKILL.md".into());
    let plan = state
        .get("plan_path")
        .and_then(Value::as_str)
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            data.join("plans").join(format!(
                "{}.md",
                hex(&Sha256::digest(canonical_id.as_bytes()))
            ))
        });
    let revision = state["revision"].as_u64().unwrap_or(0);
    let prefix = format!(
        "{} --session {} --revision {}",
        shell_quote(&exe),
        shell_quote(&canonical_id),
        revision
    );
    let instruction = match event {
        "UserPromptSubmit" => " reconcile the latest user message with the pinned plan before continuing; use revise only for an actual steer, and do not reset for questions or no-op turns.",
        _ => " read the pinned plan before continuing; use the guarded revision for plan changes.",
    };
    let next = cut(
        state
            .pointer("/cursor/next")
            .and_then(Value::as_str)
            .unwrap_or("plan a blueprint"),
        240,
    );
    let objective = cut(objective, 240);
    let context = format!(
        "command={} | session={} | revision={} | plan={} | skill={} |{} phase={} layer={} remaining={} | next={} | objective={}",
        prefix, shell_quote(&canonical_id), revision, shell_quote(&plan.display().to_string()), shell_quote(&skill),
        instruction,
        state["phase"],
        state.pointer("/cursor/layer").unwrap_or(&Value::Null),
        remaining,
        next, objective
    );
    println!(
        "{}",
        json!({"hookSpecificOutput":{"hookEventName":event,"additionalContext":cut(&context,1500)}})
    );
    Ok(())
}
fn handoff_origin() -> Option<String> {
    let map = std::env::var("COMPACTVETERAN_HANDOFF_MAP").ok()?;
    let mut file = std::fs::File::open(map).ok()?;
    let mut bytes = Vec::new();
    file.take(16385).read_to_end(&mut bytes).ok()?;
    if bytes.len() > 16384 {
        return None;
    }
    let text = String::from_utf8(bytes).ok()?;
    let scope = text
        .split_once("## Scope\n")?
        .1
        .split_once("\n## Objective")?
        .0;
    scope
        .lines()
        .find_map(|l| {
            l.strip_prefix("- graphfather session: ")
                .map(str::trim)
                .filter(|x| !x.is_empty())
                .map(str::to_owned)
        })
        .or_else(|| {
            scope.lines().find_map(|l| {
                l.strip_prefix("- current session: ")
                    .map(str::trim)
                    .filter(|x| !x.is_empty())
                    .map(str::to_owned)
            })
        })
        .or_else(|| std::env::var("COMPACTVETERAN_GRAPHFATHER_SESSION").ok())
}
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
fn shell_quote(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''"))
}
fn deny(reason: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "{}",
        json!({"hookSpecificOutput":{"hookEventName":"PreToolUse","permissionDecision":"deny","permissionDecisionReason":reason}})
    );
    Ok(())
}
fn cut(s: &str, n: usize) -> String {
    if s.len() <= n {
        s.into()
    } else {
        s.chars().take(n).collect()
    }
}
fn contains_wrapper(s: &str) -> bool {
    s.split(|c: char| c.is_whitespace() || ";&|".contains(c))
        .any(|x| x.trim_matches('\"').ends_with("the-graphfather"))
}
fn patch_succeeded(v: Option<&Value>) -> bool {
    match v {
        Some(Value::Object(x)) => {
            x.get("success").and_then(Value::as_bool) == Some(true)
                || x.get("error").is_none()
                    && x.get("output")
                        .and_then(Value::as_str)
                        .map(|s| s.to_ascii_lowercase().contains("success"))
                        .unwrap_or(false)
        }
        Some(Value::String(s)) => s.to_ascii_lowercase().contains("success"),
        _ => false,
    }
}
fn is_recognized_test(s: &str) -> bool {
    recognized_tokens(&shell_tokens(s))
}
pub fn recognized_argv(a: &[String]) -> bool {
    recognized_tokens(&a.iter().map(String::as_str).collect::<Vec<_>>())
}
fn shell_tokens(s: &str) -> Vec<&str> {
    s.split(|c: char| c.is_whitespace() || ";&|".contains(c))
        .filter(|x| !x.is_empty())
        .map(|x| x.trim_matches(|c| c == '\'' || c == '\"'))
        .collect()
}
fn recognized_tokens(t: &[&str]) -> bool {
    let mut i = 0;
    while i < t.len() {
        if t[i].contains('=') && !t[i].starts_with('-') {
            i += 1;
            continue;
        }
        if t[i] == "cd" {
            i += 2;
            continue;
        }
        break;
    }
    let x = &t[i..];
    if x.first()
        .map(|x| matches!(*x, "echo" | "rg" | "grep" | "printf"))
        .unwrap_or(true)
    {
        return false;
    }
    if x.len() >= 2 && x[0] == "cargo" && x[1] == "test" {
        return true;
    }
    if x.len() >= 2 && x[0] == "npm" && x[1] == "test" {
        return true;
    }
    if x.len() >= 3 && x[0] == "npm" && x[1] == "run" && x[2].starts_with("test") {
        return true;
    }
    if x.first()
        .map(|x| matches!(*x, "pytest" | "vitest" | "playwright"))
        .unwrap_or(false)
    {
        return true;
    }
    if x.len() >= 2 && x[0] == "npx" && matches!(x[1], "vitest" | "playwright" | "pytest") {
        return true;
    }
    false
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn recognizes_real_tests_not_search() {
        assert!(is_recognized_test("cd x && cargo test"));
        assert!(is_recognized_test("npx vitest run"));
        assert!(!is_recognized_test("rg 'cargo test' README.md"));
        assert!(is_recognized_test("cd app && npm run test:mvp"));
        assert!(!is_recognized_test("echo cargo test"));
    }
}
