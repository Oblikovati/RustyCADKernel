"""Literal ADR0042/0045 vocabulary and actual one-boundary occurrence counts; no Rust imports."""
import struct,json,pathlib,re,hashlib,sys
roles={1:[1,2],9:list(range(16,34))+[5,6]}
for stage in range(2,9): roles[stage]=list(range(3,8))
for stage in range(4,9): roles[stage]+=list(range(8,16))
for stage in [4,5]: roles[stage]+=[34,35]
counts={1:{1:1,2:1},9:{5:6,6:6,16:1,17:1,18:1,19:1,20:1,21:1,22:1,23:1,24:1,25:1,26:1,27:1,28:3,29:3,30:2,31:2,32:1,33:1}}
for stage in range(2,9): counts[stage]={3:1,4:2,5:1,6:1,7:1}
for stage in [4,5]: counts[stage].update({34:1,35:1})
for stage in [6,8]: counts[stage].update({8:1,10:1,12:1,14:1})
counts[7].update({9:1,11:1,13:1,15:1})
rows=[];encoded=bytearray()
for stage in sorted(roles):
 for role in sorted(roles[stage]):
  count=counts[stage].get(role,0)
  if count:
   status={'applied_named_fields':count}; suffix=bytes([1])+struct.pack('<Q',count)
  else:
   reason=1 if stage in [4,5] else 3 if stage==7 else 2
   assert (stage in [4,5] and 8<=role<=15) or (stage in [6,8] and role in [9,11,13,15]) or (stage==7 and role in [8,10,12,14])
   status={'inapplicable_reason':reason};suffix=bytes([2,reason])
  rows.append({'stage':stage,'role':role,**status});encoded+=bytes([stage,role])+suffix
assert len(rows)==101
assert sum('applied_named_fields' in r for r in rows)==73
assert sum(r.get('applied_named_fields',0) for r in rows)==96
log=pathlib.Path(sys.argv[1]).read_text()
payload=bytes.fromhex(re.search(r'actual-semantic-resolver-v2=([0-9a-f]+)',log)[1])
header=b'Martensite.SemanticReservationCoverage\0'+struct.pack('<I',1)
assert payload.startswith(header)
maplen=struct.unpack_from('<Q',payload,len(header))[0]
start=len(header)+8+maplen
assert struct.unpack_from('<Q',payload,start)[0]==101
assert payload[start+8:]==encoded
result={'oracle':'literal semantic vocabulary and independent physical/directed incidence counts','rows':rows,'total_named_fields':96,'payload_bytes':len(payload),'payload_sha256':hashlib.sha256(payload).hexdigest(),'row_bytes':len(encoded),'row_sha256':hashlib.sha256(encoded).hexdigest()}
pathlib.Path(sys.argv[2]).write_text(json.dumps(result,indent=2)+'\n')
print('Independent exact101rows /73applied /28inapplicable /96namedfields; payload',len(payload))
