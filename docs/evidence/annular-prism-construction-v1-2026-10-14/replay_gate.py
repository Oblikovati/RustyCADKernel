import gzip,hashlib,json,os,pathlib,re,subprocess,sys,time
root=pathlib.Path('/home/vmiguel/git/oblikovati-workspace/Martensite-prism-annular-construction')
out=root/'docs/evidence/annular-prism-construction-v1-2026-10-14';out.mkdir(parents=True,exist_ok=True)
phase=sys.argv[1]
env=os.environ.copy();env['PATH']='/home/vmiguel/.cargo/bin:'+env['PATH'];env['CARGO_BUILD_JOBS']='2'
filters=['portable_region_certificate_process_receipt','complete_prism_history_replay_across_processes_and_threads','prism_copy_lineage_owned_bytes_replay_across_processes_and_threads','frozen_prism_owned_bytes_replay_across_threads_and_repeated_operations','joined_measure_process_replay','publication_process_replay','joined_volume_process_replay']
def manifest():
 paths=subprocess.check_output(['git','ls-files','--cached','--others','--exclude-standard','crates','Cargo.toml','Cargo.lock','rust-toolchain.toml'],cwd=root,text=True).splitlines()
 return {p:hashlib.sha256((root/p).read_bytes()).hexdigest() for p in sorted(set(paths)) if (root/p).is_file()}
before=manifest();records=[]
for case in filters:
 cmd=['cargo','+1.99.0','test','-p','rustycad-tkbo','--lib',case,'--','--nocapture','--test-threads=1']
 start=time.monotonic();proc=subprocess.run(cmd,cwd=root,env=env,stdout=subprocess.PIPE,stderr=subprocess.STDOUT)
 log=out/(phase+'-'+case+'.log.gz');log.write_bytes(gzip.compress(proc.stdout,mtime=0))
 rows=re.findall(rb'([a-z0-9-]+)[:=]([0-9a-f]{100,})',proc.stdout)
 if case == 'portable_region_certificate_process_receipt':
  raw=re.search(rb'(\{"operation":"martensite.region-certificate"[^\n]+)',proc.stdout)
  if raw:rows=[(b'region-json',raw.group(1).hex().encode())]
 record={'filter':case,'command':cmd,'exit':proc.returncode,'seconds':time.monotonic()-start,'log':log.name,'payloads':[{'tag':tag.decode(),'bytes':len(h)//2,'sha256':hashlib.sha256(bytes.fromhex(h.decode())).hexdigest()} for tag,h in rows]}
 records.append(record)
 print(json.dumps(record),flush=True)
 if proc.returncode or not rows:raise SystemExit('missing successful replay')
after=manifest();assert before==after
(out/(phase+'-source.json.gz')).write_bytes(gzip.compress(json.dumps(before,sort_keys=True).encode(),mtime=0))
(out/(phase+'.json')).write_text(json.dumps({'head':subprocess.check_output(['git','rev-parse','HEAD'],cwd=root,text=True).strip(),'source_bookends_equal':True,'source_files':len(before),'commands':records},indent=2)+'\n')
