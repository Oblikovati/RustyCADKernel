# Martensite retained clipping

This renderer slice adds no OCCT API inventory credit. Capping and complete legacy
rendering parity remain unfinished.

The new immutable `RenderClipping` submission carries world-space plane chains
and an affine model matrix. Fragment shaders preserve the pinned OCCT rule:
positive halfspaces form a union within a chain, and distinct chains intersect.
Separate clipped pipeline variants leave ordinary submissions free of descriptor
uploads. The owned storage descriptor implementation is shared with point sprites
and glyph atlases. Shader freshness checks cover all sixteen SPIR-V variants.

Ordinary Graphic3d views now collect live view and structure planes at redraw.
Local sequences can override global planes, including an empty override sequence.
Disabling a chain head removes that chain; child on/off flags do not change its
membership, matching `OpenGl_Clipping::add` at OCCT `3d097a0`. Plane and model changes
are copied into immutable submissions. Standard font draws retain the native
font shader's zero-plane policy. Persistent structures classify global planes at
world anchors and retain local fragment planes. Local group persistence uses the
same model-translation reset as native OpenGL. Native bounding-box classification
culls entire structures and disables planes whose positive halfspace contains the whole box.
Capping remains unimplemented and active cap passes return explicit errors.

The view now calls the ported Graphic3d layer culling traversal, initializes cached
frustum projections, and honors On/Off/NoUpdate. It uses existing Graphic3d BVH
invalidation: unconditional rebuilding changes native camera-persistence behavior.
Native probes confirmed that empty-bound shadow structures and recalculated
ordinary structures containing only persistent groups are culled. Submission tests
that inspect their geometry explicitly disable frustum culling.

Validated locally on 2026-10-10:

- Fifteen analytic triangle scenes per device, covering single planes, unions,
  intersections, model translation, empty volumes and repeated changes.
  Both lavapipe and AMD Radeon 8060S passed 61,440 color/depth pixels with no
  Vulkan validation errors.
- The existing eighteen offscreen/window acceptance checks passed on both devices.
- Immutable upload layout, invalid inputs, live equations, head flags, local/global
  override, structure/group model composition, and failed-capture tests pass.
- Vulkan Clippy and 35 Python harness tests pass.
- Release measurements include `clipping-plane-updates`, with twenty warmups and
  one hundred samples per device. Controlled repeats validate its timing baseline
  below. All existing workloads remain present.

The paired native oracle contains 27 clipping modes across six camera, viewport
and structure-offset cases on both devices. The full 2,448-case run passed 844/1,224
on lavapipe and 764/1,224 on AMD. Clipping passed 144/162 per device: exact RGBA and
depth within the unchanged 1e-6 threshold. The 18 clipping failures per device are
visible-text raster differences, including the unclipped control. All 24 glyph
policy controls pass: fragment planes preserve glyph coverage, while whole-structure
culling removes it. The raw strict failures remain in the
[full receipt](../porting/runtime-evidence/vulkan-clipping-full-2026-10-10.json).

The [buffer audit](../porting/runtime-evidence/vulkan-clipping-buffer-audit-2026-10-10.json)
compares all 1,062 previous cases per device byte for byte. Every native buffer is
unchanged; Martensite differs only in twelve text-plane cases per device, all now
passing. No previously passing comparison regressed. All eighteen runtime checks
and fifteen analytic clipping scenes (61,440 pixels) pass on each device without
validation errors. All renderer CPU tests, Clippy and 35 Python harness tests pass.

The full run passes all nine established performance budgets. The
[independent repeat](../porting/runtime-evidence/vulkan-clipping-repeat-2026-10-10.json)
retains timing failures: lavapipe AIS removal median/p95 and circle-update p95
exceeded existing limits. Its host profile used an explicit Radeon ICD rather
than the full run's default enumeration, so exact profile comparison is rejected;
the selected hardware is identical, but these records are not treated as a pass.
A fixed main/clipping/clipping/main comparison with matching profile configuration
is retained in the [controlled receipt](../porting/runtime-evidence/vulkan-clipping-controlled-performance-2026-10-10.json).
Unchanged main also exceeded surface/window budgets. Both software clipping runs
met the nine established limits; one host clipping run met them and the other had
circle-update p95 1.023 ms against a 1.016 ms limit. These noisy failures remain
recorded; the full strict performance gate is not claimed to pass every run.

The new clipping workload passes its original 1.25x median / 1.5x p95 limits in
both controlled repeats on both profiles. The initial median/p95 is 0.421/0.517 ms
on lavapipe and 0.304/0.393 ms on AMD; controlled repeats are 0.380/0.510 and
0.380/0.496 ms on software, 0.339/0.502 and 0.331/0.577 ms on AMD.
The [ten-workload baseline](../porting/runtime-evidence/vulkan-clipping-baseline-2026-10-10.json)
preserves all nine previous rows and adds only `clipping-plane-updates`. Eight
other workloads still lack accepted baselines. The raw failures and budgets have
not been replaced or relaxed.

The [post-merge main receipt](../porting/runtime-evidence/vulkan-clipping-main-2026-10-10.json)
confirms identical capture-source fingerprints, all renderer CPU tests, all sixteen
shader variants, eighteen runtime checks and fifteen analytic clipping scenes on
both devices. Its optimized measurements pass all ten accepted workload budgets;
the eight unmatched workloads remain explicit failures of the broader timing gate.

Capping remains part of the renderer completion scope. No capability family is
marked complete by this slice, and no OCCT API inventory credit is added.

The next capping step needs homogeneous vertex input: pinned
`OpenGl_CappingPlaneResource.cxx` uses twelve plane vertices, with `w=1` at the
origin and `w=0` for directions at infinity. Martensite currently rejects non-unit
position weights. Preserve the full weight through GPU transformation; native
`Graphic3d_Group::AddPrimitiveArray` deliberately uses raw xyz with `w=1` only
for its bounds. `OpenGl_CappingAlgo.cxx` then builds an odd/even stencil mask from
closed groups using the selected plane, draws the cap through the remaining
planes, and applies plane or object aspects. Stencil targets, chain filtering,
section appearance and those vertex semantics all remain in scope.
