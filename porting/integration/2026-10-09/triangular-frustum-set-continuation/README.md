# Triangular frustum set continuation

The tested source is `/tmp/rustycad-resume`, branch `porting/resume-claude`, parent
`7fac5b5bf9ca8b35f7b33b8462dfa1a7943e14d5`. Git metadata remains read-only;
main has not advanced and source changes are uncommitted. Preserve this checkout
and the original Claude worktrees until integration is possible.

The [fresh inventory](METRICS.md) accounts for **51,053/76,041 APIs
(67.1%)** in the unchanged port scope. These are continuation
counts, not main counts. Done includes exclusions and does not prove full behavioral
parity. This follows the [rectangular checkpoint](../rectangular-frustum-continuation/README.md).

`SelectMgrTriangularFrustumSet` and `SelectMgrSelectionPolyline` now implement
polyline initialization, native Delaunay decomposition and child order, all overlaps,
full-containment boundary rejection, shared input-array copies, transformed volumes,
plane output and dumps. TKMesh and TKGeomAlgo are direct crate dependencies through
their existing owned interfaces. Circle boundaries use actual GeomIntIntSS plane
intersection and line-normal arithmetic.

Native distinctions remain observable: copied sets share mutable pixel arrays but
not built geometry; transforms reset overlap permission while copies retain it;
unbuilt copies retain empty child geometry and boundary arrays. Point containment
calls child SAT directly. Box inside flags refine containment while the method still
returns true for partial overlap. Sphere, cylinder and circle containment retain
native projection/indexing, failures and flag mutations. DetectedPoint always raises.
Runtime identity is inherited BaseFrustum, independently of the explicit dump class.

Verification passed: **149 TKV3d Rust tests** (12 new), Clippy with warnings denied,
**91,290 native comparisons** (18,163 new), one native and one Rust mapped legacy
case, seven corpus-wiring tests, formatting and diff checks. All 26 new mappings
resolve uniquely; all thirty Rust crates were recaptured with zero unresolved rows.
The expanded native cases include concave/reversed/repeated/short polylines, copies
before and after rebuild, shared-array mutation, affine/collapsed transforms, both
camera modes, circle radius and boundary failures, all dump depths and seeded picks.

[receipt.json](receipt.json) records counts and hashes.
[continuation.patch](continuation.patch) preserves all source changes relative to
the parent and passes reverse-application checking. Generated inventory binaries
are omitted and must be regenerated after integration. The [runtime report](../../../runtime-evidence/tkv3d-frustum-set-2026-10-09.json.gz)
is bound to the current source hashes. Historical reports are preserved.

[next-selecting-volume-notes.md](next-selecting-volume-notes.md) records the next
manager's interface and clipping requirements. The original unfinished TKernel
linked-slot refactor remains untouched and unvalidated. The port is incomplete.
