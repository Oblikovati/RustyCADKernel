from pathlib import Path
import hashlib,json,os,subprocess,time,sys
if len(sys.argv) != 2: raise SystemExit('expected one new output directory argument')
root=Path.cwd(); dest=root/Path(sys.argv[1]); dest.mkdir(parents=True); env=os.environ.copy()
base_env={'DISPLAY':':1','VK_LAYER_PATH':str(Path('/home/vmiguel/git/oblikovati-workspace/RustyCADKernel')/'target/vulkan-validation/extracted/usr/share/vulkan/explicit_layer.d'),'LD_LIBRARY_PATH':str(Path('/home/vmiguel/git/oblikovati-workspace/RustyCADKernel')/'target/vulkan-validation/extracted/usr/lib/x86_64-linux-gnu')}
env.update(base_env)
executables={name:root/'target/debug/examples'/name for name in ['native_gui_rapid','native_gui_acceptance','native_window_lifecycle']}
(dest/'executables-before.json').write_text(json.dumps({name:hashlib.sha256(path.read_bytes()).hexdigest() for name,path in executables.items()},indent=2)+'\n')
checks=[(f'amd-rapid-{i}','radeon','native_gui_rapid') for i in range(5)]+[(f'lavapipe-rapid-{i}','lvp','native_gui_rapid') for i in range(2)]+[('amd-acceptance','radeon','native_gui_acceptance'),('lavapipe-acceptance','lvp','native_gui_acceptance')]
checks += [(f'{driver}-lifecycle-{i}',icd,'native_window_lifecycle') for driver,icd in [('amd','radeon'),('lavapipe','lvp')] for i in range(3)]
runs=[]
for name,icd,example in checks:
    out=dest/name;out.mkdir()
    icd_path=f'/usr/share/vulkan/icd.d/{icd}_icd.json'
    env.update(VK_DRIVER_FILES=icd_path,VK_ICD_FILENAMES=icd_path)
    cmd=[str(executables[example])]
    if example != 'native_window_lifecycle': cmd.append(str(out if example=='native_gui_rapid' else out/'overlay'))
    start=time.monotonic()
    with (out/'run.log').open('w') as log:
        try:
            result=subprocess.run(cmd,env=env,stdout=log,stderr=subprocess.STDOUT,timeout=45)
            code=result.returncode
        except subprocess.TimeoutExpired:
            log.write('\nTimed out after 45 seconds.\n');code=124
    record={'name':name,'command':cmd,'environment':{**base_env,'VK_DRIVER_FILES':icd_path,'VK_ICD_FILENAMES':icd_path},'exit':code,'wall_seconds_not_performance':time.monotonic()-start}
    runs.append(record);(dest/'runs.json').write_text(json.dumps(runs,indent=2)+'\n')
    print(json.dumps(record),flush=True)
    if code:
        raise SystemExit(code)
