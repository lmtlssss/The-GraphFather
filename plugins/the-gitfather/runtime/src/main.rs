mod hooks;
mod state;
mod trust;

use state::Store;
use std::{env, path::PathBuf, process::exit};

fn main() {
    if let Err(e) = run() {
        eprintln!("the-gitfather: {e}");
        exit(2);
    }
}
fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut a: Vec<String> = env::args().skip(1).collect();
    let mut data = env::var_os("GITFATHER_DATA")
        .map(PathBuf::from)
        .or_else(|| env::var_os("PLUGIN_DATA").map(PathBuf::from))
        .or_else(|| {
            env::var_os("CODEX_HOME")
                .map(|p| PathBuf::from(p).join("plugins/data/the-gitfather-the-gitfather"))
        })
        .ok_or("PLUGIN_DATA, GITFATHER_DATA, or CODEX_HOME is required")?;
    let mut session = env::var("CODEX_THREAD_ID").ok();
    while !a.is_empty() && (a[0] == "--data-dir" || a[0] == "--session") {
        let k = a.remove(0);
        let v = a.first().ok_or("missing option value")?.clone();
        a.remove(0);
        if k == "--data-dir" {
            data = PathBuf::from(v)
        } else {
            session = Some(v)
        }
    }
    let cmd = a.first().map(String::as_str).unwrap_or("status");
    if cmd == "hook" {
        return hooks::run(&data);
    }
    if cmd == "trust" {
        trust::set(true)?;
        println!("{}", serde_json::json!({"ok":true}));
        return Ok(());
    }
    if cmd == "untrust" {
        trust::set(false)?;
        println!("{}", serde_json::json!({"ok":true}));
        return Ok(());
    }
    if cmd == "doctor" {
        return Store::open(&data, session.as_deref().unwrap_or("doctor"))?
            .doctor()
            .map_err(Into::into);
    }
    let session = session.ok_or("--session or CODEX_THREAD_ID is required")?;
    if cmd == "doctor" {
        return Store::open(&data, &session)?.doctor().map_err(Into::into);
    }
    let mut s = Store::open(&data, &session)?;
    let out = match cmd {
        "status" => s.status()?,
        "plan" => s.plan(a.get(1).ok_or("plan requires FILE")?)?,
        "cursor" => s.cursor(a[1..].join(" "))?,
        "mark" => s.mark(a.get(1).ok_or("component")?, a.get(2).ok_or("evidence")?)?,
        "advance" => s.advance()?,
        "issue" => s.issue(
            a.get(1).ok_or("component")?,
            a.get(2..).unwrap_or(&[]).join(" "),
        )?,
        "changed" => s.changed(a[1..].join(" "))?,
        "finish" => s.finish()?,
        "reset" => s.reset(a[1..].join(" "))?,
        "check" => s.check(&a[1..])?,
        _ => return Err("unknown command".into()),
    };
    println!("{}", serde_json::to_string(&out)?);
    Ok(())
}
