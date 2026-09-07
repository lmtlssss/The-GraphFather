use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, HashSet},
    fs,
    path::Path,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};
const SCHEMA: u32 = 1;
const MAX: usize = 4096;
#[derive(Clone, Serialize, Deserialize)]
struct Blueprint {
    objective: String,
    components: Vec<String>,
    layers: Vec<String>,
    next: String,
}
#[derive(Clone, Serialize, Deserialize)]
struct Cursor {
    layer: Option<String>,
    incomplete_components: Vec<String>,
    next: String,
}
#[derive(Clone, Serialize, Deserialize)]
struct Issue {
    component: String,
    reason: String,
    resolved: bool,
}
#[derive(Clone, Serialize, Deserialize)]
struct Receipt {
    kind: String,
    label: String,
    argv_sha256: String,
    generation: u64,
    status: String,
    exit_code: Option<i32>,
}
#[derive(Clone, Serialize, Deserialize)]
struct Session {
    schema: u32,
    session_id: String,
    phase: String,
    generation: u64,
    blueprint: Option<Blueprint>,
    cursor: Cursor,
    marks: BTreeMap<String, BTreeMap<String, String>>,
    issues: Vec<Issue>,
    receipts: Vec<Receipt>,
    proof_generation: Option<u64>,
}
pub struct Store {
    c: Connection,
    id: String,
}
impl Store {
    pub fn open(d: &Path, id: &str) -> Result<Self, Box<dyn std::error::Error>> {
        if id.trim().is_empty() || id.len() > 256 {
            return Err("valid session_id required".into());
        }
        fs::create_dir_all(d)?;
        perm(d, 0o700)?;
        let p = d.join("state.sqlite");
        let c = Connection::open(&p)?;
        perm(&p, 0o600)?;
        c.busy_timeout(std::time::Duration::from_secs(5))?;
        c.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=FULL; CREATE TABLE IF NOT EXISTS gitfather_schema(version INTEGER NOT NULL); INSERT INTO gitfather_schema(version) SELECT 1 WHERE NOT EXISTS(SELECT 1 FROM gitfather_schema); CREATE TABLE IF NOT EXISTS sessions(id TEXT PRIMARY KEY, document TEXT NOT NULL); CREATE TABLE IF NOT EXISTS events(seq INTEGER PRIMARY KEY AUTOINCREMENT, session_id TEXT NOT NULL, at INTEGER NOT NULL, kind TEXT NOT NULL, data TEXT NOT NULL);")?;
        let v: i64 = c.query_row("SELECT version FROM gitfather_schema", [], |r| r.get(0))?;
        if v != SCHEMA as i64 {
            return Err("unsupported state schema".into());
        }
        Ok(Self { c, id: id.into() })
    }
    fn empty(&self) -> Session {
        Session {
            schema: SCHEMA,
            session_id: self.id.clone(),
            phase: "blueprint".into(),
            generation: 0,
            blueprint: None,
            cursor: Cursor {
                layer: None,
                incomplete_components: vec![],
                next: "plan a blueprint".into(),
            },
            marks: BTreeMap::new(),
            issues: vec![],
            receipts: vec![],
            proof_generation: None,
        }
    }
    fn load(&self) -> Result<Session, Box<dyn std::error::Error>> {
        let x: Option<String> = self
            .c
            .query_row(
                "SELECT document FROM sessions WHERE id=?",
                [&self.id],
                |r| r.get(0),
            )
            .optional()?;
        match x {
            None => Ok(self.empty()),
            Some(x) => {
                let s: Session = serde_json::from_str(&x)?;
                if s.schema != SCHEMA || s.session_id != self.id {
                    return Err("invalid session document".into());
                }
                Ok(s)
            }
        }
    }
    fn mutate<F>(&mut self, k: &str, data: Value, f: F) -> Result<Value, Box<dyn std::error::Error>>
    where
        F: FnOnce(&mut Session) -> Result<(), Box<dyn std::error::Error>>,
    {
        let empty = self.empty();
        let tx = self.c.transaction()?;
        let x: Option<String> = tx
            .query_row(
                "SELECT document FROM sessions WHERE id=?",
                [&self.id],
                |r| r.get(0),
            )
            .optional()?;
        let mut s = match x {
            Some(x) => serde_json::from_str(&x)?,
            None => empty,
        };
        if s.schema != SCHEMA || s.session_id != self.id {
            return Err("invalid session document".into());
        };
        f(&mut s)?;
        tx.execute("INSERT INTO sessions(id,document)VALUES(?,?) ON CONFLICT(id)DO UPDATE SET document=excluded.document",params![self.id,serde_json::to_string(&s)?])?;
        tx.execute(
            "INSERT INTO events(session_id,at,kind,data)VALUES(?,?,?,?)",
            params![self.id, now(), k, serde_json::to_string(&data)?],
        )?;
        tx.commit()?;
        Ok(view(&s))
    }
    pub fn status(&self) -> Result<Value, Box<dyn std::error::Error>> {
        Ok(view(&self.load()?))
    }
    pub fn plan(&mut self, file: &str) -> Result<Value, Box<dyn std::error::Error>> {
        let b: Blueprint = serde_json::from_slice(&fs::read(file)?)?;
        valid(&b)?;
        self.mutate(
            "plan",
            json!({"objective_length":b.objective.len()}),
            move |s| {
                if s.blueprint.is_some() {
                    return Err("blueprint already exists; use reset".into());
                }
                s.phase = "build".into();
                s.cursor = Cursor {
                    layer: Some(b.layers[0].clone()),
                    incomplete_components: b.components.clone(),
                    next: b.next.clone(),
                };
                s.blueprint = Some(b);
                Ok(())
            },
        )
    }
    pub fn cursor(&mut self, t: String) -> Result<Value, Box<dyn std::error::Error>> {
        bound(&t, MAX, "cursor")?;
        self.mutate("cursor", json!({"length":t.len()}), move |s| {
            project(s)?;
            s.cursor.next = t;
            Ok(())
        })
    }
    pub fn mark(&mut self, c: &str, e: &str) -> Result<Value, Box<dyn std::error::Error>> {
        bound(e, 8192, "evidence")?;
        let c: String = c.into();
        self.mutate(
            "mark",
            json!({"component":c,"evidence_length":e.len()}),
            move |s| {
                let b = project(s)?.clone();
                component(&b, &c)?;
                if s.phase == "build" {
                    let l = s.cursor.layer.clone().ok_or("current layer missing")?;
                    let m = s.marks.entry(l).or_default();
                    m.insert(c, e.into());
                    s.cursor.incomplete_components = b
                        .components
                        .iter()
                        .filter(|x| !m.contains_key(*x))
                        .cloned()
                        .collect()
                } else if s.phase == "repair" {
                    s.issues
                        .iter_mut()
                        .find(|x| x.component == c && !x.resolved)
                        .ok_or("no open repair issue for component")?
                        .resolved = true;
                    s.marks
                        .entry("repair".into())
                        .or_default()
                        .insert(c, e.into());
                } else {
                    return Err("mark only during build or repair".into());
                }
                Ok(())
            },
        )
    }
    pub fn advance(&mut self) -> Result<Value, Box<dyn std::error::Error>> {
        self.mutate("advance", json!({}), |s| {
            let b = project(s)?.clone();
            if s.phase == "repair" {
                if s.issues.iter().any(|x| !x.resolved) {
                    return Err("open repair issues remain".into());
                }
                s.phase = "proof".into();
                s.cursor.next = "run a whole check for final confirmation".into();
                return Ok(());
            }
            if s.phase != "build" {
                return Err("advance only during build or repair".into());
            }
            let l = s.cursor.layer.clone().ok_or("current layer missing")?;
            if b.components
                .iter()
                .any(|x| s.marks.get(&l).and_then(|m| m.get(x)).is_none())
            {
                return Err("current layer has incomplete components".into());
            }
            let n = b
                .layers
                .iter()
                .position(|x| x == &l)
                .ok_or("invalid layer")?;
            if n + 1 == b.layers.len() {
                s.phase = "proof".into();
                s.cursor = Cursor {
                    layer: None,
                    incomplete_components: vec![],
                    next: "run a whole check".into(),
                }
            } else {
                s.cursor = Cursor {
                    layer: Some(b.layers[n + 1].clone()),
                    incomplete_components: b.components.clone(),
                    next: b.next.clone(),
                }
            }
            Ok(())
        })
    }
    pub fn issue(&mut self, c: &str, r: String) -> Result<Value, Box<dyn std::error::Error>> {
        bound(&r, MAX, "reason")?;
        let c: String = c.into();
        self.mutate(
            "issue",
            json!({"component":c,"reason_length":r.len()}),
            move |s| {
                component(project(s)?, &c)?;
                if s.phase != "proof" && s.phase != "repair" {
                    return Err("issues only from proof or repair".into());
                }
                if s.issues.iter().any(|x| x.component == c && !x.resolved) {
                    return Err("component already has an open issue".into());
                }
                s.issues.push(Issue {
                    component: c,
                    reason: r,
                    resolved: false,
                });
                s.phase = "repair".into();
                s.proof_generation = None;
                Ok(())
            },
        )
    }
    pub fn changed(&mut self, r: String) -> Result<Value, Box<dyn std::error::Error>> {
        bound(&r, MAX, "reason")?;
        self.mutate("changed", json!({"reason_length":r.len()}), |s| {
            project(s)?;
            s.generation += 1;
            s.proof_generation = None;
            if s.phase == "complete" {
                s.phase = "proof".into();
                s.cursor.next = "run a whole check after the declared change".into()
            }
            Ok(())
        })
    }
    pub fn finish(&mut self) -> Result<Value, Box<dyn std::error::Error>> {
        self.mutate("finish", json!({}), |s| {
            if s.phase != "proof" || s.proof_generation != Some(s.generation) {
                return Err(
                    "finish requires a successful whole check at current generation".into(),
                );
            }
            s.phase = "complete".into();
            Ok(())
        })
    }
    pub fn reset(&mut self, r: String) -> Result<Value, Box<dyn std::error::Error>> {
        bound(&r, MAX, "reset reason")?;
        let id = self.id.clone();
        self.mutate("reset", json!({"reason_length":r.len()}), move |s| {
            let g = s.generation + 1;
            *s = Session {
                schema: SCHEMA,
                session_id: id.clone(),
                phase: "blueprint".into(),
                generation: 0,
                blueprint: None,
                cursor: Cursor {
                    layer: None,
                    incomplete_components: vec![],
                    next: "plan a blueprint".into(),
                },
                marks: BTreeMap::new(),
                issues: vec![],
                receipts: vec![],
                proof_generation: None,
            };
            s.generation = g;
            Ok(())
        })
    }
    pub fn check(&mut self, a: &[String]) -> Result<Value, Box<dyn std::error::Error>> {
        let sep = a
            .iter()
            .position(|x| x == "--")
            .ok_or("check requires --")?;
        if sep < 2 {
            return Err("check requires KIND LABEL".into());
        }
        let (kind, label) = (&a[0], &a[1]);
        let argv = &a[sep + 1..];
        if argv.is_empty() {
            return Err("command required".into());
        }
        let mut why = None;
        let mut n = 2;
        while n < sep {
            if a[n] != "--reason" {
                return Err("unknown check option".into());
            }
            n += 1;
            why = Some(a.get(n).ok_or("--reason requires text")?.clone());
            n += 1
        }
        if kind == "safety" {
            bound(why.as_deref().unwrap_or(""), MAX, "safety reason")?
        }
        if !["smoke", "whole", "narrow", "safety"].contains(&kind.as_str()) {
            return Err("invalid check kind".into());
        }
        if kind == "smoke" && test_argv(argv) {
            return Err("recognized test command requires whole or narrow check kind".into());
        }
        let hash = hex(&Sha256::digest(serde_json::to_vec(argv)?));
        let r = self.mutate(
            "check_reserved",
            json!({"kind":kind,"label":label,"argv_sha256":hash}),
            |s| {
                project(s)?;
                match kind.as_str() {
                    "smoke" if s.phase != "build" => return Err("smoke only during build".into()),
                    "whole" if s.phase != "proof" => return Err("whole only in proof".into()),
                    "narrow" if s.phase != "repair" || !s.issues.iter().any(|i| !i.resolved) => {
                        return Err("narrow needs an open repair issue".into())
                    }
                    _ => {}
                }
                if s.receipts.iter().any(|x| {
                    x.generation == s.generation && x.kind == *kind && x.argv_sha256 == hash
                }) {
                    return Err("same check already reserved at this generation".into());
                }
                s.receipts.push(Receipt {
                    kind: kind.clone(),
                    label: label.clone(),
                    argv_sha256: hash.clone(),
                    generation: s.generation,
                    status: "running".into(),
                    exit_code: None,
                });
                Ok(())
            },
        )?;
        let g = r["generation"].as_u64().unwrap();
        let z = Command::new(&argv[0]).args(&argv[1..]).status();
        let (ok, code, msg) = match z {
            Ok(x) => (x.success(), x.code().unwrap_or(1), None),
            Err(e) => (false, 127, Some(e.to_string())),
        };
        self.mutate(
            "check_finished",
            json!({"kind":kind,"label":label,"argv_sha256":hash,"success":ok,"exit_code":code}),
            |s| {
                let q = s
                    .receipts
                    .iter_mut()
                    .find(|x| {
                        x.generation == g
                            && x.kind == *kind
                            && x.argv_sha256 == hash
                            && x.status == "running"
                    })
                    .ok_or("running receipt lost")?;
                q.status = if ok {
                    "success".into()
                } else {
                    "failed".into()
                };
                q.exit_code = Some(code);
                if kind == "whole" {
                    s.proof_generation = if ok && s.generation == g {
                        Some(g)
                    } else {
                        None
                    }
                }
                Ok(())
            },
        )?;
        if !ok {
            return Err(msg
                .unwrap_or_else(|| format!("check failed with exit code {code}"))
                .into());
        }
        Ok(
            json!({"success":true,"generation":g,"kind":kind,"label":label,"cursor":self.status()?["cursor"]}),
        )
    }
    pub fn doctor(&self) -> Result<(), Box<dyn std::error::Error>> {
        let _: i64 = self
            .c
            .query_row("SELECT version FROM gitfather_schema", [], |r| r.get(0))?;
        println!("state      ok\nschema     1\nwritable   ok");
        Ok(())
    }
}
fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
fn hex(x: &[u8]) -> String {
    x.iter().map(|x| format!("{x:02x}")).collect()
}
fn bound(x: &str, n: usize, k: &str) -> Result<(), Box<dyn std::error::Error>> {
    if x.trim().is_empty() || x.len() > n {
        Err(format!("{k} must be nonempty and at most {n} bytes").into())
    } else {
        Ok(())
    }
}
fn valid(b: &Blueprint) -> Result<(), Box<dyn std::error::Error>> {
    bound(&b.objective, MAX, "objective")?;
    bound(&b.next, MAX, "next")?;
    if b.components.is_empty()
        || b.components.len() > 64
        || b.layers.is_empty()
        || b.layers.len() > 16
        || b.layers[0] != "scaffold"
    {
        return Err("blueprint requires scaffold-first, 1-64 components, and 1-16 layers".into());
    }
    for v in [&b.components, &b.layers] {
        let mut h = HashSet::new();
        for x in v {
            bound(x, 256, "blueprint id")?;
            if !h.insert(x) {
                return Err("component/layer ids must be unique".into());
            }
        }
    }
    Ok(())
}
fn project(s: &Session) -> Result<&Blueprint, Box<dyn std::error::Error>> {
    s.blueprint
        .as_ref()
        .ok_or_else(|| "plan a blueprint first".into())
}
fn component(b: &Blueprint, c: &str) -> Result<(), Box<dyn std::error::Error>> {
    if b.components.iter().any(|x| x == c) {
        Ok(())
    } else {
        Err("unknown component".into())
    }
}
fn test_argv(a: &[String]) -> bool {
    a.iter()
        .any(|x| matches!(x.as_str(), "test" | "pytest" | "vitest" | "playwright"))
}
fn view(s: &Session) -> Value {
    json!({"schema":s.schema,"session_id":s.session_id,"phase":s.phase,"generation":s.generation,"blueprint":s.blueprint,"cursor":s.cursor,"marks":s.marks,"issues":s.issues,"receipts":s.receipts,"proof_generation":s.proof_generation})
}
#[cfg(unix)]
fn perm(p: &Path, m: u32) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(p, fs::Permissions::from_mode(m))
}
#[cfg(not(unix))]
fn perm(_: &Path, _: u32) -> std::io::Result<()> {
    Ok(())
}
