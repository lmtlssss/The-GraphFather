#!/usr/bin/env python3
"""Small subprocess proof for v0.2 receipt and revision boundaries."""
import json, os, subprocess, tempfile, time
from pathlib import Path

BIN = os.environ.get("GRAPHFATHER_BIN")
if not BIN: raise SystemExit("GRAPHFATHER_BIN is required")
def run(data, *args, ok=True):
    p=subprocess.run([BIN,"--data-dir",str(data),"--session","proof",*args],text=True,capture_output=True)
    if (p.returncode==0)!=ok: raise AssertionError((args,p.returncode,p.stdout,p.stderr))
    lines=[x for x in p.stdout.splitlines() if x.strip()]
    return json.loads(lines[-1]) if lines else None
with tempfile.TemporaryDirectory() as td:
    d=Path(td); plan=d/"p.json"; plan.write_text(json.dumps({"objective":"proof","components":["a","b"],"layers":["scaffold"],"next":"done"}))
    s=run(d,"plan",str(plan)); rev=s["revision"]
    run(d,"mark","a","ok"); run(d,"mark","b","ok"); run(d,"advance")
    # whole scoped checks are rejected
    run(d,"check","whole","x","--component","a","--","true",ok=False)
    # stale guarded writes are rejected
    p=subprocess.run([BIN,"--data-dir",str(d),"--session","proof","--revision",str(rev+1),"mark","a","again"],text=True,capture_output=True)
    if p.returncode==0: raise AssertionError("stale guarded mark unexpectedly succeeded")
with tempfile.TemporaryDirectory() as td:
    d=Path(td); plan=d/"p.json"; plan.write_text(json.dumps({"objective":"proof","components":["a","b"],"layers":["scaffold","behavior"],"next":"next"}))
    run(d,"plan",str(plan)); run(d,"check","smoke","a","--component","a","--","true"); run(d,"check","smoke","b","--component","b","--","true")
    req=d/"r.json"; req.write_text(json.dumps({"expected_revision":1,"reason":"change","blueprint":json.loads(plan.read_text()),"invalidate":{"b":"behavior"}})); run(d,"revise",str(req))
    run(d,"check","smoke","a2","--component","a","--","true",ok=False)
    run(d,"check","smoke","b2","--component","b","--","true")
with tempfile.TemporaryDirectory() as td:
    d=Path(td); plan=d/"p.json"; plan.write_text(json.dumps({"objective":"proof","components":["a","b"],"layers":["scaffold"],"next":"next"}))
    run(d,"plan",str(plan)); run(d,"check","smoke","a","--component","a","--","true"); run(d,"changed","real external edit"); run(d,"check","smoke","a2","--component","a","--","true")
def race(current_first):
    with tempfile.TemporaryDirectory() as td:
        d=Path(td); plan=d/"p.json"; plan.write_text(json.dumps({"objective":"proof","components":["a"],"layers":["scaffold"],"next":"next"})); run(d,"plan",str(plan)); run(d,"mark","a","ok"); run(d,"advance")
        started=d/"started"; release=d/"release"; argv=[BIN,"--data-dir",str(d),"--session","proof","--revision","1","check","whole","race","--","sh","-c","touch \"$1\"; while [ ! -e \"$2\" ]; do sleep 0.02; done","sh",str(started),str(release)]
        p=subprocess.Popen(argv,text=True,stdout=subprocess.PIPE,stderr=subprocess.PIPE)
        for _ in range(250):
            if started.exists(): break
            time.sleep(.02)
        amended=json.loads(plan.read_text()); amended["objective"]="steered"; req=d/"r.json"; req.write_text(json.dumps({"expected_revision":1,"reason":"steer","blueprint":amended})); run(d,"revise",str(req))
        if current_first: run(d,"check","whole","current","--","true")
        release.touch(); out=p.communicate(timeout=5)
        if p.returncode != 0: raise AssertionError(out)
        st=run(d,"status");
        if current_first: run(d,"finish")
        else: run(d,"finish",ok=False)
race(False); race(True)
print("receipt proof: basic subprocess gates passed")
