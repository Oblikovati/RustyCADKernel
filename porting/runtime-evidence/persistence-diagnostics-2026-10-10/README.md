# Martensite persistence raster diagnosis

Upstream OCCT: `3d097a0328e71b826377d4814ab05ec3c3d23871`.
The strict loop renders `64-64-0-0-pers-axial` through the native and Martensite
viewer paths on the host, then calls `vulkan_capture_compare.compare_captures`.
The result is red: one pixel at (43, 25) differs and 1,171 common red pixels have
maximum depth difference `1.4007091522216797e-6`. No tolerance was changed.
Reproduce through the complete command in `docs/vulkan-renderer.md`; its per-case
results and raw buffers isolate this scene. Main captures remain under
`target/vulkan-persistence-main-captures/host` after worktree deletion.

## Findings and probes

- The native fixture used `Aspect_POM_None`, which inherits a layer offset.
  Martensite explicitly uses `Aspect_POM_Off`. Matching the fixture to Off removes
  the approximately 0.00131 sloped-depth bias. Lavapipe then passes all 240 scenes.
- `matrix-probe.json` records bit-identical float32 CPU clip matrices.
  `bound-uniforms.json` reads actual native shader projection/world-view/model
  uniforms; they match the expected persistence inputs.
- `separate-matrices.json`: multiplying separate matrices in the vertex shader
  instead of passing a CPU-combined matrix does not change the failing scene.
- `positive-viewport.json`: positive viewport height with reversed capture rows
  does not change it.
- `identical-shader.json`: replacing the native vertex shader with the same
  single-MVP expression does not change it.
- `depth-clip-control.json`: enabling `VK_EXT_depth_clip_control`, with negative
  one-to-one clip depth and no shader depth conversion, does not change it.
- `precise.json`: disabling contraction of the vertex matrix multiplication
  does not change it, including the common-color depth error above.
- The native FBO requests D24/stencil, while Martensite and Oblikovati use D32.
  `depth-format.json` and `depth-stencil.json` record explicit rejection of both
  X8_D24 and D24_S8 by this RADV device's attachment format feature query.
  These are unsupported probes, not passing tests or proof of the cause.

Every experimental renderer/shader/oracle modification was removed. Production
keeps the original D32 attachment, negative viewport and shader depth conversion.
Only the native fixture's explicit polygon-offset correction remains.

`native-variation.json` compares the full corrected matrix across native OpenGL
and Vulkan on both profiles. `compare-native-profiles.py RECEIPT OUTPUT` reproduces
that diagnostic from a receipt whose capture paths still exist. It conditions
depth bounds on matching RGBA. Two pixels vary between native OpenGL drivers and
both Vulkan outputs match one native color. However, 2,238 Vulkan depth pixels
fall outside the matching-color native range plus the original 1e-6 tolerance.
This rules out treating all failures as observed native-driver variation.

The cause of those remaining depth differences is unresolved. These diagnostics
are not an acceptance waiver. The default harness continues to fail the 34 host
persistence cases. Existing 126 non-persistence cases per profile still pass.
Raw paths in historical probes point to the deleted worktree; the committed
receipts retain their results, and the final main matrix retains current images.
No compiled probes or build caches are archived.

## Performance

The six existing workload baselines and 1.25x median / 1.5x p95 limits are unchanged.
The seventh workload measures 64 independently persistent groups with a changing
camera, 20 warmup and 100 measured completed frames. Its raw distributions are
retained in the matrix receipts. It intentionally has no accepted baseline while
native functional parity is unresolved: the gate reports an unmatched workload.
Collection is not a regression pass.

Oblikovati's `head/internal/native/viewport.cpp` was inspected read-only as a
reference: it uses D32 depth. No changes were made to that repository.
