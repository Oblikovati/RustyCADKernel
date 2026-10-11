# Martensite lit-marker runtime evidence

Gouraud/Phong normal-bearing points now use live Graphic3d material/light state,
per-vertex colors, sprite coverage and clipping. The retained sources at
`d29dc4f0871dc88013604d60242ed2b48c01336e` match the renderer receipt's hashes.
CPU checks pass: 62 library, 3 surface-lighting, 3 marker-lighting and 2 capture
example tests; 56 renderer-tool tests and 2 palette tests pass. All-target Clippy
passes with warnings denied.

Both profiles complete 96 strict captures. Lavapipe passes 52 and fails 44;
native AMD passes 12 and fails 84. All 192 geometry/highlight role checks per
profile pass, and runtime acceptance reports zero Vulkan validation errors.
Strict point raster/depth differences remain failures; no tolerance changed.

A separate functional regression compares exact visible RGBA value counts for
all 30 lit cases per profile. All 60 match native OpenGL, including Gouraud,
Phong, per-vertex colors, clipping and a changed normal. It does not relocate
pixels or override any strict comparison. The initial native fixture accidentally
kept the common oracle's Unlit global view policy; the palette regression fails
for that error and passes after both lit fixtures explicitly select Phong.
`fixture-correction.json` and the red/green logs preserve this diagnosis.

Both profiles run 31 release frame workloads, with 20 warmup frames and 100
samples. An independent same-device/driver repeat passes every median/p95 budget
(1.25/1.5 ratios). The 1,024-point live-normal mutation/redraw/readback workload
has median/p95 0.669/0.821 ms on lavapipe and 0.417/0.564 ms on AMD.
This is not an accepted overall renderer baseline while strict captures fail.

`captures.tar.gz` contains generated RGBA/depth/images/diffs only. Extract it
and reproduce the functional palette regression using:

```sh
python3 verify_lit_palette.py --receipt lit-marker-renderer-integrated.json.gz \
  --captures captures --output palette-check.json
```

Regenerate the whole capture/performance matrix with the `point-cloud` command
in [the point-cloud documentation](../../../docs/ais-point-cloud.md).
The strict harness intentionally exits with failure until raster parity is complete.
The integration receipt records main checks, fresh inventory and deletion of the
completed worktree and branch. PBR markers, lit lines and remaining point material
alpha policies still need implementations.
