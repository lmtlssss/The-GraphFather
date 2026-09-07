use rusqlite::{params, Connection, OptionalExtension, TransactionBehavior};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, HashSet},
    fs,
    io::{self, Read},
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
    #[serde(default)]
    dependencies: BTreeMap<String, Vec<String>>,
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
    #[serde(default)]
    component: Option<String>,
    #[serde(default)]
    component_epoch: u64,
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
    #[serde(default)]
    revision: u64,
    #[serde(default)]
    input_epochs: BTreeMap<String, u64>,
}
#[derive(Deserialize)]
struct Revise {
    expected_revision: u64,
    reason: String,
    blueprint: Blueprint,
    #[serde(default)]
    invalidate: BTreeMap<String, String>,
}
pub struct Store {
    c: Connection,
    id: String,
    data: std::path::PathBuf,
    expected_revision: Option<u64>,
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
        c.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=FULL; CREATE TABLE IF NOT EXISTS graphfather_schema(version INTEGER NOT NULL); INSERT INTO graphfather_schema(version) SELECT 1 WHERE NOT EXISTS(SELECT 1 FROM graphfather_schema); CREATE TABLE IF NOT EXISTS sessions(id TEXT PRIMARY KEY, document TEXT NOT NULL); CREATE TABLE IF NOT EXISTS events(seq INTEGER PRIMARY KEY AUTOINCREMENT, session_id TEXT NOT NULL, at INTEGER NOT NULL, kind TEXT NOT NULL, data TEXT NOT NULL);")?;
        let v: i64 = c.query_row("SELECT version FROM graphfather_schema", [], |r| r.get(0))?;
        if v != SCHEMA as i64 {
            return Err("unsupported state schema".into());
        }
        Ok(Self {
            c,
            id: id.into(),
            data: fs::canonicalize(d)?,
            expected_revision: None,
        })
    }
    pub fn set_expected_revision(&mut self, revision: Option<u64>) {
        self.expected_revision = revision;
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
            revision: 0,
            input_epochs: BTreeMap::new(),
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
                Ok(Session {
                    revision: s.revision,
                    input_epochs: s.input_epochs,
                    ..s
                })
            }
        }
    }
    fn mutate<F>(&mut self, k: &str, data: Value, f: F) -> Result<Value, Box<dyn std::error::Error>>
    where
        F: FnOnce(&mut Session) -> Result<(), Box<dyn std::error::Error>>,
    {
        let empty = self.empty();
        let tx = self
            .c
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
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
        if k != "check_finished" {
            if let Some(expected) = self.expected_revision {
                if s.revision != expected {
                    return Err("stale revision".into());
                }
            }
        }
        let before = serde_json::to_string(&s)?;
        f(&mut s)?;
        if serde_json::to_string(&s)? == before {
            let path = publish_tx(&tx, &self.id, &self.data, &s)?;
            tx.commit()?;
            return Ok(with_path(view(&s), path));
        }
        tx.execute("INSERT INTO sessions(id,document)VALUES(?,?) ON CONFLICT(id)DO UPDATE SET document=excluded.document",params![self.id,serde_json::to_string(&s)?])?;
        tx.execute(
            "INSERT INTO events(session_id,at,kind,data)VALUES(?,?,?,?)",
            params![
                self.id,
                now(),
                k,
                serde_json::to_string(&if k == "revise" {
                    json!({"reason":data["reason"],"old":before})
                } else {
                    data
                })?
            ],
        )?;
        let path = publish_tx(&tx, &self.id, &self.data, &s)?;
        tx.commit()?;
        Ok(with_path(view(&s), path))
    }
    pub fn status(&mut self) -> Result<Value, Box<dyn std::error::Error>> {
        let empty = self.empty();
        let tx = self
            .c
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let raw: Option<String> = tx
            .query_row(
                "SELECT document FROM sessions WHERE id=?",
                [&self.id],
                |r| r.get(0),
            )
            .optional()?;
        let s = match raw {
            Some(x) => serde_json::from_str(&x)?,
            None => empty,
        };
        let path = publish_tx(&tx, &self.id, &self.data, &s)?;
        tx.commit()?;
        Ok(with_path(view(&s), path))
    }
    pub fn plan(&mut self, file: &str) -> Result<Value, Box<dyn std::error::Error>> {
        let bytes = if file == "-" {
            let mut x = Vec::new();
            io::stdin().take(131073).read_to_end(&mut x)?;
            if x.len() > 131072 {
                return Err("plan input exceeds 128 KiB".into());
            };
            x
        } else {
            fs::read(file)?
        };
        let b: Blueprint = serde_json::from_slice(&bytes)?;
        valid(&b)?;
        validate_dependencies(&b)?;
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
                s.revision = s.revision.saturating_add(1).max(1);
                Ok(())
            },
        )
    }
    pub fn revise(&mut self, file: &str) -> Result<Value, Box<dyn std::error::Error>> {
        let mut bytes = Vec::new();
        if file == "-" {
            io::stdin().take(131073).read_to_end(&mut bytes)?;
        } else {
            bytes = fs::read(file)?;
        }
        if bytes.len() > 131072 {
            return Err("revise input exceeds 128 KiB".into());
        }
        let req: Revise = serde_json::from_slice(&bytes)?;
        valid(&req.blueprint)?;
        validate_dependencies(&req.blueprint)?;
        bound(&req.reason, MAX, "reason")?;
        for (c, l) in &req.invalidate {
            component(&req.blueprint, c)?;
            if !req.blueprint.layers.iter().any(|x| x == l) {
                return Err("invalidate layer unknown".into());
            }
        }
        self.mutate("revise", json!({"reason":req.reason}), move |s| {
            if s.revision != req.expected_revision {
                return Err("stale revision".into());
            }
            let prior = s.blueprint.clone().ok_or("plan a blueprint first")?;
            if prior.objective == req.blueprint.objective
                && prior.components == req.blueprint.components
                && prior.layers == req.blueprint.layers
                && prior.next == req.blueprint.next
                && prior.dependencies == req.blueprint.dependencies
                && req.invalidate.is_empty()
            {
                return Ok(());
            }
            let next_only = prior.objective == req.blueprint.objective
                && prior.components == req.blueprint.components
                && prior.layers == req.blueprint.layers
                && prior.dependencies == req.blueprint.dependencies
                && req.invalidate.is_empty();
            s.revision = s.revision.saturating_add(1);
            s.blueprint = Some(req.blueprint.clone());
            if next_only {
                s.cursor.next = req.blueprint.next.clone();
                return Ok(());
            }
            s.marks
                .retain(|l, _| l == "repair" || req.blueprint.layers.contains(l));
            for m in s.marks.values_mut() {
                m.retain(|c, _| req.blueprint.components.contains(c));
            }
            let mut affected: BTreeMap<String, usize> = BTreeMap::new();
            for (c, l) in &req.invalidate {
                affected.insert(
                    c.clone(),
                    req.blueprint.layers.iter().position(|x| x == l).unwrap(),
                );
            }
            for c in req
                .blueprint
                .components
                .iter()
                .filter(|c| !prior.components.contains(c))
            {
                affected.entry(c.clone()).or_insert(0);
            }
            for c in prior
                .components
                .iter()
                .filter(|c| !req.blueprint.components.contains(c))
            {
                for d in dependent_closure(c, &prior, &req.blueprint) {
                    affected
                        .entry(d)
                        .and_modify(|x| *x = (*x).min(0))
                        .or_insert(0);
                }
            }
            for c in req.blueprint.components.iter() {
                if prior.dependencies.get(c) != req.blueprint.dependencies.get(c) {
                    affected
                        .entry(c.clone())
                        .and_modify(|x| *x = (*x).min(0))
                        .or_insert(0);
                }
            }
            let seeds: Vec<String> = affected.keys().cloned().collect();
            for c in seeds {
                let at = *affected.get(&c).unwrap();
                for d in dependent_closure(&c, &prior, &req.blueprint) {
                    let e = affected.entry(d).or_insert(at);
                    *e = (*e).min(at);
                }
            }
            for (c, at) in affected {
                for l in req.blueprint.layers.iter().skip(at) {
                    if let Some(m) = s.marks.get_mut(l) {
                        m.remove(&c);
                    }
                }
                if let Some(m) = s.marks.get_mut("repair") {
                    m.remove(&c);
                }
                *s.input_epochs.entry(c).or_insert(0) += 1;
            }
            s.issues
                .retain(|i| req.blueprint.components.contains(&i.component));
            s.generation += 1;
            s.proof_generation = None;
            let next = req
                .blueprint
                .layers
                .iter()
                .find(|l| {
                    req.blueprint
                        .components
                        .iter()
                        .any(|c| s.marks.get(*l).and_then(|m| m.get(c)).is_none())
                })
                .cloned();
            if let Some(l) = next {
                s.phase = "build".into();
                s.cursor = Cursor {
                    layer: Some(l.clone()),
                    incomplete_components: req
                        .blueprint
                        .components
                        .iter()
                        .filter(|c| s.marks.get(&l).and_then(|m| m.get(*c)).is_none())
                        .cloned()
                        .collect(),
                    next: req.blueprint.next.clone(),
                };
            } else if s.issues.iter().any(|i| !i.resolved) {
                s.phase = "repair".into();
                s.cursor.layer = None;
                s.cursor.incomplete_components = vec![];
                s.cursor.next = "resolve open repair issues".into();
            } else {
                s.phase = "proof".into();
                s.cursor.layer = None;
                s.cursor.incomplete_components = vec![];
                s.cursor.next = req.blueprint.next.clone();
            }
            Ok(())
        })
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
                if s.issues.iter().any(|i| !i.resolved) {
                    s.phase = "repair".into();
                    s.cursor.next = "resolve open repair issues".into();
                    return Ok(());
                }
                s.phase = "proof".into();
                s.cursor = Cursor {
                    layer: None,
                    incomplete_components: vec![],
                    next: "run a whole check".into(),
                }
            } else {
                let l = b.layers[n + 1].clone();
                let incomplete = b
                    .components
                    .iter()
                    .filter(|c| s.marks.get(&l).and_then(|m| m.get(*c)).is_none())
                    .cloned()
                    .collect::<Vec<_>>();
                s.cursor = Cursor {
                    layer: Some(l),
                    incomplete_components: incomplete,
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
            json!({"component":c,"reason":r,"reason_length":r.len()}),
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
        self.mutate(
            "changed",
            json!({"reason":r,"reason_length":r.len()}),
            |s| {
                project(s)?;
                s.generation += 1;
                let ids = s.blueprint.as_ref().map(|b| b.components.clone()).unwrap_or_default();
                for id in ids { *s.input_epochs.entry(id).or_insert(0) += 1; }
                s.proof_generation = None;
                if s.phase == "complete" {
                    s.phase = "proof".into();
                    s.cursor.next = "run a whole check after the declared change".into()
                }
                Ok(())
            },
        )
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
        let empty = self.empty();
        let tx = self
            .c
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let raw: Option<String> = tx
            .query_row(
                "SELECT document FROM sessions WHERE id=?",
                [&self.id],
                |x| x.get(0),
            )
            .optional()?;
        let old: Session = match raw {
            Some(x) => serde_json::from_str(&x)?,
            None => empty,
        };
        if let Some(expected) = self.expected_revision {
            if old.revision != expected {
                return Err("stale revision".into());
            }
        }
        let fresh = Session {
            schema: SCHEMA,
            session_id: self.id.clone(),
            phase: "blueprint".into(),
            generation: old.generation + 1,
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
            revision: old.revision.saturating_add(1),
            input_epochs: BTreeMap::new(),
        };
        tx.execute("INSERT INTO sessions(id,document)VALUES(?,?) ON CONFLICT(id)DO UPDATE SET document=excluded.document",params![self.id,serde_json::to_string(&fresh)?])?;
        tx.execute(
            "INSERT INTO events(session_id,at,kind,data)VALUES(?,?,?,?)",
            params![
                self.id,
                now(),
                "reset_archive",
                serde_json::to_string(&json!({"reason":r,"old":view(&old)}))?
            ],
        )?;
        let path = publish_tx(&tx, &self.id, &self.data, &fresh)?;
        tx.commit()?;
        Ok(with_path(view(&fresh), path))
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
        bound(label, 256, "label")?;
        let argv = &a[sep + 1..];
        if argv.is_empty() {
            return Err("command required".into());
        }
        let mut why = None;
        let mut scoped: Option<String> = None;
        let mut n = 2;
        while n < sep {
            if a[n] == "--component" {
                n += 1;
                scoped = Some(a.get(n).ok_or("--component requires ID")?.clone());
                n += 1;
                continue;
            }
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
        if kind == "whole" && scoped.is_some() {
            return Err("whole checks cannot be scoped".into());
        }
        if kind == "smoke" && crate::hooks::recognized_argv(argv) {
            return Err("recognized test command requires whole or narrow check kind".into());
        }
        let hash = hex(&Sha256::digest(serde_json::to_vec(argv)?));
        let r = self.mutate(
            "check_reserved",
            json!({"kind":kind,"label":label,"argv_sha256":hash,"reason":why}),
            |s| {
                project(s)?;
                if let Some(c) = &scoped {
                    component(s.blueprint.as_ref().unwrap(), c)?;
                }
                let epoch = scoped
                    .as_ref()
                    .map(|c| *s.input_epochs.get(c).unwrap_or(&0))
                    .unwrap_or(0);
                match kind.as_str() {
                    "smoke" if s.phase != "build" => return Err("smoke only during build".into()),
                    "whole" if s.phase != "proof" => return Err("whole only in proof".into()),
                    "narrow" if s.phase != "repair" || !s.issues.iter().any(|i| !i.resolved) => {
                        return Err("narrow needs an open repair issue".into())
                    }
                    _ => {}
                }
                if s.receipts.iter().any(|x| {
                    x.kind == *kind
                        && x.argv_sha256 == hash
                        && x.component == scoped
                        && ((scoped.is_some() && x.component_epoch == epoch)
                            || (scoped.is_none() && x.generation == s.generation))
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
                    component: scoped.clone(),
                    component_epoch: epoch,
                });
                Ok(())
            },
        )?;
        let g = r["generation"].as_u64().unwrap();
        let epoch = scoped
            .as_ref()
            .map(|c| {
                r.pointer("/input_epochs")
                    .and_then(|x| x.get(c))
                    .and_then(Value::as_u64)
                    .unwrap_or(0)
            })
            .unwrap_or(0);
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
                            && x.component == scoped
                            && x.component_epoch == epoch
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
                    if s.generation == g { s.proof_generation = if ok { Some(g) } else { None }; }
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
            .query_row("SELECT version FROM graphfather_schema", [], |r| r.get(0))?;
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
fn publish_tx(
    c: &rusqlite::Transaction<'_>,
    id: &str,
    data: &Path,
    s: &Session,
) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    let mut q = c.prepare(
        "SELECT data FROM events WHERE session_id=? AND kind='revise' ORDER BY seq DESC LIMIT 8",
    )?;
    let rows = q.query_map([id], |r| r.get::<_, String>(0))?;
    let mut h = Vec::new();
    for x in rows {
        let v: Value = serde_json::from_str(&x?)?;
        let old = v
            .get("old")
            .and_then(Value::as_str)
            .and_then(|x| serde_json::from_str::<Value>(x).ok())
            .and_then(|x| x.get("revision").and_then(Value::as_u64))
            .unwrap_or(0);
        let reason = v
            .get("reason")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        h.push((old, reason));
    }
    Ok(crate::markdown::publish(data, &view(s), &h)?)
}
fn with_path(mut v: Value, p: std::path::PathBuf) -> Value {
    v["plan_path"] = json!(p);
    v
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
fn validate_dependencies(b: &Blueprint) -> Result<(), Box<dyn std::error::Error>> {
    for (c, ps) in &b.dependencies {
        component(b, c)?;
        let mut seen = HashSet::new();
        for p in ps {
            component(b, p)?;
            if p == c || !seen.insert(p) {
                return Err("invalid dependency edge".into());
            }
        }
    }
    fn visit(
        x: &str,
        b: &Blueprint,
        stack: &mut HashSet<String>,
        done: &mut HashSet<String>,
    ) -> bool {
        if done.contains(x) {
            return false;
        }
        if !stack.insert(x.into()) {
            return true;
        }
        for p in b.dependencies.get(x).into_iter().flatten() {
            if visit(p, b, stack, done) {
                return true;
            }
        }
        stack.remove(x);
        done.insert(x.into());
        false
    }
    let mut st = HashSet::new();
    let mut d = HashSet::new();
    for c in &b.components {
        if visit(c, b, &mut st, &mut d) {
            return Err("dependency cycle".into());
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
fn dependent_closure(seed: &str, old: &Blueprint, new: &Blueprint) -> Vec<String> {
    let mut out = Vec::new();
    let mut todo = vec![seed.to_string()];
    while let Some(x) = todo.pop() {
        for b in [old, new] {
            for (c, ds) in &b.dependencies {
                if ds.iter().any(|d| d == &x) && !out.contains(c) {
                    out.push(c.clone());
                    todo.push(c.clone());
                }
            }
        }
    }
    out
}
fn view(s: &Session) -> Value {
    json!({"schema":s.schema,"session_id":s.session_id,"phase":s.phase,"generation":s.generation,"revision":s.revision,"input_epochs":s.input_epochs,"blueprint":s.blueprint,"cursor":s.cursor,"marks":s.marks,"issues":s.issues,"receipts":s.receipts,"proof_generation":s.proof_generation})
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
