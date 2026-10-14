import hashlib,json,os,platform,subprocess,time
from pathlib import Path
root=Path('/home/vmiguel/git/oblikovati-workspace/Martensite-prism-embedding-theorem')
out=Path('/tmp/martensite-prism-embedding-frozen');out.mkdir(exist_ok=True)
os.chdir(root)
def git(*args): return subprocess.check_output(['git',*args],text=True).strip()
assert not git('status','--porcelain')
def snapshot():
    files=git('ls-files').splitlines()
    selected=[p for p in files if p.startswith(('crates/','.cargo/')) or Path(p).name in ('Cargo.toml','Cargo.lock','build.rs','rust-toolchain.toml')]
    return {p:hashlib.sha256(Path(p).read_bytes()).hexdigest() for p in selected if Path(p).is_file()}
before=snapshot();(out/'before.json').write_text(json.dumps(before,sort_keys=True,indent=2)+'\n')
cargo='/home/vmiguel/.cargo/bin/cargo';env=os.environ.copy();env['CARGO_BUILD_JOBS']='2'
base=[cargo,'+1.99.0'];packages=['-p','rustycad-tkmath','-p','rustycad-tkbo']
commands=[('tests',base+['test','--locked',*packages,'--all-features','--all-targets']),('docs',base+['test','--locked',*packages,'--all-features','--doc']),('clippy',base+['clippy','--locked',*packages,'--all-features','--all-targets','--','-D','warnings']),('fmt',base+['fmt','--all','--','--check'])]
report={'source':git('rev-parse','HEAD'),'base_revision':'83eccdbcc63c6a672df5e0c5facbdcca79df2c37','commands':[],'cargo_build_jobs':2,'platform':platform.platform(),'machine':platform.machine(),'rustc':subprocess.check_output(['/home/vmiguel/.cargo/bin/rustc','+1.99.0','-Vv'],text=True),'scope':'ADR0029 private polynomial trimmed-prism embedding version 1; no body coverage promotion','thread_sanitizer':'not run; recorded host startup limitation remains incomplete','input_files':len(before)}
for name,command in commands:
    start=time.monotonic()
    with (out/(name+'.log')).open('w') as log:
        result=subprocess.run(command,env=env,stdout=log,stderr=subprocess.STDOUT)
    report['commands'].append({'name':name,'command':command,'exit':result.returncode,'seconds':time.monotonic()-start})
    (out/'receipt.json').write_text(json.dumps(report,indent=2)+'\n')
    print(name,result.returncode,flush=True)
    if result.returncode: break
after=snapshot();(out/'after.json').write_text(json.dumps(after,sort_keys=True,indent=2)+'\n')
report['inputs_unchanged']=before==after
report['git_status_after']=git('status','--porcelain')
(out/'receipt.json').write_text(json.dumps(report,indent=2)+'\n')
assert before==after
assert all(row['exit']==0 for row in report['commands']) and len(report['commands'])==len(commands)
