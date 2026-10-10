# Martensite clipping implementation in progress

Worktree branch: `porting/martensite-clipping`, based on `e364a3738`.
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
font shader's zero-plane policy. Capping, persistent clipping and native bounding
box interactions still need implementation or native verification; active capping
and persistence combinations currently return explicit errors.

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

Before integration: extend the pinned OCCT capture oracle and full image/depth
matrix; cover clipping of lines and points, local/global persistence and font/bound
interactions; inspect the captures; compare the existing accepted performance
budgets and independently repeat the new workload; record final source/device
receipts. Capping remains part of the renderer completion scope. No capability
family is marked complete by this checkpoint.
