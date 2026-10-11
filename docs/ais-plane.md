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

## Verification and remaining renderer work

`cargo test -p rustycad-tkv3d --test ais_plane` checks construction, setters,
geometry caches, selection replacement, appearance ownership, minimum persistence,
manual-frame thresholds and actual shaded quadrangle submissions. The TKV3d
numeric corpus adds 210 exact request sequences spanning three geometry frames,
seven constructor/XYZ setup variants, linked and detached drawer preconditions,
and all defined state operations.

Vulkan quadrangle lowering, AIS plane view captures and plane-specific performance
workloads on both lavapipe and the native host GPU are still pending. Numeric
submission parity does not establish renderer replacement equivalence. These
remain the next renderer work before progressing to unrelated porting.
