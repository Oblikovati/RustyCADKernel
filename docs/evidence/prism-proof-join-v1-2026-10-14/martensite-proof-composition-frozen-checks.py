import hashlib,json,os,pathlib,platform,re,subprocess,time,datetime,gzip
root=pathlib.Path('/home/vmiguel/git/oblikovati-workspace/Martensite-prism-proof-composition')
out=pathlib.Path('/tmp/martensite-proof-composition-frozen-afb12e5cc');out.mkdir(exist_ok=True)
env=os.environ.copy();env['PATH']='/home/vmiguel/.cargo/bin:'+env['PATH'];env['CARGO_BUILD_JOBS']='2';env['CARGO_TARGET_DIR']=str(root/'target')
cargo=['/home/vmiguel/.cargo/bin/cargo','+1.99.0']
def output(args):return subprocess.check_output(args,cwd=root,env=env,text=True).strip()
def inputs():
 paths=output(['git','ls-files']).splitlines()
 paths=[p for p in paths if p.startswith(('crates/','.cargo/')) or p in ('Cargo.toml','Cargo.lock','rust-toolchain.toml','rust-toolchain')]
 return {p:hashlib.sha256((root/p).read_bytes()).hexdigest() for p in paths if (root/p).is_file()}
receipt={'source_revision':output(['git','rev-parse','HEAD']),'started_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'platform':platform.platform(),'rustc':output(['/home/vmiguel/.cargo/bin/rustc','+1.99.0','-vV']),'jobs':2,'target_dir':str(root/'target'),'scope':'TKBO only; TKMath/TKBRep sources unchanged; no OCCT/GPU/TSan execution','steps':[]}
assert not output(['git','status','--porcelain'])
before=inputs();(out/'inputs-before.json').write_text(json.dumps(before,indent=2,sort_keys=True)+'\n')
def run(name,args):
 print('START '+name,flush=True);start=time.monotonic()
 with (out/(name+'.log')).open('w') as log: result=subprocess.run(args,cwd=root,env=env,stdout=log,stderr=subprocess.STDOUT)
 receipt['steps'].append({'name':name,'argv':args,'exit_code':result.returncode,'seconds':time.monotonic()-start})
 (out/'receipt.json').write_text(json.dumps(receipt,indent=2)+'\n')
 print('DONE '+name+' '+str(result.returncode),flush=True)
 if result.returncode:raise RuntimeError(name)
run('tests',cargo+['test','-p','rustycad-tkbo','--all-targets','--all-features','--locked','--','--test-threads=1'])
run('doctests',cargo+['test','-p','rustycad-tkbo','--doc','--all-features','--locked'])
run('clippy',cargo+['clippy','-p','rustycad-tkbo','--all-targets','--all-features','--locked','--','-D','warnings'])
run('fmt',cargo+['fmt','--all','--','--check'])
run('docs',cargo+['doc','-p','rustycad-tkbo','--all-features','--no-deps','--locked'])
replays=[('join','sketch_region_prism::frozen::proof_join::tests::joined_math_byte_receipt','same-body-prism-proof-v1:'),('frozen_v1','sketch_region_prism::frozen::replay_tests::frozen_prism_owned_bytes_replay_across_threads_and_repeated_operations','frozen-prism-v1:'),('complete','sketch_region_prism::region_certificate::measurements::tests::complete::replay::complete_prism_measure_process_receipt','{"operation":"martensite.prism-complete-measures"'),('cap_subset','sketch_region_prism::region_certificate::measurements::tests::replay::portable_prism_measure_process_receipt','{"operation":')]
receipt['replays']=[]
for name,test,prefix in replays:
 extracted=[]
 for process in [1,2]:
  label=f'{name}-process-{process}'
  run(label,cargo+['test','-p','rustycad-tkbo','--lib','--all-features','--locked',test,'--','--exact','--nocapture'])
  lines=[line for line in (out/(label+'.log')).read_text().splitlines() if line.startswith(prefix)]
  assert len(lines)==1,(label,len(lines))
  value=bytes.fromhex(lines[0].split(':',1)[1]) if name in ('join','frozen_v1') else (lines[0]+'\n').encode()
  (out/(label+'.bin')).write_bytes(value);extracted.append(value)
 assert extracted[0]==extracted[1],name
 receipt['replays'].append({'name':name,'bytes':len(extracted[0]),'sha256':hashlib.sha256(extracted[0]).hexdigest(),'two_processes_equal':True})
after=inputs();(out/'inputs-after.json').write_text(json.dumps(after,indent=2,sort_keys=True)+'\n')
receipt['inputs_count']=len(before);receipt['input_hashes_unchanged']=before==after
assert before==after
receipt['tests_passed']=sum(map(int,re.findall(r'test result: ok\. (\d+) passed', (out/'tests.log').read_text())))
receipt['doctests_passed']=sum(map(int,re.findall(r'test result: ok\. (\d+) passed', (out/'doctests.log').read_text())))
receipt['finished_utc']=datetime.datetime.now(datetime.timezone.utc).isoformat()
receipt['status']='passed';(out/'receipt.json').write_text(json.dumps(receipt,indent=2)+'\n')
print(json.dumps({k:receipt[k] for k in ('source_revision','tests_passed','doctests_passed','inputs_count','input_hashes_unchanged','replays','status')}),flush=True)
