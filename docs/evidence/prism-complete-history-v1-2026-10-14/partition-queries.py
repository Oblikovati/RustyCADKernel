import pathlib,json,hashlib,subprocess,os,time,platform,re
root=pathlib.Path.cwd();out=pathlib.Path('/tmp/martensite-prism-complete-history');env=os.environ.copy();env['PATH']='/home/vmiguel/.cargo/bin:'+env['PATH'];env['CARGO_BUILD_JOBS']='2'
files=subprocess.check_output(['git','ls-files','crates/tkbo','Cargo.toml','Cargo.lock']).decode().splitlines()
def snapshot(): return {f:hashlib.sha256((root/f).read_bytes()).hexdigest() for f in files}
before=snapshot();(out/'partition-queries-source.json').write_text(json.dumps(before,indent=2)+'\n')
receipt={'revision':subprocess.check_output(['git','rev-parse','HEAD']).decode().strip(),'platform':platform.platform(),'rustc':subprocess.check_output(['rustc','+1.99.0','-Vv'],env=env).decode(),'jobs':2,'commands':[]}
filters=[('all-tkbo',''),('queries','partition_member_query_tests'),('lineage','prism_copy_lineage_owned_bytes_replay_across_processes_and_threads'),('frozen','frozen_prism_owned_bytes_replay_across_threads_and_repeated_operations')]
for label,needle in filters:
 cmd=['cargo','+1.99.0','test','-p','rustycad-tkbo','--lib',needle,'--locked','--','--nocapture','--test-threads=1'];start=time.monotonic()
 with (out/f'partition-queries-{label}.log').open('wb') as log: code=subprocess.run(cmd,env=env,stdout=log,stderr=subprocess.STDOUT).returncode
 entry={'command':cmd,'exit_code':code,'seconds':time.monotonic()-start,'source_unchanged':before==snapshot()};receipt['commands'].append(entry);(out/'partition-queries.json').write_text(json.dumps(receipt,indent=2)+'\n');print(json.dumps(entry),flush=True)
 if code:break
