"""Phase evidence for the isolated host AIS cycle; not a replacement timing gate."""
import hashlib,json,os,sys
from pathlib import Path
sys.path.insert(0,str(Path('tools').resolve()))
from verify_vulkan import run
from vulkan_profiles import probe_profile,confirm_device
from vulkan_performance import summarize_samples
profile=probe_profile('host',dict(os.environ),list(Path('/usr/share/vulkan/icd.d').glob('*.json')),run)
binary=Path('target/vulkan-harness/release/examples/ais_timing_phases').resolve()
previous=json.loads(Path('target/diagnostics/ais-surface-isolation.json').read_text())
assert previous['driver_info']==profile.probe
limits=previous['limits'];runs=[]
for index in range(6):
 output=run([str(binary)],profile.environment);confirm_device(output,profile.device)
 record=json.loads(next(s.removeprefix('MARTENSITE_PHASES=') for s in output.splitlines() if s.startswith('MARTENSITE_PHASES=')))
 samples=record['samples_ms']
 record['total']=summarize_samples([r[5] for r in samples])
 record['backend']=summarize_samples([r[4] for r in samples])
 record['outside_backend']=summarize_samples([r[5]-r[4] for r in samples])
 record['ais_context']=summarize_samples([r[0]+r[2] for r in samples])
 before=list(map(int,record['schedstat_before'].split()));after=list(map(int,record['schedstat_after'].split()))
 record['render_thread_cpu_ms']=(after[0]-before[0])/1e6
 record['render_thread_runqueue_ms']=(after[1]-before[1])/1e6
 record['failed_metrics']=[key for key,limit in limits.items() if record['total'][key]>limit]
 runs.append(record)
 print(json.dumps({'run':index,'total_median':record['total']['median_ms'],'backend_median':record['backend']['median_ms'],'outside_backend_median':record['outside_backend']['median_ms'],'ais_median':record['ais_context']['median_ms'],'cpu_ms':record['render_thread_cpu_ms'],'runqueue_ms':record['render_thread_runqueue_ms'],'failed':record['failed_metrics']}),flush=True)
report={'scope':'Instrumented AIS erase/display 20 warmup, 100 samples. Backend wall time includes allocation, submission, fence wait, capture and scheduling. Render-thread schedstat excludes driver worker threads.','columns':['erase_ms','erased_redraw_ms','display_ms','displayed_redraw_ms','backend_sum_ms','total_ms'],'limits':limits,'device':profile.device.name,'driver_info':profile.probe,'source_sha256':hashlib.sha256(Path('target/diagnostics/ais_timing_phases.rs').read_bytes()).hexdigest(),'binary_sha256':hashlib.sha256(binary.read_bytes()).hexdigest(),'runs':runs}
Path('target/diagnostics/ais-surface-phases.json').write_text(json.dumps(report,indent=2)+'\n')
sys.exit(1 if any(r['failed_metrics'] for r in runs) else 0)
