import hashlib,json,os,pathlib,platform,re,subprocess,time
out=pathlib.Path('/tmp/martensite-copy-provenance-evidence')
env=os.environ.copy();env['PATH']='/home/vmiguel/.cargo/bin:'+env['PATH'];env['CARGO_BUILD_JOBS']='2'
def sources():
 files=subprocess.check_output(['git','ls-files','crates','Cargo.toml','Cargo.lock']).decode().splitlines()
 return {f:hashlib.sha256(pathlib.Path(f).read_bytes()).hexdigest() for f in files}
initial=sources();(out/'final-source-before.json').write_text(json.dumps(initial,indent=2)+'\n')
receipt={'revision':subprocess.check_output(['git','rev-parse','HEAD']).decode().strip(),'platform':platform.platform(),'rustc':subprocess.check_output(['rustc','+1.99.0','-Vv'],env=env).decode(),'jobs':2,'commands':[]}
packages=['-p','rustycad-tkmath','-p','rustycad-tkbrep','-p','rustycad-tkbo']
commands=[('clippy',['clippy',*packages,'--all-targets','--all-features','--locked','--','-D','warnings']),('fmt',['fmt','--all','--check']),('all-targets',['test',*packages,'--all-targets','--all-features','--locked']),('doctests',['test',*packages,'--doc','--all-features','--locked'])]
for label in ['process-first','process-second']:
 commands.append((label,['test','-p','rustycad-tkbo','--lib','prism_copy_lineage_owned_bytes_replay_across_processes_and_threads','--locked','--','--nocapture','--test-threads=1']))
commands.append(('legacy-replay',['test','-p','rustycad-tkbo','--lib','frozen_prism_owned_bytes_replay_across_threads_and_repeated_operations','--locked','--','--nocapture','--test-threads=1']))
for label,args in commands:
 cmd=['cargo','+1.99.0',*args];start=time.monotonic();log=out/('final-'+label+'.log')
 with log.open('wb') as stream: code=subprocess.run(cmd,env=env,stdout=stream,stderr=subprocess.STDOUT).returncode
 current=sources();text=log.read_text(errors='replace')
 record={'label':label,'command':cmd,'exit_code':code,'seconds':time.monotonic()-start,'source_unchanged':current==initial,'passed_tests':sum(int(n) for n in re.findall(r'test result: ok\. (\d+) passed',text))}
 receipt['commands'].append(record);(out/'final-receipt.json').write_text(json.dumps(receipt,indent=2)+'\n');print(json.dumps(record),flush=True)
 if code or current!=initial: break
(out/'final-source-after.json').write_text(json.dumps(sources(),indent=2)+'\n')
