# Triangular frustum continuation

The tested tree is `/tmp/rustycad-resume`, branch `porting/resume-claude`, parent
`7fac5b5bf9ca8b35f7b33b8462dfa1a7943e14d5`. Git metadata remains read-only, so main
has not advanced and these changes remain uncommitted. Preserve this checkout and
the original Claude worktrees until integration and checks can finish.

The [fresh inventory](METRICS.md) accounts for **50,954/76,041 APIs
(67.0%)** in the unchanged port scope. These are continuation counts,
not main counts. Done includes exclusions and does not establish full behavioral
parity. This follows the [frustum core checkpoint](../frustum-core-continuation/README.md).

`SelectMgrTriangularFrustum` now implements the concrete volume: stored selection
pixels, builder projection, native vertex/edge/plane ordering and cached extrema,
transformed copies, copies awaiting reconstruction, every overlap overload,
append-only plane equations and JSON dumps. Protected state is composed from the
existing template core and frustum base. Native type ancestry and handle casts work.

Init retains old geometry until Build. ScaleAndTransform ignores the scale and
builder arguments, keeps the camera and triangle, and starts with a fresh builder
and default tolerance. CopyWithBuilder copies pixels and adopts the builder but
leaves geometry empty until Build. Clear releases only the builder. Rust Build on
a null builder reports Standard_NullObject with its invalid state; native code
would dereference null for that invalid call.

Native overlap behavior is retained: pick fields and inside flags stay unchanged;
clip ranges are ignored; point-only overlap is false; the sphere inside-pointer
overload tests an enclosing box while its pick overload tests the sphere core.
Boundary polygon/triangle queries are false even when all edges overlap; unknown
triangle sensitivity is true. The probe compares reversed and degenerate pixels,
both camera and depth modes, viewports, affine/collapsed transforms, copied/rebuilt
and cleared volumes, all queries and dump depths. Only process-local pointer
addresses are canonicalized; numeric values are compared without tolerance.

Verification passed: 115 TKV3d Rust tests (nine new), Clippy with warnings denied,
19,491 native comparisons (12,352 new), one native and one Rust mapped legacy test,
corpus-wiring tool tests, formatting and diff checks. All 27 new API mappings resolve
uniquely: 26 increase the API Done count, while the destructor is counted separately
as boilerplate by the existing inventory rules. The inventory captures all thirty
Rust crates and has zero unresolved
ledger rows; full API coverage and behavioral parity remain unproven.

[receipt.json](receipt.json) records exact counts and hashes.
[continuation.patch](continuation.patch) preserves all source changes relative to
the parent, including earlier recovered work, and passes reverse-application checking.
Generated inventory binaries are omitted and must be regenerated after integration.
The [source-bound runtime report](../../../runtime-evidence/tkv3d-triangular-frustum-2026-10-09.json.gz)
was checked against the current source tree. Earlier reports remain historical.

The [next-axis source notes](next-axis-notes.md) record getter failure and inherited
RTTI constraints that must be preserved. The remaining concrete selectors are
AxisIntersector, RectangularFrustum and
TriangularFrustumSet, followed by the selecting-volume manager. AxisIntersector can
reuse the existing ray kernels but needs faithful clipping, pick-result mutations,
initialization assertions and the native identity-transform behavior (a fresh,
uninitialized default axis). The original unfinished TKernel linked-slot refactor
remains untouched and unvalidated. The port is incomplete.
