#!/usr/bin/env python3
import json, os, subprocess, tempfile, pathlib
b=os.environ['GRAPHFATHER_BIN']; d=tempfile.mkdtemp(); p=pathlib.Path(d); bp={'objective':'x','components':['a','b'],'layers':['scaffold','behavior'],'next':'next','dependencies':{'b':['a']}}
(p/'b.json').write_text(json.dumps(bp))
def run(s,*a,ok=True):
 r=subprocess.run([b,'--data-dir',d,'--session',s,*a],text=True,capture_output=True)
 if (r.returncode==0)!=ok: raise AssertionError((a,r.stderr,r.stdout))
 return json.loads(r.stdout) if r.stdout else {}
st=run('one','plan',str(p/'b.json'));assert st['revision']==1 and pathlib.Path(st['plan_path']).is_absolute()
for x in ('a','b'):run('one','mark',x,'e')
run('one','advance');
for x in ('a','b'):run('one','mark',x,'e')
run('one','advance'); st=run('one','status');assert st['phase']=='proof'
rev={'expected_revision':st['revision'],'reason':'add','blueprint':dict(bp,components=['a','b','c'],dependencies={'b':['a'],'c':['b']})}
(p/'r.json').write_text(json.dumps(rev));st=run('one','revise',str(p/'r.json'));assert st['phase']=='build' and 'c' in st['cursor']['incomplete_components']
bad=dict(rev,expected_revision=1);(p/'bad.json').write_text(json.dumps(bad));run('one','revise',str(p/'bad.json'),ok=False)
assert pathlib.Path(st['plan_path']).exists();print('steering proof passed')

# independent revisions: preserve old marks, reject malformed graph input, and retain repair.
def fresh(name, blueprint):
 f=p/(name+'.json');f.write_text(json.dumps(blueprint));return run(name,'plan',str(f))
base={'objective':'y','components':['a','b','c','d'],'layers':['scaffold','behavior'],'next':'go','dependencies':{'b':['a'],'c':['b']}}
fresh('graph',base)
for x in base['components']:run('graph','mark',x,'e')
st=run('graph','status');assert set(st['marks']['scaffold'])==set(base['components'])
def revise(s, old, reason, blueprint, invalidate={}):
 f=p/(s+'-r.json');f.write_text(json.dumps({'expected_revision':old['revision'],'reason':reason,'blueprint':blueprint,'invalidate':invalidate}));return run(s,'revise',str(f))
added=dict(base,components=base['components']+['e'],dependencies=dict(base['dependencies'],e=['d']))
st=revise('graph',st,'add',added);assert set(st['marks']['scaffold'])==set(base['components']) and 'e' in st['cursor']['incomplete_components']
st=revise('graph',st,'invalidate',added,{'a':'behavior'});assert 'a' not in st['marks'].get('behavior',{})
bad=dict(added,dependencies={'a':['missing']});before=run('graph','status');f=p/'unknown.json';f.write_text(json.dumps({'expected_revision':before['revision'],'reason':'bad','blueprint':bad}));run('graph','revise',str(f),ok=False);assert run('graph','status')['revision']==before['revision']
cyclic=dict(added,dependencies={'a':['b'],'b':['a']});f=p/'cycle.json';f.write_text(json.dumps({'expected_revision':before['revision'],'reason':'bad','blueprint':cyclic}));run('graph','revise',str(f),ok=False)
f=p/'unknown-invalidate.json';f.write_text(json.dumps({'expected_revision':before['revision'],'reason':'bad','blueprint':added,'invalidate':{'missing':'scaffold'}}));run('graph','revise',str(f),ok=False)
print('graph preservation/rejection: passed')

def complete(s, blueprint):
 fresh(s,blueprint)
 for layer in blueprint['layers']:
  for x in blueprint['components']:run(s,'mark',x,'e')
  run(s,'advance')
 return run(s,'status')
effects=complete('effects',base)
effects=revise('effects',effects,'invalidate populated layer',base,{'a':'behavior'})
assert set(effects['marks']['scaffold'])==set(base['components'])
assert set(effects['marks']['behavior'])=={'d'}
assert all(effects['input_epochs'].get(x,0)>0 for x in ('a','b','c')) and effects['input_epochs'].get('d',0)==0
assert effects['cursor']['layer']=='behavior' and effects['cursor']['incomplete_components']==['a','b','c']

struct=complete('struct',base)
edge=dict(base,dependencies={'b':['d'],'c':['b']})
struct=revise('struct',struct,'edge structural',edge,{'b':'behavior'})
assert set(struct['marks']['scaffold'])=={'a','d'}

done=complete('done',base);run('done','check','whole','whole','--','/bin/true');done=run('done','finish');g=done['generation'];proof=done['proof_generation']
nextbp=dict(base,next='different exact next');done=revise('done',done,'next only',nextbp);assert done['phase']=='complete' and done['generation']==g and done['proof_generation']==proof
removed=dict(nextbp,components=['a','b','c'],dependencies={'b':['a'],'c':['b']});done=revise('done',done,'remove d',removed);assert done['phase']=='proof' and all(set(done['marks'][l])=={'a','b','c'} for l in ('scaffold','behavior'))
inserted=dict(removed,layers=['scaffold','behavior','integration']);done=revise('done',done,'insert layer',inserted);assert done['phase']=='build' and done['cursor']['layer']=='integration' and done['cursor']['incomplete_components']==['a','b','c']
print('effects/structural/complete revision matrix: passed')

# reset is a revision boundary, not a return to revision one.
rst=fresh('reset',base);rst=revise('reset',rst,'semantic',dict(base,next='r'));rst=run('reset','reset','new project');assert rst['revision']>2
f=p/'reset-plan.json';f.write_text(json.dumps(base));rst=run('reset','plan',str(f));assert rst['revision']>2
run('reset','--revision','1','cursor','stale',ok=False)

# a surviving unresolved repair blocks proof even through an unrelated amendment.
repair=complete('repair',base);repair=run('repair','issue','a','fix a');assert repair['phase']=='repair'
unrelated=dict(base,objective='changed objective');repair=revise('repair',repair,'unrelated',unrelated);assert repair['phase']=='repair' and any(i['component']=='a' and not i['resolved'] for i in repair['issues'])
run('repair','advance',ok=False);run('repair','finish',ok=False);run('repair','mark','a','repair evidence');repair=run('repair','advance');assert repair['phase']=='proof'
print('reset monotonic and repair continuity: passed')
