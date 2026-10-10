# Rectangular frustum continuation

The tested source is `/tmp/rustycad-resume`, branch `porting/resume-claude`, parent
`7fac5b5bf9ca8b35f7b33b8462dfa1a7943e14d5`. Git metadata remains read-only;
main has not advanced and source changes are uncommitted. Preserve this checkout
and the original Claude worktrees for integration when Git writes are available.

The [fresh inventory](METRICS.md) accounts for **51,027/76,041 APIs
(67.1%)** in the unchanged port scope. These are continuation
counts, not main counts. Done includes exclusions and does not prove full behavioral
parity. This follows the [axis checkpoint](../axis-intersector-continuation/README.md).

`SelectMgrRectangularFrustum` and `SelectMgrSelectionRectangle` now implement
point/box construction, vertex and projection caches, copy/scale/transforms, finite
view-segment helpers, all overlaps and clipped picks, planar predicates, plane
output and dumps. Native BaseFrustum RTTI is preserved with separate Rust casters.
Shared triangle arithmetic and quadric helpers serve rectangular and axis selectors.

Native distinctions are retained: negative scales fail; scale one plus Identity
form returns an uninitialized default; Other-form numeric identity takes the normal
transform path. Copies await reconstruction. Point and quadric depths rescale after
transforms while box depths remain local. Plane picks only write depth on success;
clipped point and quadric hits retain output writes. GetPlanes replaces the output.
Mouse position uses sentinels outside Point mode; DetectedPoint requires Point mode.
Native coordinate-substitution degeneracies remain visible in planar predicates.
Null-camera deep native dumps are not invoked; the identity reset with null builder
is compared at dump depth zero. Null-builder Rust projections report failures.

Verification passed: **137 TKV3d Rust tests** (12 new), Clippy with warnings denied,
**73,127 native comparisons** (31,423 new), one native and one Rust mapped legacy
case, seven corpus-wiring tests, formatting and diff checks. All 42 new mappings
resolve uniquely; all thirty Rust crates were recaptured and the ledger has zero
unresolved rows. Earlier TKernel validation remains in the preceding checkpoint.

[receipt.json](receipt.json) records counts and hashes.
[continuation.patch](continuation.patch) preserves all source changes relative to
the parent and passes reverse-application checking. Generated inventory binaries
are omitted and must be regenerated after integration. The [runtime report](../../../runtime-evidence/tkv3d-rectangular-frustum-2026-10-09.json.gz)
is bound to the current source hashes. Historical reports are preserved.

[next-frustum-set-notes.md](next-frustum-set-notes.md) records the next concrete
selector's triangulation dependencies and native state contracts. The original
unfinished TKernel linked-slot refactor remains untouched and unvalidated.
The port is incomplete.
