import hashlib,json,os,pathlib,re,subprocess,time,datetime
root=pathlib.Path('/home/vmiguel/git/oblikovati-workspace/Martensite-prism-proof-composition');out=pathlib.Path('/tmp/martensite-proof-composition-frozen-afb12e5cc')
env=os.environ.copy();env['PATH']='/home/vmiguel/.cargo/bin:'+env['PATH'];env['CARGO_BUILD_JOBS']='2';env['CARGO_TARGET_DIR']=str(root/'target')
receipt=json.loads((out/'receipt.json').read_text());assert receipt['source_revision']==subprocess.check_output(['git','rev-parse','HEAD'],cwd=root,text=True).strip()
receipt['runner_note']='Initial standalone complete-measure selector used source filename instead of registered module path and executed zero tests; assertion rejected it. Corrected exact selectors below each execute one test. No zero-test runtime credit.'
replays=[('complete_corrected','sketch_region_prism::region_certificate::measurements::tests::complete_tests::replay_tests::complete_prism_measure_process_receipt'),('cap_subset','sketch_region_prism::region_certificate::measurements::tests::replay_tests::portable_prism_measure_process_receipt')]
for name,test in replays:
 values=[]
 for process in [1,2]:
  label=f'{name}-process-{process}';args=['/home/vmiguel/.cargo/bin/cargo','+1.99.0','test','-p','rustycad-tkbo','--lib','--all-features','--locked',test,'--','--exact','--nocapture'];start=time.monotonic()
  with (out/(label+'.log')).open('w') as log:r=subprocess.run(args,cwd=root,env=env,stdout=log,stderr=subprocess.STDOUT)
  receipt['steps'].append({'name':label,'argv':args,'exit_code':r.returncode,'seconds':time.monotonic()-start});assert r.returncode==0
  text=(out/(label+'.log')).read_text();assert '1 passed; 0 failed' in text
  lines=[line for line in text.splitlines() if line.startswith('{"operation":')];assert len(lines)==1
  value=(lines[0]+'\n').encode();(out/(label+'.bin')).write_bytes(value);values.append(value)
 assert values[0]==values[1]
 receipt.setdefault('replays',[]).append({'name':name,'bytes':len(values[0]),'sha256':hashlib.sha256(values[0]).hexdigest(),'two_processes_equal':True})
paths=subprocess.check_output(['git','ls-files'],cwd=root,text=True).splitlines();paths=[p for p in paths if p.startswith(('crates/','.cargo/')) or p in ('Cargo.toml','Cargo.lock','rust-toolchain.toml','rust-toolchain')]
after={p:hashlib.sha256((root/p).read_bytes()).hexdigest() for p in paths if (root/p).is_file()};before=json.loads((out/'inputs-before.json').read_text());(out/'inputs-after.json').write_text(json.dumps(after,indent=2,sort_keys=True)+'\n');assert before==after
receipt.update(inputs_count=len(before),input_hashes_unchanged=True,tests_passed=sum(map(int,re.findall(r'test result: ok\. (\d+) passed',(out/'tests.log').read_text()))),doctests_passed=sum(map(int,re.findall(r'test result: ok\. (\d+) passed',(out/'doctests.log').read_text()))),finished_utc=datetime.datetime.now(datetime.timezone.utc).isoformat(),status='passed')
(out/'receipt.json').write_text(json.dumps(receipt,indent=2)+'\n');print(json.dumps({k:receipt[k] for k in ('source_revision','tests_passed','doctests_passed','inputs_count','input_hashes_unchanged','replays','status')}))
