# Martensite clipping implementation in progress

Worktree branch: `porting/martensite-clipping`, with main through `2b0ecc12a` integrated.
This work is not yet integrated and adds no API inventory credit.

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
- Vulkan Clippy and eight Python harness test modules pass.
- Release measurements include `clipping-plane-updates`, with twenty warmups and
  one hundred samples per device. These are collected timings, not an accepted
  regression baseline. All existing workloads remain present.

The paired native oracle now contains 27 clipping modes across six camera,
viewport and structure-offset cases on both devices. The focused 26-mode run
passed 288/312 comparisons: exact RGBA and depth within the unchanged 1e-6
threshold. The 24 failures are visible-text cases; a new unclipped text control
separates glyph policy from the existing font raster differences. Those failures
remain strict failures. A full 2,448-case matrix and release timings are pending
against the final native-compatible culling policy.

Before integration: finish the full matrix, inspect the corrected captures,
verify the text control, compare the existing accepted performance budgets and
independently repeat the new workload; record final source/device receipts. Capping remains part of the renderer completion scope. No capability
family is marked complete by this checkpoint.
