"""Run the isolated AIS latency gate repeatedly without replacing an acceptance receipt."""
import hashlib,json,os,sys
from pathlib import Path
sys.path.insert(0, str(Path('tools').resolve()))
from verify_vulkan import run
from vulkan_profiles import probe_profile,confirm_device
from vulkan_performance import summarize_samples
profile=probe_profile('host',dict(os.environ),list(Path('/usr/share/vulkan/icd.d').glob('*.json')),run)
binary=Path('target/release/examples/readback_phase_probe').resolve()
baseline=json.loads(Path('porting/runtime-evidence/vulkan-surfaces-baseline-2026-10-10.json').read_text())
previous=next(p for p in baseline['profiles'] if p['profile']=='host')
assert previous['device']['name']==profile.device.name and previous['driver_info']==profile.probe
row=next(r for r in previous['performance'] if r['workload']=='ais-display-erase')
limits={'median_ms':row['median_ms']*1.25,'p95_ms':row['p95_ms']*1.5}
results=[]
for index in range(6):
 output=run([str(binary)],profile.environment);confirm_device(output,profile.device)
 record=json.loads(next(s.removeprefix('MARTENSITE_ISOLATION=') for s in output.splitlines() if s.startswith('MARTENSITE_ISOLATION=')))
 record.update(summarize_samples(record['samples_ms']))
 record['failed_metrics']=[key for key,limit in limits.items() if record[key]>limit]
 results.append(record)
 print(json.dumps({'run':index,'median_ms':record['median_ms'],'p95_ms':record['p95_ms'],'failed_metrics':record['failed_metrics']}),flush=True)
report={'scope':'Isolated original AIS erase/display cycle, 20 warmup and 100 measured two-frame samples. Diagnostic run, does not replace acceptance.','limits':limits,'binary_sha256':hashlib.sha256(binary.read_bytes()).hexdigest(),'source_sha256':hashlib.sha256(Path('target/diagnostics/ais_timing_isolation.rs').read_bytes()).hexdigest(),'device':profile.device.name,'driver_info':profile.probe,'runs':results}
Path('target/diagnostics/ais-readback-isolation.json').write_text(json.dumps(report,indent=2)+'\n')
sys.exit(1 if any(r['failed_metrics'] for r in results) else 0)
