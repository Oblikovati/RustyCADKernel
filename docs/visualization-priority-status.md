# Visualization priority status

Retained evidence from the [porting plan](porting-plan.md#current-priorities).
Critical application blockers precede broader feature expansion. Keep these
regression gates and unfinished work.

Continue Martensite's Vulkan renderer through the existing Graphic3d interfaces.
Match the legacy OCCT OpenGL renderer's capabilities using Vulkan 1.3; keep
additional renderer features outside the current scope. Build a tested offscreen
rendering path before broadening unrelated porting. The initial offscreen path
now runs through V3d views and AIS display/erase presentations. AIS lifecycle,
bulk and selected-object transitions, removal, status queries and per-view affinity
have native comparisons. Structure/group transform persistence is integrated as
partial. The 34 native AMD capture differences are traced to rasterizer precision
([controlled evidence](vulkan-raster-precision.md)); retain the raw failures and
separate diagnostic gate before claiming bitwise parity or accepting its timing
baseline. Segments/polylines, widths and shader patterns now reach Vulkan;
[line evidence](vulkan-lines.md) records remaining gradient/depth parity gaps
and the new performance workload. The [standard curve presentation family](stdprs-curve.md)
now supplies all eight Add/Match overloads, native numeric comparisons and
GPU/performance fixtures. [AIS_Line](ais-line.md) now consumes this path, with
live appearance, unit-aware selection and GPU/performance coverage. User decision
on 2026-10-10: defer the two AMD infinite-line pixel-row differences as known
special cases; keep their evidence and continue feature work. [AIS_Point](ais-point.md)
now adds live point datums, native selection and a batched Vulkan sprite path
with functional and performance coverage. Point raster/color parity remains partial.
[Vulkan text](vulkan-text.md) now renders retained font glyphs and oriented labels;
[Shadow text](vulkan-shadow-text.md), [retained text face culling](vulkan-text-culling.md),
[Subtitle backgrounds](vulkan-subtitle-text.md), and [Decal outlines](vulkan-decal-text.md)
are integrated with native
fixtures, both-device captures and optimized workloads. Strict text raster
parity and broader text capabilities remain partial. [Blend/XOR](vulkan-blend-text.md)
adds real logic operations and explicit native pixel text, with quantified color
conversion/raster limitations retained.
[AIS_Axis and labeled axis presentations](ais-axis.md) now add coordinate datums,
native selection snapshots and inherited text-aspect rendering.
[Plane presentation builders](plane-presentations.md) now preserve native edges,
isolines, normal arrows and datum polygon submissions. [AIS plane datums](ais-plane.md)
add native geometry, cached triangulation selection, size/color ownership and minimum
persistence. Vulkan quadrangle lowering, unordered surface blending, device captures
and plane-update timings are integrated. Strict raster/depth and timing gates remain
open; the unsafe upstream XYZ constructor is explicitly partial.
[Plane trihedrons](ais-plane-trihedron.md) now add labeled XY frames, child-owner
selection, native comparisons and geometry-update performance coverage.
[Native X11 presentation](vulkan-window-presentation.md) now adds swapchains,
resize/rebind/remap checks, actual window captures and presentation timings.
[Native GUI composition](native-gui-composition.md) now attaches owned GUI layers
to that same view, with model/GUI alpha isolation, extent rejection and atomic
replacement. Both drivers pass native captures, source tests and extracted GUI
regressions; seven workloads per driver retain 1,400 measured samples. See the
[integration receipt](evidence/gui-integration-2026-10-14/receipt.json) for source
and performance revision boundaries. Release and application acceptance remain open.
[Circle datums and adaptive deflection curves](ais-circle.md) add circular/arc
construction geometry, filled/boundary selection and live geometry-update workloads.
[Retained clipping](vulkan-clipping.md) adds live plane chains, persistent-anchor
and bounding-box policy, view frustum culling, native capture controls and a
measured plane-update workload. [Homogeneous vertex input](vulkan-homogeneous-vertices.md)
preserves full position weights for the infinite planes needed by capping.
[Stencil capping](vulkan-capping.md) now builds closed-group masks and infinite
section surfaces, including native chain, instance and persistence policy.
Opaque Phong-family section materials now share live material/light extraction;
textures, custom section shaders and strict clipping-boundary parity remain open.
[Shader material records](vulkan-materials.md) now match native coefficient
packing. [Opaque Phong-family lighting](vulkan-lighting.md) now adds live normals,
directional/point/spot/head lights, front/back materials and clipped shaders;
strict native capture and performance gates remain open.
[AIS animation timelines](v3d-design.md#ais-animation-timelines) now provide
native child scheduling and timer lifecycle. [Camera animation](v3d-design.md#ais-camera-animation)
adds live camera interpolation and deferred redraw. [Object animation](v3d-design.md#ais-object-animation)
adds exact local transformation interpolation, context selection updates and
immediate-layer Z-fit invalidation with native and GPU/performance fixtures.
[Ordered group flipping](vulkan-group-flipping.md) now preserves annotation
brackets, live orientation, lighting/clipping transforms and native cap filtering.
[AIS text labels](ais-text-label.md) add native font-measured annotation placement,
sensitive rectangles, retained aspect changes and viewer persistence transitions.
[Point clouds](ais-point-cloud.md) now preserve aliased/copied buffers, cached
bounds, application geometry callbacks and forced subset-owner highlighting,
with native comparisons and both GPU profiles. [Nodal marker lighting](vulkan-lit-markers.md)
now supports Gouraud/Phong points, sprite coverage and clipping. Retain strict
point/line raster gates alongside live-normal update timings; continue remaining
point material/alpha policies. [Nodal line lighting](vulkan-lit-lines.md) now adds
Gouraud/Phong segments with clipping, native solid fallback for requested patterns,
live-normal snapshots and both-device functional/performance coverage. Strict
endpoint and color interpolation parity remains open.
[Translucent surface lighting](vulkan-lit-transparency.md) now combines the three
Phong-family algorithms with material opacity, distinguished back faces,
deferred pass ordering and live opaque/translucent transitions.
[Retained transparency](vulkan-oit-contract.md) now implements unordered blending,
weighted OIT and dual depth peeling with live parameters, context alpha state,
multisample fallback, layers and capping. Both-device functional replay and
35-workload performance measurements are recorded with the implementation.
Keep the documented texel, sprite, far-plane, high-sample and text driver
differences explicit; full raster parity and API completion are not claimed.
Continue remaining alpha-mask and cap/annotation policies alongside these
retained diagnostic gates.
[AIS trihedrons](ais-trihedron.md) now provide cached wire/shaded datum parts,
native owner selection and protected application subclass hooks. Vulkan polygon
offsets support the shaded geometry with both-device runtime checks; strict
shaded selection, image parity and performance-baseline gates remain open.
[Rubber-band overlays](ais-rubber-band.md) now preserve live screen-point sequences,
frontier-constrained polygon filling, reusable geometry and application subclass
hooks,
with paired native captures and polygon-update timings.
[AIS view input buffers](ais-view-input-buffer.md) now preserve all public gesture
records, native defaults and reset/copy semantics through committed oracle assets.
[AIS navigation dependencies](ais-view-navigation.md) now preserve camera policies,
mouse gestures and six-axis walking state with native boundary/flag/copy fixtures.
[AIS view controller input](ais-view-controller.md) now queues native selection,
zoom and Z rotation, with public policies and UI-only navigation cancellation.
[Keyboard navigation](ais-view-controller-keyboard.md) now preserves frame
timing, all six axes, stance/thrust/release semantics and focus-loss reset.
[Click and touch dispatch](ais-view-controller-pointer.md) now preserves animation
ownership, selection bindings and native touch transitions.
[Window-listener integration](ais-view-controller-window-listener.md) now exposes
the complete controller callback interface with typed mouse/touch failures.
[UI/render queue transfer](ais-view-controller-transfer.md) preserves pending
requests, allocator ownership and drag priority across frames.
[Camera frustums](ais-camera-frustum.md) add live geometry, appearance and both
selection modes. [Primitive-array initialization](sensitive-primitive-initialization.md)
now preserves all five overloads and native retained/failed initialization state.
[Light-source binding](ais-light-source-binding.md) adds properties and world/local
transforms. [Light presentations](ais-light-source-presentation.md) now add
concrete construction, geometry, selection, directional highlighting and dragging,
with native fixtures and both-device captures and optimized timings. Classes remain
partial. Vulkan Shadow-style light labels now render; strict named-light capture
differences and native-default-layer parity remain open.
[Color scales](ais-color-scale.md) now preserve native palette policies, labels,
layout and real presentation submissions. [Controller picking](ais-controller-picking.md)
and [selection-aware fitting](ais-view-controller-fit.md) now use real retained
geometry and cameras. [Touch synthesis](ais-view-controller-gestures.md) preserves
native contact-count transitions and integer pan boundaries.
[Primitive matching](sensitive-primitive-matching.md) now has native differential
and ownership/lifecycle fixtures; traversal regressions preserve callback ordering.
[Native point alpha](vulkan-point-alpha.md) adds native point/sprite alpha policies
and committed GPU oracle buffers. Exact-cutoff software differences and additional
sprite/material capabilities remain open.
[Native line alpha](vulkan-line-alpha.md) now preserves forced blending in draw
order with depth writes. Committed native buffers support Rust-only GPU replay;
strict software endpoint parity and timing-baseline acceptance remain open.
Continue its remaining dispatch alongside the remaining presentation
and legacy renderer capabilities
(see [current evidence](vulkan-renderer.md)).
Keep functional captures and performance regression workloads together on both
lavapipe and the native host GPU.
Advance TKService/TKV3d functionality needed by this path alongside it; Vulkan
work does not wait for every visualization API to be ported.
