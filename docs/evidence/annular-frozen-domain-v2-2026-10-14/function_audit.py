import pathlib,re,json
roots=['crates/tkbo/src/sketch_region_prism/frozen/annular_join','crates/tkbo/src/sketch_region_prism/frozen/topology/annular','crates/tkbo/src/sketch_region_prism/frozen/annular_fixture']
result=[]
for root in roots:
 for p in pathlib.Path(root).glob('*.rs'):
  s=p.read_text();masked=re.sub(r'//[^\n]*|/\*[\s\S]*?\*/|"(?:\\.|[^"\\])*"',lambda m:re.sub(r'[^\n]',' ',m.group()),s)
  for m in re.finditer(r'\bfn\s+(\w+)',masked):
   start=masked.find('{',m.end());depth=1;i=start+1
   while depth and i<len(masked):depth+=(masked[i]=='{')-(masked[i]=='}');i+=1
   first=s[:start].count('\n')+1;last=s[:i].count('\n')+1;body=last-first-1
   if body>20:result.append((str(p),m.group(1),body))
print(json.dumps(result,indent=2))
