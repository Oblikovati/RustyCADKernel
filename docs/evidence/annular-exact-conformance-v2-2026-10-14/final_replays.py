"""Compare full old/new payloads and old callback trace in two independent processes."""
import pathlib,subprocess,gzip,hashlib,json,re,os,time
root=pathlib.Path(__file__).resolve().parents[3];out=pathlib.Path(__file__).resolve().parent
env=os.environ.copy();env['PATH']='/home/vmiguel/.cargo/bin:'+env['PATH'];env['CARGO_BUILD_JOBS']='2'
manifest=json.loads(gzip.decompress((out/'final-source.json.gz').read_bytes()))
def stable(): return all(hashlib.sha256((root/p).read_bytes()).hexdigest()==digest for p,digest in manifest.items())
def payloads(raw):
 rows=re.findall(rb'([a-z0-9-]+)[:=]([0-9a-f]{100,})',raw)
 if rows:return [(tag,bytes.fromhex(value.decode())) for tag,value in rows]
 match=re.search(rb'(\{"operation":"martensite.region-certificate"[^\n]+)',raw)
 if match:return [(b'region-json',match.group(1))]
 match=re.search(rb'exact-conformance-callback-baseline:([^\n]+)',raw)
 assert match,'missing exact replay payload'
 return [(b'callback-trace',match.group(1))]
old=json.loads((out/'baseline.json').read_text())['records'][1:]
new=json.loads((out/'behavior-pinned.json').read_text())['records'][1]
records=[]
for process in ['first','second']:
 for index,reference in enumerate(old+[new]):
  assert stable();start=time.monotonic();command=reference['command'];run=subprocess.run(command,cwd=root,env=env,stdout=subprocess.PIPE,stderr=subprocess.STDOUT)
  log=f'final-replay-{process}-{index}.log.gz';(out/log).write_bytes(gzip.compress(run.stdout,mtime=0))
  expected=payloads(gzip.decompress((out/reference['log']).read_bytes()));actual=payloads(run.stdout)
  equal=bool(actual) and expected==actual
  record={'command':command,'exit':run.returncode,'seconds':time.monotonic()-start,'log':log,'reference':reference['log'],'full_bytes_equal':equal,'source_unchanged':stable(),'payloads':[{'tag':tag.decode(),'bytes':len(value),'sha256':hashlib.sha256(value).hexdigest()} for tag,value in actual]}
  records.append(record);(out/'final-replays.json').write_text(json.dumps(records,indent=2)+'\n');print(json.dumps(record),flush=True)
  assert run.returncode==0 and equal and record['source_unchanged']
command=['cargo','+1.99.0','test','-p','rustycad-tkbo','--lib','sketch_region_prism::exact_tolerance::annular::cancel_tests::every_annular_geometry_checkpoint_cancels_with_exact_prefix_and_reuse','--','--exact','--nocapture','--test-threads=1']
run=subprocess.run(command,cwd=root,env=env,stdout=subprocess.PIPE,stderr=subprocess.STDOUT)
(out/'final-cancellation.log.gz').write_bytes(gzip.compress(run.stdout,mtime=0));assert run.returncode==0 and stable()
match=re.search(rb'annular-conformance-cancellation-checkpoints=(\d+)',run.stdout);assert match
(out/'final-cancellation.json').write_text(json.dumps({'command':command,'exit':run.returncode,'geometry_checkpoints':int(match[1]),'metadata_checkpoints':16,'source_unchanged':True},indent=2)+'\n')
