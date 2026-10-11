# Martensite nodal marker lighting

Martensite now applies the existing world-space material/light snapshots to
normal-bearing point arrays. The retained Graphic3d extraction path reads live
normals, resolves the aspect/view shading model and selects Gouraud or Phong.
Points without normals and facet shading models remain Unlit, matching
`OpenGl_ShaderManager::ChooseMarkerShadingModel` in pinned OCCT `3d097a0`.
PBR markers and lit lines still report explicit unsupported requests.

Lit markers preserve screen-space size, nearest-sampled sprite coverage, the
native alpha discard threshold and per-view clipping. Lighting uses descriptor
set 0, clipping retains set 1, and marker pixels use set 2; ordinary sprites keep
set 0. The same GLSL lighting functions serve surfaces and markers, and the same
sprite sampler serves lit and Unlit markers. All variants target Vulkan 1.3.

The public `PrimitiveDraw::with_lighting` API accepts Gouraud/Phong markers with
one normal per vertex, rejects derivative-based facet marker shading, and
continues to reject lighting on stencil masks and unsupported textured triangles.
Live normal edits produce new snapshots without mutating earlier extracted draws.
Material transparency policies remain a separate renderer prerequisite; native
OCCT binds its marker program with `Graphic3d_AlphaMode_Opaque`.

The `point-cloud` capture family includes paired native/Vulkan fixtures for
Gouraud, Phong, alternating nodal normals, per-vertex colors, clipping and normal
updates. Its 16 modes run in six camera/window/translation configurations on
lavapipe and the native AMD GPU. Geometry/highlight role assertions accompany
strict RGBA/depth comparisons; existing raster differences stay visible.

The `lit-point-normal-updates` release benchmark redraws 1,024 normal-bearing
colored points with a live normal mutation and production readback on each frame.
It runs with 20 warmup frames and 100 timing samples alongside the complete
31-workload matrix. Driver/device identity and raw timing distributions are
recorded for both profiles.

Run CPU checks with `cargo test -p martensite-vulkan` and GPU capture/performance
checks with the documented `point-cloud` command in [point clouds](ais-point-cloud.md).
Runtime outcomes, source hashes and retained failures are recorded in
[the integration evidence](../porting/runtime-evidence/lit-markers-2026-10-11/README.md).
