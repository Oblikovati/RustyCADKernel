"""Freeze the complete new behavior before structural cleanup; retain failures verbatim."""
import pathlib, subprocess, gzip, hashlib, json, re, os, sys, time
root=pathlib.Path(__file__).resolve().parents[3]; out=pathlib.Path(__file__).resolve().parent; phase=sys.argv[1]
env=os.environ.copy(); env['PATH']='/home/vmiguel/.cargo/bin:'+env['PATH']; env['CARGO_BUILD_JOBS']='2'
base=['cargo','+1.99.0','test','-p','rustycad-tkbo','--lib']
commands=[base+['exact_tolerance::annular','--','--test-threads=2'],base+['sketch_region_prism::exact_tolerance::annular::replay_tests::annular_exact_tolerance_replays_across_processes_threads_and_owned_input_drop','--','--exact','--nocapture','--test-threads=1'],['cargo','+1.99.0','clippy','-p','rustycad-tkbo','--all-targets','--all-features','--','-D','warnings']]
paths=subprocess.check_output(['git','ls-files','--cached','--others','--exclude-standard','crates','Cargo.toml','Cargo.lock','rust-toolchain.toml'],cwd=root,text=True).splitlines()
def manifest(): return {p:hashlib.sha256((root/p).read_bytes()).hexdigest() for p in sorted(set(paths)) if (root/p).is_file()}
before=manifest(); records=[]
for index,command in enumerate(commands):
 start=time.monotonic(); run=subprocess.run(command,cwd=root,env=env,stdout=subprocess.PIPE,stderr=subprocess.STDOUT)
 log=f'{phase}-{index}.log.gz'; (out/log).write_bytes(gzip.compress(run.stdout,mtime=0))
 records.append({'command':command,'exit':run.returncode,'seconds':time.monotonic()-start,'log':log,'results':re.findall(r'test result:.*',run.stdout.decode(errors='replace'))}); print(json.dumps(records[-1]),flush=True)
 if run.returncode: break
assert before==manifest()
(out/(phase+'-source.json.gz')).write_bytes(gzip.compress(json.dumps(before,sort_keys=True).encode(),mtime=0))
(out/(phase+'.json')).write_text(json.dumps({'head':subprocess.check_output(['git','rev-parse','HEAD'],cwd=root,text=True).strip(),'source_unchanged':True,'records':records},indent=2)+'\n')
sys.exit(records[-1]['exit'])
