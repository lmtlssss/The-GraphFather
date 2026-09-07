#!/usr/bin/env python3
import json, os, pathlib, subprocess, tempfile
CV = os.environ["CV_BIN"]; GF = os.environ["GF_BIN"]
def call(argv, env, obj=None):
    p = subprocess.run(argv, input=json.dumps(obj).encode() if obj else None, env=env, stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=True)
    return p.stdout.decode()
with tempfile.TemporaryDirectory() as d:
    root=pathlib.Path(d); home=root/"home"; repo=root/"repo"; home.mkdir(); repo.mkdir()
    subprocess.run(["git","-C",str(repo),"init","-q"],check=True)
    subprocess.run(["git","-C",str(repo),"config","user.email","fixture@example.com"],check=True)
    subprocess.run(["git","-C",str(repo),"config","user.name","fixture"],check=True)
    (repo/"README.md").write_text("fixture\n"); subprocess.run(["git","-C",str(repo),"add","."],check=True); subprocess.run(["git","-C",str(repo),"commit","-qm","fixture"],check=True)
    env=os.environ.copy(); env.update(HOME=str(home),CODEX_HOME=str(home/".codex"),XDG_STATE_HOME=str(home/".local/state"))
    base={"session_id":"cv-origin","cwd":str(repo),"model":"gpt-5.6-sol","turn_id":"turn1"}
    call([CV,"hook","prompt"],env,dict(base,hook_event_name="UserPromptSubmit",prompt="fixture")); call([CV,"hook","stop"],env,dict(base,hook_event_name="Stop",last_assistant_message="fixture complete"))
    maps=list((home/".codex/project-maps").glob("*.md")); assert len(maps)==1, maps
    mapping=maps[0]; data=root/"gf"; gfenv=dict(env,GRAPHFATHER_DATA=str(data)); plan=root/"plan.json"
    plan.write_text(json.dumps({"objective":"fixture","components":["core"],"layers":["scaffold"],"next":"prove"}))
    call([GF,"--data-dir",str(data),"--session","cv-origin","plan",str(plan)],gfenv); call([GF,"--data-dir",str(data),"--session","cv-origin","mark","core","fixture"],gfenv)
    call([GF,"--data-dir",str(data),"--session","cv-origin","check","smoke","fixture","--","/usr/bin/true"],gfenv)
    baseline=json.loads(call([GF,"--data-dir",str(data),"--session","cv-origin","status"],gfenv))
    def hook(session, map_path=mapping):
        e=dict(gfenv,COMPACTVETERAN_HANDOFF_MAP=str(map_path))
        return call([GF,"--data-dir",str(data),"--session",session,"hook"],e,dict(hook_event_name="SessionStart",session_id=session,cwd=str(repo),model="gpt-5.6-sol"))
    hook("next-1"); a=json.loads(call([GF,"--data-dir",str(data),"--session","next-1","status"],gfenv)); assert a["session_id"]=="cv-origin" and a["plan_path"]==baseline["plan_path"] and a["revision"]==baseline["revision"] and a["marks"]==baseline["marks"] and a["receipts"]==baseline["receipts"]
    hook("next-1")
    legacy=root/"legacy.md"; legacy.write_text("## Scope" + chr(10) + "- current session: next-1" + chr(10) + chr(10) + "## Objective" + chr(10))
    hook("next-2",legacy); b=json.loads(call([GF,"--data-dir",str(data),"--session","next-2","status"],gfenv)); assert b["session_id"]=="cv-origin" and b["plan_path"]==baseline["plan_path"]
    u=json.loads(call([GF,"--data-dir",str(data),"--session","unrelated","status"],gfenv)); assert u["blueprint"] is None
    conflict_plan=root/"conflict.json"; conflict_plan.write_text(json.dumps({"objective":"conflict","components":["x"],"layers":["scaffold"],"next":"stop"}))
    call([GF,"--data-dir",str(data),"--session","conflict","plan",str(conflict_plan)],gfenv)
    before=json.loads(call([GF,"--data-dir",str(data),"--session","conflict","status"],gfenv)); hook("conflict"); after=json.loads(call([GF,"--data-dir",str(data),"--session","conflict","status"],gfenv)); assert before==after
    missing_map=root/"missing.md"; missing_map.write_text("## Scope" + chr(10) + "- current session: absent-origin" + chr(10) + chr(10) + "## Objective" + chr(10))
    hook("missing",missing_map); m=json.loads(call([GF,"--data-dir",str(data),"--session","missing","status"],gfenv)); assert m["blueprint"] is None
print("map-generation=passed alias-chain=passed ledger-preservation=passed conflict=passed independent-origin=passed")
