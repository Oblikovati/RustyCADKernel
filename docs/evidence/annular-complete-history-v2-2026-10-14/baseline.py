from pathlib import Path
import subprocess,os,json,gzip,hashlib,re
out=Path('/tmp/martensite-prism-annular-history'); env=os.environ.copy();env['PATH']='/home/vmiguel/.cargo/bin:'+env['PATH'];env['CARGO_BUILD_JOBS']='2'
filters=['portable_region_certificate_process_receipt','complete_prism_history_replay_across_processes_and_threads','prism_copy_lineage_owned_bytes_replay_across_processes_and_threads','frozen_prism_owned_bytes_replay_across_threads_and_repeated_operations','sketch_region_prism::publication::replay_tests::publication_process_replay','joined_measure_process_replay','two_loop_owned_process_receipt','annular_frozen_join_owned_process_receipt']
manifest={};commands=[]
for index,test in enumerate(filters):
 command=['cargo','+1.99.0','test','-p','rustycad-tkbo','--all-features',test,'--','--test-threads=1','--nocapture'];log=out/('baseline-'+str(index)+'.log')
 with log.open('wb') as stream: status=subprocess.run(command,stdout=stream,stderr=subprocess.STDOUT,env=env).returncode
 commands.append({'command':command,'exit':status});assert status==0,test
 raw=log.read_bytes()
 if index==0:values=re.findall(rb'\{"operation":"martensite.region-certificate"[^\n]*\}',raw)
 elif index==6: values=[bytes.fromhex(x.decode()) for x in re.findall(rb'two-loop-region-domain-v2:([0-9a-f]+)',raw)]
 else:values=[bytes.fromhex(x.decode()) for x in re.findall(rb'(?<![0-9a-f])[0-9a-f]{600,}(?![0-9a-f])',raw)]
 values=set(values);assert values,test;manifest[test]=[{'bytes':len(v),'sha256':hashlib.sha256(v).hexdigest()} for v in sorted(values)]
 print(test,status,manifest[test],flush=True)
(out/'baseline-legacy.json').write_text(json.dumps(manifest,indent=2)+'\n');(out/'baseline-commands.json').write_text(json.dumps(commands,indent=2)+'\n')
