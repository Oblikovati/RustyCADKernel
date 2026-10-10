"""Retain six isolated AIS runs, with five measured backend phases per frame."""
import hashlib,json,os,sys
from pathlib import Path
sys.path.insert(0,str(Path('tools').resolve()))
from verify_vulkan import run
from vulkan_profiles import probe_profile,confirm_device
from vulkan_performance import summarize_samples
profile=probe_profile('host',dict(os.environ),list(Path('/usr/share/vulkan/icd.d').glob('*.json')),run)
binary=Path('target/release/examples/readback_phase_probe').resolve()
records=[]
for index in range(6):
 output=run([str(binary)],profile.environment);confirm_device(output,profile.device)
 events=[json.loads(line) for line in output.splitlines() if line.startswith('{"diagnostic":')]
 phases=[event['phases_ms'] for event in events if event['diagnostic']=='DEBUG-martensite-backend-phases'][40:]
 assert len(phases)==200
 cycles=[[sum(pair[n] for pair in phases[i:i+2]) for n in range(5)] for i in range(0,len(phases),2)]
 totals=json.loads(next(line.removeprefix('MARTENSITE_ISOLATION=') for line in output.splitlines() if line.startswith('MARTENSITE_ISOLATION=')))
 summaries=[summarize_samples([row[n] for row in cycles]) for n in range(5)]
 memory=[event for event in events if event['diagnostic']=='DEBUG-martensite-memory-selection']
 record={'frame_samples_ms':phases,'cycle_summary':summaries,'whole_cycle':summarize_samples(totals['samples_ms']),'cycle_samples_ms':totals['samples_ms'],'memory_selections':sorted({(r['index'],r['flags'],r['allowed']) for r in memory})}
 records.append(record)
 print(json.dumps({'run':index,'cycle_median':record['whole_cycle']['median_ms'],'phase_medians':[r['median_ms'] for r in summaries],'memory':record['memory_selections']}),flush=True)
result={'phase_order':['resource_creation_upload','command_creation_recording','submission_fence_wait','capture_readback','destruction'],'scope':'Instrumented isolated AIS erase/display; log overhead outside measured backend phase boundaries. 20 warmup and 100 two-frame samples. Not an acceptance gate.','device':profile.device.name,'driver_info':profile.probe,'binary_sha256':hashlib.sha256(binary.read_bytes()).hexdigest(),'patch_sha256':hashlib.sha256(Path('target/diagnostics/backend-phase-after.patch').read_bytes()).hexdigest(),'runs':records}
Path('target/diagnostics/backend-phase-after.json').write_text(json.dumps(result,indent=2)+'\n')
