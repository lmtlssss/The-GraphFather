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
    before=(paths[0].read_bytes(),paths[0].stat().st_mtime_ns,run(data,"one","status")); run(data,"one","revise",str(inp),ok=False) # stale is rejected
    assert paths[0].read_bytes()==before[0]
    paths[0].unlink(); recovered=run(data,"one","status"); assert pathlib.Path(recovered["plan_path"]).exists()
    hook=subprocess.run([str(BIN),"--data-dir",str(data),"hook"],input=json.dumps({"hook_event_name":"UserPromptSubmit","session_id":"one","user_prompt":"?"}),text=True,capture_output=True); assert hook.returncode==0 and len(hook.stdout)<=1500 and "reconcile" in hook.stdout and "--revision" in hook.stdout
    huge=json.dumps({"hook_event_name":"UserPromptSubmit","session_id":"one","user_prompt":"x"*140000}); bad=subprocess.run([str(BIN),"--data-dir",str(data),"hook"],input=huge,text=True,capture_output=True); assert bad.returncode==0 and bad.stdout==""
    print("publication: ok (paths, permissions, projection, stale/no-op protection, recovery, bounded hook, oversize input)")
if __name__=="__main__": main()
