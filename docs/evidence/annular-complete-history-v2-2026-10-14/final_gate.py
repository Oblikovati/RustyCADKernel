from pathlib import Path
import subprocess, os, json, hashlib, re, time
root=Path('/home/vmiguel/git/oblikovati-workspace/Martensite-prism-annular-history')
out=Path('/tmp/martensite-prism-annular-history')
env=os.environ.copy();env['PATH']='/home/vmiguel/.cargo/bin:'+env['PATH'];env['CARGO_BUILD_JOBS']='2';env['CARGO_TARGET_DIR']=str(root/'target')
os.chdir(root)
commands=[]
def sources():
 paths=subprocess.check_output(['git','ls-files','-z']).decode().split('\0')
 return {p:hashlib.sha256(Path(p).read_bytes()).hexdigest() for p in paths if p and (p.endswith(('.rs','.toml','.lock')) or p=='rust-toolchain')}
def record(name,command):
 start=time.monotonic()
 with (out/(name+'.log')).open('wb') as stream: status=subprocess.run(command,env=env,stdout=stream,stderr=subprocess.STDOUT).returncode
 commands.append({'name':name,'command':command,'exit':status,'seconds':time.monotonic()-start})
 (out/'final-commands.json').write_text(json.dumps(commands,indent=2)+'\n')
 print(name,status,round(commands[-1]['seconds'],2),flush=True)
 assert status==0,(name,status)
 return (out/(name+'.log')).read_bytes()
before=sources();(out/'final-source-before.json').write_text(json.dumps(before,indent=2)+'\n')
record('final-environment',['bash','-c','rustc +1.99.0 -vV; cargo +1.99.0 -V; uname -a; git rev-parse HEAD'])
record('final-all-targets',['cargo','+1.99.0','test','-p','rustycad-tkbo','--all-targets','--all-features','--locked','--','--test-threads=2'])
record('final-docs',['cargo','+1.99.0','test','-p','rustycad-tkbo','--doc','--all-features','--locked'])
record('final-clippy',['cargo','+1.99.0','clippy','-p','rustycad-tkbo','--all-targets','--all-features','--locked','--','-D','warnings'])
record('final-fmt',['cargo','+1.99.0','fmt','--all','--','--check'])
filters=['portable_region_certificate_process_receipt','complete_prism_history_replay_across_processes_and_threads','prism_copy_lineage_owned_bytes_replay_across_processes_and_threads','frozen_prism_owned_bytes_replay_across_threads_and_repeated_operations','sketch_region_prism::publication::replay_tests::publication_process_replay','joined_measure_process_replay','two_loop_owned_process_receipt','annular_frozen_join_owned_process_receipt']
def payloads(index,raw):
 if index==0:return set(re.findall(rb'\{"operation":"martensite.region-certificate"[^\n]*\}',raw))
 if index==6:return set(bytes.fromhex(x.decode()) for x in re.findall(rb'two-loop-region-domain-v2:([0-9a-f]+)',raw))
 return set(bytes.fromhex(x.decode()) for x in re.findall(rb'(?<![0-9a-f])[0-9a-f]{600,}(?![0-9a-f])',raw))
manifest={}
for run in range(2):
 for index,test in enumerate(filters):
  name='final-replay-'+str(run)+'-'+str(index)
  raw=record(name,['cargo','+1.99.0','test','-p','rustycad-tkbo','--all-features','--locked',test,'--','--test-threads=1','--nocapture'])
  actual=payloads(index,raw);expected=payloads(index,(out/('baseline-'+str(index)+'.log')).read_bytes());assert actual and actual==expected,(run,test,'actual byte inequality')
  manifest[name]={'test':test,'exact_baseline_bytes_equal':True,'payloads':[{'bytes':len(v),'sha256':hashlib.sha256(v).hexdigest()} for v in sorted(actual)]}
  (out/'final-replay.json').write_text(json.dumps(manifest,indent=2)+'\n')
  print('exact-byte baseline match',name,flush=True)
after=sources();(out/'final-source-after.json').write_text(json.dumps(after,indent=2)+'\n');assert before==after,'source changed during gate'
print('PASS source bookends',len(before),'files; replay checks',len(manifest),flush=True)
