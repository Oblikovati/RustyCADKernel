import pathlib, subprocess, hashlib, json, os, time, gzip, re
root=pathlib.Path.cwd(); evidence=root/'docs/evidence/two-loop-region-domain-v2-2026-10-14'; temp=pathlib.Path('/tmp/martensite-two-loop-region-domain')
env=os.environ.copy();env['PATH']='/home/vmiguel/.cargo/bin:'+env['PATH'];env['CARGO_BUILD_JOBS']='2'
paths=subprocess.check_output(['git','ls-files','crates','Cargo.toml','Cargo.lock','rust-toolchain.toml']).decode().splitlines()
def snapshot():return {p:hashlib.sha256((root/p).read_bytes()).hexdigest() for p in paths if (root/p).is_file()}
baseline=snapshot();(evidence/'final-source-bookend.json').write_text(json.dumps(baseline,indent=2)+'\n')
records=[]
commands=[('all-targets',['test','-p','rustycad-tkbo','--all-targets','--all-features','--','--test-threads=1']),('docs',['test','-p','rustycad-tkbo','--doc','--all-features']),('clippy',['clippy','-p','rustycad-tkbo','--all-targets','--all-features','--','-D','warnings']),('fmt',['fmt','--all','--check'])]
commands += [('replay-'+str(i),r['command'][2:]) for i,r in enumerate(json.loads((temp/'post-style-commands.json').read_text()))]
commands += [('v2-process-'+str(i),['test','-p','rustycad-tkbo','--all-features','two_loop_owned_process_receipt','--','--test-threads=1','--nocapture']) for i in range(2)]
for label,args in commands:
 assert snapshot()==baseline,'source changed before '+label
 command=['cargo','+1.99.0']+args; start=time.time(); log=temp/('final-'+label+'.log')
 with log.open('wb') as stream:status=subprocess.run(command,stdout=stream,stderr=subprocess.STDOUT,env=env).returncode
 stable=snapshot()==baseline; raw=log.read_bytes();(evidence/(log.name+'.gz')).write_bytes(gzip.compress(raw,mtime=0))
 records.append({'label':label,'command':command,'exit':status,'seconds':time.time()-start,'source_unchanged':stable,'results':re.findall(r'test result:.*',raw.decode(errors='replace'))})
 (evidence/'final-commands.json').write_text(json.dumps(records,indent=2)+'\n'); print(label,status,stable,flush=True)
 assert status==0 and stable,'gate failed '+label
print('FINAL GATES PASSED',flush=True)
