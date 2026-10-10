"""Focused differential evidence; deliberately not the clean-checkout TKernel acceptance gate."""
import hashlib,json,os,subprocess,sys
from pathlib import Path
sys.path.insert(0,'tools')
from legacy_runtime_evidence import runtime_input_hashes,runtime_git_state
from porting import snapshot
from tkernel_osd_threads_corpus import cases
root=Path.cwd();source=Path('/home/vmiguel/git/oblikovati-workspace/OCCT')
build=Path('/home/vmiguel/git/oblikovati-workspace/RustyCADKernel/target/occt-all-full')
out=root/'target/oracle-bvh-osd-threads';out.mkdir(exist_ok=True,parents=True)
initial=runtime_input_hashes(root);revision=snapshot(source)['revision']
libs=list(build.rglob('libTKernel.so'));assert len(libs)==1
command=['g++','-std=c++17','-O2','-pthread','-I'+str(build/'include/opencascade'),'tools/oracle/tkernel_probe.cxx',str(libs[0]),'-Wl,-rpath,'+str(libs[0].parent),'-o',str(out/'native-probe')]
compiled=subprocess.run(command,capture_output=True,text=True);(out/'compile.log').write_text(compiled.stdout+compiled.stderr);assert compiled.returncode==0
built=subprocess.run(['cargo','build','--locked','-p','rustycad-tkernel','--example','oracle_probe'],capture_output=True,text=True);(out/'rust-build.log').write_text(built.stdout+built.stderr);assert built.returncode==0
requests='\n'.join(cases())+'\n';(out/'inputs.txt').write_text(requests)
outputs=[]
for name,executable in [('native',out/'native-probe'),('rust',Path(os.environ['CARGO_TARGET_DIR'])/'debug/examples/oracle_probe')]:
 result=subprocess.run([str(executable)],input=requests,capture_output=True,text=True,timeout=120)
 (out/(name+'.txt')).write_text(result.stdout);(out/(name+'-stderr.txt')).write_text(result.stderr)
 assert result.returncode==0,(name,result.returncode,result.stderr[-1000:]);outputs.append(result.stdout)
assert outputs[0]==outputs[1],'focused OSD thread differential mismatch'
assert len(outputs[0].splitlines())==len(cases())
assert initial==runtime_input_hashes(root),'source changed during validation'
report={'kind':'focused-differential','scope':'Existing complete OSD thread/pool/parallel corpus; not full TKernel acceptance','revision':revision,'cases_passed':len(cases()),'source_hashes':initial,'rust_git_state':runtime_git_state(root),'commands':[command,['cargo','build','--locked','-p','rustycad-tkernel','--example','oracle_probe']],'input_sha256':hashlib.sha256(requests.encode()).hexdigest(),'output_sha256':hashlib.sha256(outputs[0].encode()).hexdigest()}
(out/'report.json').write_text(json.dumps(report,indent=2)+'\n');print('Focused OSD thread differential:',len(cases()),'cases passed')
