use crate::state::Store;
use serde_json::{json, Value};
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
    let state = store.status()?;
    if event == "PreToolUse" {
        let tool = input.get("tool_name").and_then(Value::as_str).unwrap_or("");
        let command = input
            .pointer("/tool_input/command")
            .and_then(Value::as_str)
            .unwrap_or("");
        if is_recognized_test(command) && !contains_wrapper(command) {
            return deny(
                "recognized test command: wrap it with the-gitfather check KIND LABEL -- COMMAND",
            );
        };
        if tool == "apply_patch" && state["phase"] == "complete" {
            return deny("project is complete; use changed or reset first");
        };
        return Ok(());
    }
    if event == "PostToolUse"
        && input.get("tool_name").and_then(Value::as_str) == Some("apply_patch")
        && input
            .pointer("/tool_output/success")
            .and_then(Value::as_bool)
            .unwrap_or(false)
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
    let context=format!("the-gitfather: {} | phase={} layer={} remaining={} | next={} | command=\"${{PLUGIN_DATA}}/the-gitfather\" --session {}",objective,state["phase"],state.pointer("/cursor/layer").unwrap_or(&Value::Null),remaining,state.pointer("/cursor/next").and_then(Value::as_str).unwrap_or("plan a blueprint"),id);
    println!(
        "{}",
        json!({"hookSpecificOutput":{"hookEventName":event,"additionalContext":cut(&context,1500)}})
    );
    Ok(())
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
        .any(|x| x == "the-gitfather")
}
fn is_recognized_test(s: &str) -> bool {
    let t: Vec<_> = s.split_whitespace().collect();
    if !t.iter().any(|x| {
        matches!(
            *x,
            "cargo" | "npm" | "npx" | "pytest" | "vitest" | "playwright"
        )
    }) {
        return false;
    };
    t.iter()
        .any(|x| matches!(*x, "test" | "pytest" | "vitest" | "playwright"))
        || s.contains("cargo test")
        || s.contains("npm test")
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn recognizes_real_tests_not_search() {
        assert!(is_recognized_test("cd x && cargo test"));
        assert!(is_recognized_test("npx vitest run"));
        assert!(!is_recognized_test("rg 'cargo test' README.md"));
    }
}
