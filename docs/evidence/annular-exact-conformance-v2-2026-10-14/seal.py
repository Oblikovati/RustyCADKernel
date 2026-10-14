"""Seal only after complete frozen source gates and exact two-process comparisons pass."""
import pathlib, subprocess, json, gzip, hashlib, re
root=pathlib.Path(__file__).resolve().parents[3];out=pathlib.Path(__file__).resolve().parent
receipt=json.loads((out/'final-gate.json').read_text());assert len(receipt['commands'])==5
assert all(row['exit']==0 and row['source_unchanged'] for row in receipt['commands'])
replays=json.loads((out/'final-replays.json').read_text());assert len(replays)==24
assert all(row['exit']==0 and row['full_bytes_equal'] and row['source_unchanged'] for row in replays)
manifest=json.loads(gzip.decompress((out/'final-source.json.gz').read_bytes()))
assert all(hashlib.sha256((root/p).read_bytes()).hexdigest()==digest for p,digest in manifest.items())
changed=subprocess.check_output(['git','diff','--name-only','978179353',receipt['source'],'--','crates'],cwd=root,text=True).splitlines()
crate_files={p:hashlib.sha256((root/p).read_bytes()).hexdigest() for p in changed if (root/p).is_file()}
artifacts={str(p.relative_to(out)):hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted(out.rglob('*')) if p.is_file() and p.name!='seal.json'}
results={row['label']:sum(int(n) for result in row['results'] for n in re.findall(r'(\d+) passed;',result)) for row in receipt['commands']}
seal={'source':receipt['source'],'dependency_base':'978179353','source_inputs':len(manifest),'results':results,'independent_process_records':len(replays),'crate_files':crate_files,'artifacts':artifacts}
(out/'seal.json').write_text(json.dumps(seal,indent=2)+'\n');print(json.dumps({'source':seal['source'],'results':results,'source_inputs':len(manifest),'crate_files':len(crate_files),'artifacts':len(artifacts)}))
