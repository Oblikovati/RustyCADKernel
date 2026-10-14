from pathlib import Path
import subprocess,os,json,hashlib,time
root=Path.cwd();out=Path('/tmp/martensite-annular-frozen-domain');env=os.environ.copy();env['PATH']='/home/vmiguel/.cargo/bin:'+env['PATH'];env['CARGO_BUILD_JOBS']='2'
def sources():
 return {str(p):hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted(Path('crates/tkbo').rglob('*.rs'))}
before=sources();(out/'final-source-before.json').write_text(json.dumps(before,indent=2)+'\n')
commands=[('all-targets',['cargo','+1.99.0','test','-p','rustycad-tkbo','--all-targets','--all-features']),('docs',['cargo','+1.99.0','test','-p','rustycad-tkbo','--doc','--all-features']),('clippy',['cargo','+1.99.0','clippy','-p','rustycad-tkbo','--all-targets','--all-features','--','-D','warnings']),('fmt',['cargo','+1.99.0','fmt','--all','--','--check'])]
results=[]
for label,command in commands:
 start=time.monotonic()
 with (out/('final-'+label+'.log')).open('wb') as log:status=subprocess.run(command,env=env,stdout=log,stderr=subprocess.STDOUT).returncode
 results.append({'label':label,'command':command,'exit':status,'seconds':time.monotonic()-start});print(results[-1],flush=True)
 (out/'final-commands.json').write_text(json.dumps(results,indent=2)+'\n')
 if status:break
after=sources();(out/'final-source-after.json').write_text(json.dumps(after,indent=2)+'\n');assert before==after,'source changed during frozen gate'
