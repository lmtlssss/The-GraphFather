#!/usr/bin/env python3
"""Small real-process proof for the private living-plan projection."""
import hashlib, json, os, pathlib, shutil, sqlite3, subprocess, tempfile, time

ROOT = pathlib.Path(__file__).resolve().parents[1]
BIN = pathlib.Path(os.environ["GRAPHFATHER_BIN"]).resolve()

def run(data, session, *args, ok=True):
    p = subprocess.run([str(BIN), "--data-dir", str(data), "--session", session, *args], text=True, capture_output=True)
    if (p.returncode == 0) != ok: raise AssertionError((args, p.returncode, p.stdout, p.stderr))
    return json.loads(p.stdout) if p.stdout.strip() else None

def main():
  with tempfile.TemporaryDirectory() as td:
    data=pathlib.Path(td); plan=data/"input.json"
    base={"objective":"build the app","components":["api","ui"],"layers":["scaffold","behavior"],"next":"wire api","dependencies":{"ui":["api"]}}
    plan.write_text(json.dumps(base)); a=run(data,"one","plan",str(plan)); b=run(data,"two","plan",str(plan))
    paths=[pathlib.Path(a["plan_path"]),pathlib.Path(b["plan_path"])]
    assert paths[0].is_absolute() and paths[0]!=paths[1] and paths[0].stat().st_mode&0o777==0o600 and paths[0].parent.stat().st_mode&0o777==0o700
    run(data,"one","mark","api","api done"); run(data,"one","mark","ui","ui done")
    rev=run(data,"one","status")["revision"]
    amendment={"expected_revision":rev,"reason":"add export","blueprint":{"objective":"build the app","components":["api","ui","export"],"layers":["scaffold","behavior"],"next":"scaffold export","dependencies":{"ui":["api"],"export":["api"]}},"invalidate":{"ui":"behavior"}}
    inp=data/"revise.json"; inp.write_text(json.dumps(amendment)); out=run(data,"one","revise",str(inp)); text=paths[0].read_text(); assert "add export" in text and "dependencies" in text and "api done" in text
    db=sqlite3.connect(data/"state.sqlite"); events=db.execute("select count(*) from events where session_id='one'").fetchone()[0]; db.close(); before=(paths[0].read_bytes(),paths[0].stat().st_mtime_ns,run(data,"one","status")); run(data,"one","revise",str(inp),ok=False) # stale is rejected
    assert paths[0].read_bytes()==before[0]
    noop={"expected_revision":before[2]["revision"],"reason":"same","blueprint":amendment["blueprint"]}; ni=data/"noop.json"; ni.write_text(json.dumps(noop)); run(data,"one","revise",str(ni)); assert paths[0].stat().st_mtime_ns==before[1]; db=sqlite3.connect(data/"state.sqlite"); assert db.execute("select count(*) from events where session_id='one'").fetchone()[0]==events; db.close()
    current=run(data,"one","status")["revision"]; concurrent={**noop,"expected_revision":current,"invalidate":{"api":"scaffold"}}
    for i in (1,2): (data/f"c{i}.json").write_text(json.dumps({**concurrent,"reason":f"winner {i}"}))
    ps=[subprocess.Popen([str(BIN),"--data-dir",str(data),"--session","one","revise",str(data/f"c{i}.json")],stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True) for i in (1,2)]; [p.communicate() for p in ps]; assert sum(p.returncode==0 for p in ps)==1
    paths[0].unlink(); recovered=run(data,"one","status"); assert pathlib.Path(recovered["plan_path"]).exists()
    hook=subprocess.run([str(BIN),"--data-dir",str(data),"hook"],input=json.dumps({"hook_event_name":"UserPromptSubmit","session_id":"one","user_prompt":"?"}),text=True,capture_output=True); assert hook.returncode==0 and len(hook.stdout)<=1500 and "reconcile" in hook.stdout and "--revision" in hook.stdout
    huge=json.dumps({"hook_event_name":"UserPromptSubmit","session_id":"one","user_prompt":"x"*140000}); bad=subprocess.run([str(BIN),"--data-dir",str(data),"hook"],input=huge,text=True,capture_output=True); assert bad.returncode==0 and bad.stdout==""
    print("publication: ok (paths, permissions, projection, stale/no-op protection, recovery, bounded hook, oversize input)")
if __name__=="__main__": main()
