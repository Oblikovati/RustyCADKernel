# Martensite nodal line lighting

Normal-bearing segments and polylines now use retained Graphic3d materials,
lights and live normal snapshots with Gouraud or Phong in Vulkan 1.3. Missing
normals and facet line shading retain OCCT's unlit fallback. Unsupported PBR
and translucent material requests remain explicit errors. The shared nodal path
continues to support point sprites without changing their descriptor slots.

OCCT `3d097a0328e71b826377d4814ab05ec3c3d23871` evaluates the front material
for lit lines and markers. Its Gouraud/Phong shaders do not install the unlit
stipple mask: a dashed aspect on a normal-bearing lit line produces solid
coverage. Martensite preserves this behavior. Ordinary unlit line patterns
remain unchanged. Line width and clipping continue to apply to lit segments.

Three retained-line tests cover live normal snapshots, Gouraud/Phong selection
and facet fallback. The public draw lighting test also rejects derivative-based
facet lighting on line topology. Six analytic GPU scenes exercise solid,
pattern-requested and clipped lines in both nodal algorithms alongside the
existing surface-lighting checks, with exact color/depth assertions and zero
Vulkan validation errors on lavapipe and the native AMD GPU.

The local `lit-lines` capture family contains ten modes across six camera,
window-size and translation configurations. It compares native OCCT OpenGL
against Martensite for normals, colors, clipping, shading fallback and live
normal/material/light mutations. Exact pixel and depth thresholds remain intact;
remaining endpoint and interpolation differences do not establish full renderer
parity and are retained in the integration evidence.

The `lit-line-normal-updates` release benchmark changes a normal and a light,
redraws through V3d, and includes scene extraction, Vulkan submission, waiting
and color/depth readback. Each device runs 20 warmup frames and 100 samples.
Timings are measured alongside all existing workloads; strict capture failures
keep the overall renderer baseline unaccepted.

```sh
cargo test -p martensite-vulkan --all-targets --locked
python3 tools/verify_vulkan.py --source ../OCCT --build target/occt-all-full \
  --case-family lit-lines --profiles lavapipe host \
  --host-icd /usr/share/vulkan/icd.d/radeon_icd.json \
  --samples 100 --warmup 20 --output target/vulkan-lit-lines/receipt.json
```

Native OCCT verification runs locally during porting. CI continues to run the
Rust tests, analytic GPU checks and release performance workloads. Captures,
source hashes, timings and open strict failures are recorded in
[the runtime evidence](../porting/runtime-evidence/lit-lines-2026-10-11/README.md).
