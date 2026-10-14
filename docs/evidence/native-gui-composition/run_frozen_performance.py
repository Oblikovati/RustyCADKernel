from pathlib import Path
import os,subprocess,json,hashlib,time,datetime
root=Path.cwd();proof=root/'target/gui-composition-proof';freeze=json.loads((proof/'performance-freeze.json').read_text());sources=json.loads((proof/'frozen-source-sha256.json').read_text());records=[];observations=[]

def hashes():
 changed=[name for name,sha in sources.items() if hashlib.sha256((root/name).read_bytes()).hexdigest()!=sha]
 binaries={name:hashlib.sha256(Path(freeze[name]).read_bytes()).hexdigest() for name in ['binary','acceptance_binary']}
 assert not changed,changed
 assert binaries['binary']==freeze['binary_sha256']
 assert binaries['acceptance_binary']==freeze['acceptance_binary_sha256']
 external=json.loads((proof/'driver-validation-sha256.json').read_text())
 assert all(hashlib.sha256(Path(name).read_bytes()).hexdigest()==sha for name,sha in external.items())
 return {'utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'sources_verified':len(sources),'binaries':binaries,'driver_validation_files_verified':len(external)}

def external_heavy():
 found=[]
 for directory in Path('/proc').iterdir():
  if not directory.name.isdigit():continue
  try:
   exe=Path((directory/'exe').readlink()).name;argv=[v.decode(errors='replace') for v in (directory/'cmdline').read_bytes().split(b'\0') if v];cwd=str((directory/'cwd').readlink());comm=(directory/'comm').read_text().strip()
  except OSError:continue
  heavy=exe in ['rustc','cargo','cargo-llvm-cov','cargo-nextest','rust-lld','cc','gcc','clang','clang++','llvm-profdata'] or 'performance' in comm or (argv and 'target/' in argv[0] and ('/examples/' in argv[0] or '/deps/' in argv[0]))
  if heavy and cwd!=str(root):found.append({'pid':int(directory.name),'exe':exe,'cwd':cwd,'argv_prefix':argv[:4]})
 return found

def run(profile,phase,env,argv,log):
 with log.open('wb') as output:
  process=subprocess.Popen(argv,cwd=root,env=env,stdout=output,stderr=subprocess.STDOUT)
  while process.poll() is None:
   observations.append({'utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'profile':profile,'phase':phase,'external_heavy':external_heavy()})
   time.sleep(0.5)
  records.append({'profile':profile,'phase':phase,'argv':argv,'environment_override':freeze['profiles'][profile],'cwd':str(root),'log':str(log.relative_to(root)),'exit_code':process.returncode})
  (proof/'performance-commands.json').write_text(json.dumps(records,indent=2)+'\n')
  (proof/'performance-process-observations.json').write_text(json.dumps(observations,indent=2)+'\n')
  assert process.returncode==0,(profile,phase,process.returncode)

bookends=[hashes()]
for profile in ['lavapipe','amd']:
 env=os.environ.copy()
 for key in ['VK_INSTANCE_LAYERS','VK_LAYER_PATH','VK_ADD_LAYER_PATH','VK_ICD_FILENAMES','GALLIUM_OVERRIDE_CPU_CAPS','LP_NATIVE_VECTOR_WIDTH','LIBGL_ALWAYS_SOFTWARE']:env.pop(key,None)
 env.update(freeze['profiles'][profile]);env['DISPLAY']=':1';env['LC_ALL']='C'
 wrapper=['python3','tools/with_vulkan_validation.py','--cache','/home/vmiguel/git/oblikovati-workspace/RustyCADKernel/target/vulkan-validation','--',freeze['acceptance_binary']]
 for phase in ['pre','timing','post']:
  before=external_heavy();observations.append({'utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'profile':profile,'phase':phase+'-boundary','external_heavy':before})
  assert not before,before
  if phase=='timing':argv=['/usr/bin/time','-v','-o',str(proof/f'performance-{profile}-rss.log'),freeze['binary'],'20','100']
  else:argv=wrapper+[str(proof/f'{profile}-{phase}')]
  run(profile,phase,env,argv,proof/f'performance-{profile}-{phase}.log')
  bookends.append(hashes())
  (proof/'performance-hash-bookends.json').write_text(json.dumps(bookends,indent=2)+'\n')
print(json.dumps({'exit_code':0,'contaminated':any(r['external_heavy'] for r in observations),'commands':len(records),'hash_bookends':len(bookends)}))
