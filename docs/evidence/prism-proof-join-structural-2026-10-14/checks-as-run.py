import hashlib,json,os,pathlib,re,subprocess,time,datetime,platform
root=pathlib.Path('/home/vmiguel/git/oblikovati-workspace/Martensite-prism-proof-composition');out=pathlib.Path('/tmp/martensite-proof-composition-structural-93fac56f2');out.mkdir(exist_ok=True)
baseline=pathlib.Path('/tmp/martensite-proof-composition-frozen-afb12e5cc')
env=os.environ.copy();env['PATH']='/home/vmiguel/.cargo/bin:'+env['PATH'];env['CARGO_BUILD_JOBS']='2';env['CARGO_TARGET_DIR']=str(root/'target');cargo=['/home/vmiguel/.cargo/bin/cargo','+1.99.0']
def output(args):return subprocess.check_output(args,cwd=root,env=env,text=True).strip()
def hashes():
 paths=output(['git','ls-files']).splitlines();paths=[p for p in paths if p.startswith(('crates/','.cargo/')) or p in ('Cargo.toml','Cargo.lock','rust-toolchain.toml','rust-toolchain')]
 return {p:hashlib.sha256((root/p).read_bytes()).hexdigest() for p in paths if (root/p).is_file()}
assert not output(['git','status','--porcelain'])
receipt={'source_revision':output(['git','rev-parse','HEAD']),'baseline_revision':'afb12e5ccc0119344c6aecd9157cddd6fc4be751','started_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'platform':platform.platform(),'rustc':output(['/home/vmiguel/.cargo/bin/rustc','+1.99.0','-vV']),'jobs':2,'steps':[],'replays':[]}
before=hashes();(out/'inputs-before.json').write_text(json.dumps(before,indent=2,sort_keys=True)+'\n')
def run(name,args):
 print('START '+name,flush=True);t=time.monotonic()
 with (out/(name+'.log')).open('w') as log:r=subprocess.run(args,cwd=root,env=env,stdout=log,stderr=subprocess.STDOUT)
 receipt['steps'].append({'name':name,'argv':args,'exit_code':r.returncode,'seconds':time.monotonic()-t});(out/'receipt.json').write_text(json.dumps(receipt,indent=2)+'\n');print('DONE '+name+' '+str(r.returncode),flush=True);assert r.returncode==0,name
for name,selector in [('frozen_tests','sketch_region_prism::frozen'),('source_query_tests','sketch_region_source_conformance'),('closure_census','shared_enumeration_bijects_every_retained_directed_closure_context')]:run(name,cargo+['test','-p','rustycad-tkbo','--lib','--all-features','--locked',selector,'--','--test-threads=1'])
run('doctests',cargo+['test','-p','rustycad-tkbo','--doc','--all-features','--locked'])
run('clippy',cargo+['clippy','-p','rustycad-tkbo','--all-targets','--all-features','--locked','--','-D','warnings'])
run('fmt',cargo+['fmt','--all','--','--check'])
replays=[('join','sketch_region_prism::frozen::proof_join::tests::joined_math_byte_receipt','same-body-prism-proof-v1:'),('frozen_v1','sketch_region_prism::frozen::replay_tests::frozen_prism_owned_bytes_replay_across_threads_and_repeated_operations','frozen-prism-v1:'),('complete_corrected','sketch_region_prism::region_certificate::measurements::tests::complete_tests::replay_tests::complete_prism_measure_process_receipt','{"operation":'),('cap_subset','sketch_region_prism::region_certificate::measurements::tests::replay_tests::portable_prism_measure_process_receipt','{"operation":')]
for name,test,prefix in replays:
 for process in [1,2]:
  label=f'{name}-process-{process}';run(label,cargo+['test','-p','rustycad-tkbo','--lib','--all-features','--locked',test,'--','--exact','--nocapture'])
  text=(out/(label+'.log')).read_text();assert '1 passed; 0 failed' in text
  lines=[line for line in text.splitlines() if line.startswith(prefix)];assert len(lines)==1
  value=bytes.fromhex(lines[0].split(':',1)[1]) if name in ('join','frozen_v1') else (lines[0]+'\n').encode()
  (out/(label+'.bin')).write_bytes(value);assert value==(baseline/(name+'-process-1.bin')).read_bytes()
 receipt['replays'].append({'name':name,'bytes':len(value),'sha256':hashlib.sha256(value).hexdigest(),'two_processes_and_frozen_baseline_equal':True})
after=hashes();assert before==after;(out/'inputs-after.json').write_text(json.dumps(after,indent=2,sort_keys=True)+'\n')
receipt.update(inputs_count=len(before),input_hashes_unchanged=True,status='passed',finished_utc=datetime.datetime.now(datetime.timezone.utc).isoformat())
receipt['passed']={name:sum(map(int,re.findall(r'test result: ok\. (\d+) passed',(out/(name+'.log')).read_text()))) for name in ('frozen_tests','source_query_tests','closure_census','doctests')}
(out/'receipt.json').write_text(json.dumps(receipt,indent=2)+'\n');print(json.dumps({k:receipt[k] for k in ('source_revision','inputs_count','passed','replays','status')}),flush=True)
