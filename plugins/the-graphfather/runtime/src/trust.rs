use serde_json::{json, Value};
use std::{
    io::{self, BufRead, BufReader, Read, Write},
    process::{Command, Stdio},
};
pub fn set(enable: bool) -> io::Result<()> {
    let codex = std::env::var("CODEX_BIN").unwrap_or_else(|_| "codex".into());
    let mut c = Command::new("timeout")
        .args(["10s", &codex, "app-server", "--stdio"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()?;
    let mut i = c.stdin.take().ok_or_else(|| io::Error::other("stdin"))?;
    let mut o = BufReader::new(c.stdout.take().ok_or_else(|| io::Error::other("stdout"))?);
    let mut send = |v: Value| -> io::Result<()> {
        serde_json::to_writer(&mut i, &v).map_err(io::Error::other)?;
        i.write_all(b"\n")?;
        i.flush()
    };
    let mut read = |id: u64| -> io::Result<Value> {
        let mut l = String::new();
        loop {
            l.clear();
            o.by_ref().take(65537).read_line(&mut l)?;
            if l.len() > 65536 {
                return Err(io::Error::other("app-server response too large"));
            }
            if l.is_empty() {
                return Err(io::Error::other("app-server closed"));
            };
            let v: Value = serde_json::from_str(&l).map_err(io::Error::other)?;
            if v["id"].as_u64() == Some(id) {
                if v.get("error").is_some() {
                    return Err(io::Error::other(v["error"].to_string()));
                }
                return Ok(v);
            }
        }
    };
    send(
        json!({"id":1,"method":"initialize","params":{"clientInfo":{"name":"the-graphfather","version":"0.1.0"}}}),
    )?;
    read(1)?;
    send(json!({"method":"initialized","params":{}}))?;
    send(json!({"id":2,"method":"hooks/list","params":{"cwds":[]}}))?;
    let v = read(2)?;
    let hs = v["result"]["data"]
        .as_array()
        .and_then(|a| a.first())
        .and_then(|x| x["hooks"].as_array())
        .ok_or_else(|| io::Error::other("no hooks"))?;
    let mut es = Vec::new();
    for h in hs
        .iter()
        .filter(|h| h["pluginId"] == "the-graphfather@the-graphfather")
    {
        let k = h["key"].as_str().ok_or_else(|| io::Error::other("key"))?;
        let hash = h["currentHash"]
            .as_str()
            .ok_or_else(|| io::Error::other("hash"))?;
        es.push(json!({"keyPath":format!("hooks.state.\"{k}\""),"value":if enable{json!({"enabled":true,"trusted_hash":hash})}else{Value::Null},"mergeStrategy":"replace"}));
    }
    if es.is_empty() {
        return Err(io::Error::other("no owned hooks"));
    }
    send(
        json!({"id":3,"method":"config/batchWrite","params":{"edits":es,"reloadUserConfig":true}}),
    )?;
    read(3)?;
    let _ = c.kill();
    let _ = c.wait();
    Ok(())
}
