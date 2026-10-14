import subprocess, pathlib, os, hashlib, gzip, json, time, re
root=pathlib.Path.cwd(); evidence=root/'docs/evidence/prism-tolerance-inspection-v1-2026-10-14/embedding-reconciled'; evidence.mkdir(parents=True,exist_ok=True)
env=os.environ.copy();env['PATH']='/home/vmiguel/.cargo/bin:'+env['PATH'];env['CARGO_BUILD_JOBS']='2'
def manifest():
 paths=subprocess.check_output(['git','ls-files','-z','--','crates','Cargo.toml','Cargo.lock','rust-toolchain.toml','.cargo','clippy.toml','rustfmt.toml']).decode().split('\0')
 return {p:hashlib.sha256((root/p).read_bytes()).hexdigest() for p in sorted(paths) if p and (root/p).is_file()}
def save(label,value):
 (evidence/(label+'.json')).write_text(json.dumps(value,indent=2)+'\n')
def zipped(label,value):
 with gzip.GzipFile(filename=str(evidence/(label+'.gz')),mode='wb',mtime=0) as f:f.write(value)
before=manifest();zipped('source-before.json',json.dumps(before,sort_keys=True,indent=2).encode())
commands=[('focused',['cargo','+1.99.0','test','-p','rustycad-tkbo','--lib','sketch_region_prism::tolerance_inspection']),('fmt',['cargo','+1.99.0','fmt','--all','--','--check'])]
checks=[];logs={}
for label,command in commands:
 print('starting',label,flush=True);start=time.time(); result=subprocess.run(command,cwd=root,env=env,stdout=subprocess.PIPE,stderr=subprocess.STDOUT)
 logs[label]=result.stdout; zipped(label+'.log',result.stdout)
 checks.append({'label':label,'command':command,'exit_code':result.returncode,'seconds':time.time()-start,'log':label+'.log.gz'})
 save('checks',checks);print('finished',label,result.returncode,flush=True)
after=manifest();zipped('source-after.json',json.dumps(after,sort_keys=True,indent=2).encode());save('source-comparison',{'files':len(before),'identical':before==after,'changed':[p for p in set(before)|set(after) if before.get(p)!=after.get(p)]})
assert all(row['exit_code']==0 for row in checks);assert before==after
print('complete',len(before),'source files',flush=True)
