# Martensite plane datums

`AisPlane` translates `AIS_Plane` at pinned OCCT `3d097a0`. It retains plane
geometry or an XYZ frame, supports explicit and automatically projected centers,
manual limits, native size/color ownership, minimum-size persistence, current-mode
edge widths and boundary/interior triangulation selection.

`SetSize` writes plane X/Y lengths and datum X/Y/X lengths, then schedules
presentation and selection updates. XYZ corners remain cached until Compute or
SetContext. XY/XZ/YZ geometry uses the placed frame for triangle points while the
stored component remains the corresponding global canonical plane. Unknown XYZ
categories retain their component and produce zero corner points.

Automatic frame computation projects the center onto the component. Explicit
limits use only the minimum point to compute `2.4 * abs(u/v)` lengths, replacing
lengths below `10 * Confusion` with 0.1. Selection ignores its mode, clears the
previous entities and stores a priority-10 owner with a fresh triangulation. It
uses cached geometry without recomputing the frame.

Native display acceptance permits only mode zero, but Compute also implements
mode one. Wireframes and rectangle selection use a plane translated to the center;
shaded rectangles deliberately use the original component and submit four native
quadrangle vertices. XYZ shading retains the closed polygon builder.

Size/color ownership creates default aspects when detaching from a linked drawer,
rather than copying its styles. UnsetColor retains shading and drawer colors and
restores all three datum colors from the inherited plane edge color. A manual
frame can replace a linked bold edge aspect with a default-width aspect during
SetSize, so its first computation may use width one despite current mode.

## Native unsafe construction

Pinned native constructors leave `myHasOwnSize` uninitialized. Martensite starts
it at false; oracle comparisons assign size before reading size ownership. A new
native drawer has null plane/datum aspects until callers install or link them.
The native XYZ constructor nevertheless calls ComputeFields immediately and
dereferences that null datum aspect. Martensite's XYZ constructor supplies a
safe default datum aspect. This constructor difference remains explicitly partial
in the API ledger. The defined XYZ path, SetAxis2Placement after drawer setup,
is compared against native OCCT. Detached native drawers must be reconfigured
before aspect-dependent operations after UnsetSize/UnsetColor release them.

## Verification

`cargo test -p rustycad-tkv3d --test ais_plane` checks construction, setters,
geometry caches, selection replacement, appearance ownership, minimum persistence,
manual-frame thresholds and actual shaded quadrangle submissions. The TKV3d
numeric corpus adds 210 exact request sequences spanning three geometry frames,
seven constructor/XYZ setup variants, linked and detached drawer preconditions,
and all defined state operations.

The Vulkan path now lowers quadrangles and quadrangle strips, retains quad corners
for the verified llvmpipe/radeonsi diagonal policies, and defers unordered blended
surfaces until opaque layers finish or depth must be cleared. Blended surfaces test
depth without writing it. Unlit color alpha stays independent of material opacity,
matching the native plane's material-only transparency and explicit shading-alpha
controls. Opaque framebuffer alpha remains unchanged by geometry.

The `ais-plane` capture family adds 17 real AIS plane modes and 11 quadrangle
controls, each at six projection/window/offset configurations. Full-view layer
controls assert visible occluder colors as well as comparing both engines. The
[renderer receipt](../porting/runtime-evidence/ais-plane-renderer-2026-10-11/summary.json)
retains 124/168 strict passes on lavapipe and 140/168 on AMD, with no runtime
process errors. All shaded plane and analytic transparency/layer controls pass.
Remaining wireframe, gradient rounding and depth differences keep failed statuses;
colored quad differences are at most one byte after device tessellation selection.
Native policies outside the two tested driver profiles remain unverified.

Two release workloads measure live wireframe resizing and transparent shaded
recomputation plus redraw/readback. Both pass the source-frozen repeat comparison
at median ratio 1.25 and p95 ratio 1.5. The complete 29-workload measurement retains
older AIS/window regressions, so no overall timing baseline is accepted. Raw
captures and both measurements are retained in the receipt directory. OIT, alpha
masks, transparent capping, transparent back-face distinction and translucent
line/point/text/highlight colors remain outside this implemented slice.

The new layer controls also exposed an `AspectGenId` intermediate overflow for the
valid viewer interval `[1, INT_MAX]`. Allocation now computes the remaining offset
before adding the lower bound; its failing-then-passing regression and native
allocation/reuse sequence are retained with the renderer evidence.
