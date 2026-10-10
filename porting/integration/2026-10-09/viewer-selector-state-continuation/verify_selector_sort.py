"""Source-bound failure-prefix audit; not the full clean-checkout TKernel runtime gate."""
import hashlib,json,os,re,subprocess,sys
from pathlib import Path
sys.path.insert(0,'tools')
from legacy_runtime_evidence import runtime_input_hashes,runtime_git_state
root=Path.cwd();out=root/'target/oracle-selector-sort';out.mkdir(exist_ok=True,parents=True)
initial=runtime_input_hashes(root)
commands=[['g++','-std=c++17','-O2','tools/oracle/stl_sort_failure_audit.cxx','-o',str(out/'native-audit')],[str(out/'native-audit')],['cargo','test','--locked','-p','rustycad-tkernel','--test','stl_algorithms']]
outputs=[]
for index,command in enumerate(commands):
 result=subprocess.run(command,capture_output=True,text=True,timeout=180)
 (out/f'command-{index}.log').write_text(result.stdout+result.stderr)
 assert result.returncode==0,(command,result.stderr[-2000:]);outputs.append(result.stdout)
text=(root/'crates/tkernel/tests/stl_algorithms.rs').read_text().split('fn fallible_sort_preserves_every_native_comparison_failure_prefix()',1)[1]
expected=[' '.join([count,'1' if adversarial=='true' else '0',comparisons,fingerprint]) for count,adversarial,comparisons,fingerprint in re.findall(r'\((\d+), (false|true), (\d+), (\d+)\)',text)]
assert len(expected)==8 and outputs[1].splitlines()==expected,(outputs[1],expected)
assert '4 passed; 0 failed' in outputs[2]
assert initial==runtime_input_hashes(root),'source changed during audit'
report={'kind':'focused-differential','scope':'Fallible libstdc++ sorting; eight shapes check every possible comparison failure prefix and successful path, including heap fallback. Not full TKernel runtime acceptance.','cases_passed':8,'failure_or_success_trials':sum(int(line.split()[2])+1 for line in expected),'rust_tests_passed':4,'source_hashes':initial,'rust_git_state':runtime_git_state(root),'commands':commands,'native_audit':outputs[1].splitlines(),'native_audit_sha256':hashlib.sha256(outputs[1].encode()).hexdigest(),'verifier_sha256':hashlib.sha256(Path(__file__).read_bytes()).hexdigest()}
(out/'report.json').write_text(json.dumps(report,indent=2)+'\n');print('Focused STL audit passed:',report['failure_or_success_trials'],'failure/success trials across eight shapes; four Rust tests')
