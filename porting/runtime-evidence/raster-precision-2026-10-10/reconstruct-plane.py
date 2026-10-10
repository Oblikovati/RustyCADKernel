import json,struct,math,sys,hashlib
from pathlib import Path
folder=Path(sys.argv[1])
f32=lambda value:struct.unpack('f',struct.pack('f',value))[0]
source=json.loads(Path('porting/runtime-evidence/persistence-diagnostics-2026-10-10/matrix-probe.json').read_text())
matrix=json.loads(source['opengl_runtime'].splitlines()[0].split('=',1)[1])
vertices=[]
for x,y in [(-8.,-8.),(8.,-8.),(0.,8.)]:
 clip=[f32(f32(matrix[i]*x)+f32(matrix[4+i]*y)+matrix[12+i]) for i in range(4)]
 vertices.append([f32(32*f32(clip[0]+1)),f32(32*f32(1-clip[1])),f32(.5*f32(clip[2]+1))])
reports=[]
for bits in [8,10,12,16,24]:
 tri=[(round(x*2**bits)/2**bits,round(y*2**bits)/2**bits,z) for x,y,z in vertices]
 (x0,y0,z0),(x1,y1,z1),(x2,y2,z2)=tri
 det=(x1-x0)*(y2-y0)-(x2-x0)*(y1-y0)
 a=((z1-z0)*(y2-y0)-(z2-z0)*(y1-y0))/det
 b=((x1-x0)*(z2-z0)-(x2-x0)*(z1-z0))/det
 c=z0-a*x0-b*y0
 row={'fractional_bits':bits,'plane':[a,b,c]}
 for name in ['host/gl','host/vk','lavapipe/gl','lavapipe/vk']:
  profile,api=name.split('/')
  prefix=folder/profile/(api+'-64-64-0-0-pers-axial')
  colors=Path(str(prefix)+'.rgba').read_bytes();depths=struct.unpack('4096f',Path(str(prefix)+'.depth').read_bytes())
  errors=[abs(depths[i]-(a*(i%64+.5)+b*(i//64+.5)+c)) for i in range(4096) if colors[i*4:i*4+4]==bytes([255,0,0,255])]
  row[name]={'maximum_depth_error':max(errors),'mean_depth_error':sum(errors)/len(errors)}
 reports.append(row)
result={'scope':'Independent plane reconstruction for the unmodified 64-64-0-0-pers-axial captures; no gate changes', 'vertices':vertices,'comparisons':reports}
print(json.dumps(result,indent=2))
