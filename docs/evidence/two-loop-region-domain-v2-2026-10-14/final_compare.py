from pathlib import Path
import gzip,re,json,hashlib
root=Path('docs/evidence/two-loop-region-domain-v2-2026-10-14');temp=Path('/tmp/martensite-two-loop-region-domain');rows={}
commands=json.loads((temp/'post-style-commands.json').read_text())
for i,command in enumerate(commands):
 name=command['command'][6];old=gzip.decompress((root/('post-style-'+name+'.log.gz')).read_bytes());new=(temp/('final-replay-'+str(i)+'.log')).read_bytes()
 if i==0:
  regex=rb'\{"operation":"martensite.region-certificate"[^\n]*\}'
  before=set(re.findall(regex,old));after=set(re.findall(regex,new));payloads=before
 else:
  regex=rb'(?<![0-9a-f])[0-9a-f]{600,}(?![0-9a-f])'; before=set(re.findall(regex,old));after=set(re.findall(regex,new));payloads={bytes.fromhex(x.decode()) for x in before}
 assert before and before==after,name
 rows[name]=[{'bytes':len(x),'sha256':hashlib.sha256(x).hexdigest()} for x in sorted(payloads)]
expected=gzip.decompress((root/'two-loop-draft-v2.bin.gz').read_bytes())
for i in range(2):
 raw=(temp/('final-v2-process-'+str(i)+'.log')).read_bytes();matches=re.findall(rb'two-loop-region-domain-v2:([0-9a-f]+)',raw);assert len(matches)==1
 actual=bytes.fromhex(matches[0].decode());assert actual==expected
 rows['v2-process-'+str(i)]={'bytes':len(actual),'sha256':hashlib.sha256(actual).hexdigest()}
(root/'final-replay-comparison.json').write_text(json.dumps(rows,indent=2)+'\n');print('all complete replay payloads unchanged')
