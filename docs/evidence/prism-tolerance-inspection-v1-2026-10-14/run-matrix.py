import subprocess, pathlib, os, hashlib, gzip, json, time, re
root=pathlib.Path.cwd(); evidence=root/'docs/evidence/prism-tolerance-inspection-v1-2026-10-14'; evidence.mkdir(parents=True,exist_ok=True)
env=os.environ.copy();env['PATH']='/home/vmiguel/.cargo/bin:'+env['PATH'];env['CARGO_BUILD_JOBS']='2'
def manifest():
 paths=subprocess.check_output(['git','ls-files','-z','--','crates','Cargo.toml','Cargo.lock','rust-toolchain.toml','.cargo','clippy.toml','rustfmt.toml']).decode().split('\0')
 return {p:hashlib.sha256((root/p).read_bytes()).hexdigest() for p in sorted(paths) if p and (root/p).is_file()}
def save(label,value):
 (evidence/(label+'.json')).write_text(json.dumps(value,indent=2)+'\n')
def zipped(label,value):
 with gzip.GzipFile(filename=str(evidence/(label+'.gz')),mode='wb',mtime=0) as f:f.write(value)
before=manifest();zipped('source-before.json',json.dumps(before,sort_keys=True,indent=2).encode())
commands=[('all-targets',['cargo','+1.99.0','test','-p','rustycad-tkmath','-p','rustycad-tkbrep','-p','rustycad-tkbo','--all-targets','--all-features','--locked']),('doctests',['cargo','+1.99.0','test','-p','rustycad-tkmath','-p','rustycad-tkbrep','-p','rustycad-tkbo','--doc','--all-features','--locked']),('clippy',['cargo','+1.99.0','clippy','-p','rustycad-tkmath','-p','rustycad-tkbrep','-p','rustycad-tkbo','--all-targets','--all-features','--locked','--','-D','warnings']),('fmt',['cargo','+1.99.0','fmt','--all','--','--check'])]
for label in ['process-first','process-second']:
 commands.append((label,['cargo','+1.99.0','test','-p','rustycad-tkbo','--lib','sketch_region_prism::tolerance_inspection::replay_tests::tolerance_arithmetic_replays_across_repeated_operations_and_independent_workers','--','--exact','--nocapture','--test-threads=1']))
commands.append(('frozen-replay',['cargo','+1.99.0','test','-p','rustycad-tkbo','--lib','sketch_region_prism::frozen::replay_tests::frozen_prism_owned_bytes_replay_across_threads_and_repeated_operations','--','--exact','--nocapture','--test-threads=1']))
checks=[];logs={}
for label,command in commands:
 print('starting',label,flush=True);start=time.time(); result=subprocess.run(command,cwd=root,env=env,stdout=subprocess.PIPE,stderr=subprocess.STDOUT)
 logs[label]=result.stdout; zipped(label+'.log',result.stdout)
 checks.append({'label':label,'command':command,'exit_code':result.returncode,'seconds':time.time()-start,'log':label+'.log.gz'})
 save('checks',checks);print('finished',label,result.returncode,flush=True)
after=manifest();zipped('source-after.json',json.dumps(after,sort_keys=True,indent=2).encode());save('source-comparison',{'files':len(before),'identical':before==after,'changed':[p for p in set(before)|set(after) if before.get(p)!=after.get(p)]})
payload=[]
for label in ['process-first','process-second']:
 match=re.search(rb'prism-tolerance-arithmetic-v1:([0-9a-f]+)',logs[label]);payload.append(bytes.fromhex(match[1].decode()) if match else b'')
frozen=bytes.fromhex(re.search(rb'frozen-prism-v1:([0-9a-f]+)',logs['frozen-replay'])[1].decode())
save('process-replay',{'arithmetic_bytes':len(payload[0]),'equal':payload[0]==payload[1] and bool(payload[0]),'sha256':hashlib.sha256(payload[0]).hexdigest(),'frozen_bytes':len(frozen),'frozen_sha256':hashlib.sha256(frozen).hexdigest(),'frozen_matches_baseline':len(frozen)==89921 and hashlib.sha256(frozen).hexdigest()=='88e28942e1fc6ad301a90f5f07624962d842f0262bde1377103cd3470a871fb4'})
assert all(row['exit_code']==0 for row in checks);assert before==after;assert payload[0]==payload[1] and payload[0]
print('complete',len(before),'source files',flush=True)
