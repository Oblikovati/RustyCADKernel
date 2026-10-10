"""Diagnose native cross-driver variation without changing the strict acceptance verdict."""
import hashlib,json,struct,sys
from pathlib import Path
receipt=Path(sys.argv[1]); destination=Path(sys.argv[2])
report=json.loads(receipt.read_text())
profiles={row['profile']:{case['case']:case for case in row['cases']} for row in report['profiles']}
rows=[]
for label,host in profiles['host'].items():
 if '-pers-' not in label: continue
 soft=profiles['lavapipe'][label]
 prefixes=[host['native_capture'],soft['native_capture'],host['ported_capture'],soft['ported_capture']]
 colors=[Path(prefix+'.rgba').read_bytes() for prefix in prefixes]
 depths=[struct.unpack(f'{len(color)//4}f',Path(prefix+'.depth').read_bytes()) for prefix,color in zip(prefixes,colors)]
 native_color_differences=0; outside_colors=0; outside_depths=0; native_max_depth=0.; software_reference_error=0.
 for index in range(len(colors[0])//4):
  pixels=[rgba[index*4:index*4+4] for rgba in colors]
  native_color_differences += pixels[0]!=pixels[1]
  if pixels[0]==pixels[1]: native_max_depth=max(native_max_depth,abs(depths[0][index]-depths[1][index]))
  if pixels[2]==pixels[1]: software_reference_error=max(software_reference_error,abs(depths[2][index]-depths[1][index]))
  for actual in [2,3]:
   candidates=[depths[reference][index] for reference in [0,1] if pixels[actual]==pixels[reference]]
   outside_colors += not candidates
   outside_depths += bool(candidates) and not min(candidates)-1e-6<=depths[actual][index]<=max(candidates)+1e-6
 rows.append({'case':label,'native_color_mismatch_pixels':native_color_differences,'native_max_common_color_depth_difference':native_max_depth,'host_vulkan_max_common_color_depth_error_vs_software_native':software_reference_error,'vulkan_colors_absent_from_native_results':outside_colors,'vulkan_depths_outside_matching_color_native_range':outside_depths,'capture_sha256':{prefix+suffix:hashlib.sha256(Path(prefix+suffix).read_bytes()).hexdigest() for prefix in prefixes for suffix in ['.rgba','.depth']}})
result={'scope':'Diagnostic only. Original exact-RGBA / 1e-6 depth gate remains unchanged and failures are retained. Compare both Vulkan outputs with the observed per-pixel native depth range, conditioned on matching native RGBA. This does not prove all host differences are benign or waive acceptance.','receipt_sha256':hashlib.sha256(receipt.read_bytes()).hexdigest(),'cases':rows,'native_cases_with_color_variation':sum(r['native_color_mismatch_pixels']>0 for r in rows),'native_cases_exceeding_strict_depth_tolerance':sum(r['native_max_common_color_depth_difference']>1e-6 for r in rows),'vulkan_colors_absent_from_native_results':sum(r['vulkan_colors_absent_from_native_results'] for r in rows),'vulkan_depths_outside_matching_color_native_range':sum(r['vulkan_depths_outside_matching_color_native_range'] for r in rows)}
destination.write_text(json.dumps(result,indent=2)+'\n')
print(json.dumps({key:value for key,value in result.items() if key!='cases'}))
