#!/usr/bin/env sh
set -eu
root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
bin="${GRAPHFATHER_BIN:-$root/plugins/the-graphfather/runtime/target/debug/the-graphfather}"
[ -x "$bin" ] || { echo "build first or set GRAPHFATHER_BIN" >&2; exit 1; }
ROOT="$root" BIN="$bin" python3 - <<'PY'
import json,os,subprocess,tempfile,threading,stat
root=os.environ['ROOT']; b=os.environ['BIN']; d=tempfile.mkdtemp()
clean={k:v for k,v in os.environ.items() if k not in ('PLUGIN_DATA','GRAPHFATHER_DATA','CODEX_HOME','CODEX_THREAD_ID')}
assert subprocess.run([b,'--version'],env=clean,text=True,capture_output=True).returncode==0
bp=os.path.join(d,'b.json');json.dump({'objective':'prove','components':['state','hooks'],'layers':['scaffold','behavior'],'next':'wire'},open(bp,'w'))
def run(session,*a,input=None,ok=True):
 p=subprocess.run([b,'--data-dir',d,'--session',session,*a],input=input,text=True,capture_output=True,env=clean)
 if (p.returncode==0)!=ok: raise AssertionError((a,p.returncode,p.stdout,p.stderr))
 return p
def proof(s):
 run(s,'plan',bp);run(s,'advance',ok=False)
 for layer in range(2):
  run(s,'mark','state','e');run(s,'mark','hooks','e');run(s,'advance')
hook=lambda x: subprocess.run([b,'--data-dir',d,'hook'],input=json.dumps(x),text=True,capture_output=True,env=clean)
proof('one'); assert json.loads(run('two','status').stdout)['blueprint'] is None
run('one','check','whole','bad','--','/bin/false',ok=False);st=json.loads(run('one','status').stdout);assert st['receipts'][-1]['status']=='failed'
run('one','check','whole','renamed','--','/bin/false',ok=False);run('one','changed','input');
run('one','check','whole','pass','--','/bin/true');run('one','issue','state','repair it');run('one','advance',ok=False);g=json.loads(run('one','status').stdout)['generation'];assert hook({'hook_event_name':'PostToolUse','session_id':'one','tool_name':'apply_patch','tool_response':{'success':True}}).returncode==0;assert json.loads(run('one','status').stdout)['generation']==g+1;run('one','mark','state','repair evidence');run('one','advance');run('one','check','whole','final','--','/bin/true');run('one','finish');run('one','changed','edit');run('one','check','whole','latebad','--','/bin/false',ok=False);run('one','finish',ok=False)
before=json.loads(run('one','status').stdout)['generation'];run('one','reset','new task');assert json.loads(run('one','status').stdout)['generation']>before
assert hook({'hook_event_name':'PreToolUse','session_id':'one','tool_name':'Bash','tool_input':{'command':'cargo test'}}).returncode==0
assert 'deny' in hook({'hook_event_name':'PreToolUse','session_id':'one','tool_name':'Bash','tool_input':{'command':'cargo test'}}).stdout
assert hook({'hook_event_name':'PreToolUse','session_id':'one','tool_name':'Bash','tool_input':{'command':"rg 'cargo test' README"}}).stdout==''
run('patch','plan',bp);g=json.loads(run('patch','status').stdout)['generation'];assert hook({'hook_event_name':'PostToolUse','session_id':'patch','tool_name':'apply_patch','tool_response':{'success':True}}).returncode==0;assert json.loads(run('patch','status').stdout)['generation']==g+1
assert stat.S_IMODE(os.stat(os.path.join(d,'state.sqlite')).st_mode)==0o600
sentinel='PRIVATE_ARGV_SENTINEL_9f6ec';run('patch','check','smoke','sentinel','--','/bin/echo',sentinel);assert sentinel.encode() not in open(os.path.join(d,'state.sqlite'),'rb').read()
print('system proof passed')
PY
GRAPHFATHER_BIN="$bin" python3 "$root/scripts/prove-steering.py"
GRAPHFATHER_BIN="$bin" python3 "$root/scripts/prove-receipts.py"
GRAPHFATHER_BIN="$bin" python3 "$root/scripts/prove-publication.py"
