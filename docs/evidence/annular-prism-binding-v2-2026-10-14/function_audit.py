import re,pathlib,json
root=pathlib.Path('crates/tkbo/src/sketch_region_prism/region_certificate/annular_binding')
result=[]
paths = list(root.glob('*.rs'))
paths += [root.parent / name for name in ['two_loop/mapped_parents.rs', 'two_loop/mapped_geometry.rs', 'two_loop/mapped_parent_tests.rs', 'current_binding/charts.rs']]
paths += [root.parent.parent / 'frozen/complete_history' / name for name in ['bounds.rs', 'bounds_tests.rs']]
for path in sorted(paths):
 s=path.read_text()
 # Preserve offsets/newlines while masking strings and comments, including escaped quotes.
 masked=re.sub(r'//[^\n]*|/\*[\s\S]*?\*/|"(?:\\.|[^"\\])*"',lambda m:re.sub(r'[^\n]',' ',m.group()),s)
 for match in re.finditer(r'\bfn\s+(\w+)',masked):
  start=masked.find('{',match.end()); depth=1;i=start+1
  while depth and i<len(masked):
   depth += (masked[i]=='{')-(masked[i]=='}'); i+=1
  first=s[:start].count('\n')+1;last=s[:i].count('\n')+1;body=last-first-1
  if body>20:result.append((str(path.name),match.group(1),first,last,body))
print(json.dumps(result,indent=2))
print('oversized',len(result))
