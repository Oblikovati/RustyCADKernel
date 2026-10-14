"""Run from the checkout root with an exclusive GPU lease; optional fresh output directory argument."""
import os, sys, subprocess, json, hashlib, platform, time
from pathlib import Path
root=Path.cwd(); out=root/(sys.argv[1] if len(sys.argv)>1 else 'target/gui-surface-outcomes-replay')
out.mkdir(parents=True,exist_ok=False)
sys.path.insert(0,str(root/'tools'))
from vulkan_source_receipt import source_hashes

def hashes():
    result=source_hashes()
    for name in ['tools/release_verification_plan.py','tools/tests/test_release_verification_plan.py','docs/adr/0028-typed-gui-surface-outcomes.md','docs/release-notes/unreleased.md']:
        result[name]=hashlib.sha256((root/name).read_bytes()).hexdigest()
    return result

def save(name,value): (out/name).write_text(json.dumps(value,indent=2,sort_keys=True)+'\n')
before=hashes(); save('source-before.json',before)
env=os.environ.copy(); env.update(PATH='/home/vmiguel/.cargo/bin:'+env['PATH'],CARGO_BUILD_JOBS='2',PYTHONPATH='tools')
layer=Path('/home/vmiguel/git/oblikovati-workspace/RustyCADKernel/target/vulkan-validation/extracted/usr')
env.update(VK_LAYER_PATH=str(layer/'share/vulkan/explicit_layer.d'),LD_LIBRARY_PATH=str(layer/'lib/x86_64-linux-gnu'))
env.pop('MARTENSITE_VULKAN_DEVICE',None)
receipt={'source_revision':subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip(),'platform':platform.platform(),'validation_sha256':hashlib.sha256((layer/'lib/x86_64-linux-gnu/libVkLayer_khronos_validation.so').read_bytes()).hexdigest(),'runs':[],'performance_claim':'paired debug-profile observations only; shared host, no controlled previous-release acceptance'}
def run(name,args,extra=None,timeout=1800):
    current=env| (extra or {})
    record={'name':name,'command':args,'environment':{key:current.get(key) for key in ['DISPLAY','VK_LAYER_PATH','LD_LIBRARY_PATH','VK_DRIVER_FILES','VK_ICD_FILENAMES','CARGO_BUILD_JOBS','RUSTDOCFLAGS']},'load_before':os.getloadavg()}
    print(json.dumps({'starting':name}),flush=True); start=time.monotonic()
    with (out/(name+'.log')).open('w') as log:
        try: record['exit']=subprocess.run(args,env=current,stdout=log,stderr=subprocess.STDOUT,timeout=timeout).returncode
        except subprocess.TimeoutExpired:record['timeout']=timeout
    record['elapsed_seconds']=time.monotonic()-start;record['load_after']=os.getloadavg();receipt['runs'].append(record);save('receipt.json',receipt);print(json.dumps(record),flush=True)
    return record.get('exit')==0
cargo=['cargo','+1.99.0']; package=['-p','martensite-vulkan']
ready=run('build-tests',cargo+['test']+package+['--all-targets','--all-features','--no-run'])
ready=run('build-examples',cargo+['build']+package+['--examples','--all-features']) and ready
run('clippy',cargo+['clippy']+package+['--all-targets','--all-features','--','-D','warnings'])
run('docs',cargo+['doc']+package+['--all-features','--no-deps'],{'RUSTDOCFLAGS':'-D warnings'})
run('doctests',cargo+['test']+package+['--doc','--all-features'])
run('fmt',cargo+['fmt','--all','--','--check'])
run('release-plan',['python3','-m','unittest','tools.tests.test_release_verification_plan'])
if ready:
    for driver,icd in [('amd','radeon_icd.json'),('lavapipe','lvp_icd.json')]:
        env.update(VK_DRIVER_FILES='/usr/share/vulkan/icd.d/'+icd,VK_ICD_FILENAMES='/usr/share/vulkan/icd.d/'+icd)
        run(driver+'-full',cargo+['test']+package+['--all-targets','--all-features','--no-fail-fast','--','--test-threads=1'])
        run(driver+'-phases',cargo+['test']+package+['--lib','--all-features','--','--exact','backend::window_gui_native_tests::native_gui_surface_phase_gate','--ignored','--nocapture','--test-threads=1'])
        for example in ['native_window_lifecycle','native_display_lifecycle']:
            run(driver+'-'+example,[str(root/'target/debug/examples'/example)])
        for index in range(5 if driver=='amd' else 2):
            directory=out/f'{driver}-rapid-{index}';checked=index%2==1 or index==4
            run(directory.name,[str(root/'target/debug/examples/native_gui_rapid'),str(directory)]+(['--checked'] if checked else []))
        overlay=out/(driver+'-acceptance');overlay.mkdir(exist_ok=True)
        run(driver+'-acceptance',[str(root/'target/debug/examples/native_gui_acceptance'),str(overlay/'overlay')])
    checker_input=out/'checker-input';checker_input.mkdir()
    for capture in [*out.glob('*-rapid-*'),out/'amd-acceptance',out/'lavapipe-acceptance']:
        if capture.is_dir(): (checker_input/capture.name).symlink_to(Path('..')/capture.name,target_is_directory=True)
    run('exact-framebuffers',['python3',str(root/'docs/evidence/gui-framebuffer-restart-2026-10-14/check_framebuffers.py'),str(checker_input)])
    for index in range(2):
        run('host-canonical-'+str(index),cargo+['test']+package+['--test','toolkit','checked_outcome_bytes_are_stable_across_independent_host_threads','--','--nocapture','--test-threads=1'])
    canonical=[]
    for index in range(2):
        lines=(out/('host-canonical-'+str(index)+'.log')).read_text().splitlines()
        canonical.append([line[line.index('{'): ] for line in lines if '"case":"checked-gui-canonical"' in line])
    receipt['two_process_host_bytes_equal']=canonical[0]==canonical[1] and len(canonical[0])==1
    if all(item.get('exit')==0 for item in receipt['runs']):
        for driver,icd in [('amd','radeon_icd.json'),('lavapipe','lvp_icd.json')]:
            env.update(VK_DRIVER_FILES='/usr/share/vulkan/icd.d/'+icd,VK_ICD_FILENAMES='/usr/share/vulkan/icd.d/'+icd)
            for policy in ['legacy','checked']:
                name=driver+'-performance-'+policy
                run(name,['/usr/bin/time','-v','-o',str(out/(name+'.time')),str(root/'target/debug/examples/native_gui_performance'),'20','100',policy])
    receipt['binary_hashes']={str(p.relative_to(root)):hashlib.sha256(p.read_bytes()).hexdigest() for p in (root/'target/debug/examples').glob('native_*') if p.is_file() and p.suffix==''}
after=hashes();save('source-after.json',after);receipt['source_bookends_equal']=before==after;save('receipt.json',receipt)
print(json.dumps({'finished':True,'source_bookends_equal':before==after}),flush=True)

sys.exit(0 if all(item.get("exit")==0 for item in receipt["runs"]) and receipt.get("two_process_host_bytes_equal") and before==after else 1)
