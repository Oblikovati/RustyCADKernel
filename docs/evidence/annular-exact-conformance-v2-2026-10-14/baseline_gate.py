import pathlib,subprocess,json,gzip,hashlib,os,re,time,sys
root=pathlib.Path(__file__).resolve().parents[3];out=pathlib.Path(__file__).resolve().parent;phase=sys.argv[1]
env=os.environ.copy();env['PATH']='/home/vmiguel/.cargo/bin:'+env['PATH'];env['CARGO_BUILD_JOBS']='2'
old=json.loads((root/'docs/evidence/annular-prism-binding-v2-2026-10-14/final-exact-first.json').read_text())
commands=[row['command'] for row in old['commands']]
base=['cargo','+1.99.0','test','-p','rustycad-tkbo','--lib']
for name in ['sketch_region_prism::exact_tolerance::joined_replay_tests::exact_tolerance_bytes_replay_across_threads_and_operations','sketch_region_prism::frozen::annular_join::replay_tests::annular_frozen_join_owned_process_receipt','sketch_region_prism::exact_tolerance::joined_cancel_tests::exact_conformance_callback_trace_baseline']:
 commands.append(base+[name,'--','--nocapture','--test-threads=1','--exact'])
commands.insert(0,base+['exact_tolerance','--','--test-threads=2'])
paths=subprocess.check_output(['git','ls-files','--cached','--others','--exclude-standard','crates','Cargo.toml','Cargo.lock','rust-toolchain.toml'],cwd=root,text=True).splitlines()
def manifest():return {p:hashlib.sha256((root/p).read_bytes()).hexdigest() for p in sorted(set(paths)) if (root/p).is_file()}
before=manifest();records=[]
for index,cmd in enumerate(commands):
 started=time.monotonic();run=subprocess.run(cmd,cwd=root,env=env,stdout=subprocess.PIPE,stderr=subprocess.STDOUT)
 log=f'{phase}-{index}.log.gz';(out/log).write_bytes(gzip.compress(run.stdout,mtime=0))
 records.append({'command':cmd,'exit':run.returncode,'seconds':time.monotonic()-started,'log':log,'results':re.findall(r'test result:.*',run.stdout.decode(errors='replace'))})
 print(json.dumps(records[-1]),flush=True)
 assert run.returncode==0
assert before==manifest()
(out/(phase+'-source.json.gz')).write_bytes(gzip.compress(json.dumps(before,sort_keys=True).encode(),mtime=0))
(out/(phase+'.json')).write_text(json.dumps({'head':subprocess.check_output(['git','rev-parse','HEAD'],cwd=root,text=True).strip(),'source_unchanged':True,'records':records},indent=2)+'\n')
