#!/usr/bin/env python3
"""Retain exact source bookends and ordinary-host TKBO commands without masking failures."""
import argparse, hashlib, json, os, pathlib, platform, re, subprocess, time
parser=argparse.ArgumentParser();parser.add_argument('--label',required=True);args=parser.parse_args()
root=pathlib.Path.cwd();out=root/'target/prism-ordered-validation-v1'/args.label;out.mkdir(parents=True,exist_ok=True)
tool='/home/vmiguel/.rustup/toolchains/1.99.0-x86_64-unknown-linux-gnu/bin';env=os.environ.copy();env['PATH']=tool+':'+env['PATH'];env['CARGO_BUILD_JOBS']='2'
def snapshot(name):
 files=subprocess.check_output(['git','ls-files','-z']).decode().split('\0')
 source={f:hashlib.sha256(pathlib.Path(f).read_bytes()).hexdigest() for f in sorted(files) if f and pathlib.Path(f).is_file() and (pathlib.Path(f).suffix in ['.rs','.toml','.lock'] or f.startswith('.cargo/') or 'rust-toolchain' in f)}
 result={'revision':subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip(),'status':subprocess.check_output(['git','status','--short'],text=True),'source_files':source}
 (out/name).write_text(json.dumps(result,indent=2,sort_keys=True)+'\n');return result
before=snapshot('source-before.json')
commands=[('all-targets',[tool+'/cargo','test','-p','rustycad-tkbo','--all-targets','--all-features','--no-fail-fast','--','--test-threads=1']),('doctests',[tool+'/cargo','test','-p','rustycad-tkbo','--doc','--all-features']),('clippy',[tool+'/cargo','clippy','-p','rustycad-tkbo','--all-targets','--all-features','--','-D','warnings']),('format',[tool+'/cargo','fmt','--all','--','--check'])]
records=[]
for name,cmd in commands:
 start=time.monotonic()
 with (out/(name+'.log')).open('w') as log:run=subprocess.run(cmd,env=env,stdout=log,stderr=subprocess.STDOUT)
 records.append({'name':name,'command':cmd,'returncode':run.returncode,'seconds':time.monotonic()-start})
 (out/'commands.json').write_text(json.dumps(records,indent=2)+'\n');print(json.dumps(records[-1]),flush=True)
replays=[]
unit_match=re.search(r'Running unittests src/lib.rs \(([^)]+)\)',(out/'all-targets.log').read_text())
if unit_match:
 binary=root/unit_match.group(1);binary_hash=hashlib.sha256(binary.read_bytes()).hexdigest()
 cases=[('ordered','sketch_region_prism::frozen::ordered_validation::replay_tests::ordered_validation_process_receipt','ordered-validation-v1:'),
 ('containment-v1','sketch_region_prism::frozen::replay_tests::frozen_prism_owned_bytes_replay_across_threads_and_repeated_operations','frozen-prism-v1:'),
 ('topology','sketch_region_prism::frozen::topology::replay_tests::exact_topology_report_replays_across_threads_and_processes','frozen-topology-v1:'),
 ('join','sketch_region_prism::frozen::proof_join::tests::joined_math_byte_receipt','same-body-prism-proof-v1:'),
 ('orientation','sketch_region_prism::frozen::joined_orientation::tests::replay_tests::joined_orientation_process_receipt','joined-orientation-v1:'),
 ('tolerance','sketch_region_prism::exact_tolerance::joined_replay_tests::exact_tolerance_bytes_replay_across_threads_and_operations','prism-exact-tolerance-v2:')]
 for name,test,marker in cases:
  contents=[]
  for repeat in [1,2]:
   cmd=[str(binary),'--exact',test,'--nocapture'];run=subprocess.run(cmd,env=env,capture_output=True)
   (out/f'{name}-process-{repeat}.log').write_bytes(run.stdout+run.stderr)
   rows=[line[len(marker):] for line in run.stdout.decode().splitlines() if line.startswith(marker)]
   content=bytes.fromhex(rows[0]) if run.returncode==0 and len(rows)==1 else b''
   (out/f'{name}-process-{repeat}.bin').write_bytes(content);contents.append(content)
   replays.append({'name':name,'repeat':repeat,'command':cmd,'returncode':run.returncode,'rows':len(rows),'bytes':len(content),'sha256':hashlib.sha256(content).hexdigest()})
  if not contents[0] or contents[0]!=contents[1]: raise RuntimeError(f'incomplete or mismatched replay {name}')
  if name=='containment-v1' and contents[0]!=(root/'crates/tkbo/src/sketch_region_prism/frozen/fixtures/containment-v1.bin').read_bytes(): raise RuntimeError('legacy V1 fixture changed')
 (out/'replays.json').write_text(json.dumps({'binary':str(binary),'binary_sha256_before':binary_hash,'binary_sha256_after':hashlib.sha256(binary.read_bytes()).hexdigest(),'runs':replays},indent=2)+'\n')
after=snapshot('source-after.json')
report={'source_unchanged':before['source_files']==after['source_files'],'revision_unchanged':before['revision']==after['revision'],'host':platform.platform(),'toolchain':subprocess.check_output([tool+'/rustc','-Vv'],text=True),'jobs':2,'commands':records}
(out/'result.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report),flush=True)
raise SystemExit(0 if report['source_unchanged'] and report['revision_unchanged'] and all(r['returncode']==0 for r in records) else 1)
