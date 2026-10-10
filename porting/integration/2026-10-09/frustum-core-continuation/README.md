# Frustum core continuation

The tested tree is `/tmp/rustycad-resume`, branch `porting/resume-claude`, parent
`7fac5b5bf9ca8b35f7b33b8462dfa1a7943e14d5`. Git metadata remains read-only, so main
has not advanced and these changes remain uncommitted. Preserve this checkout and
the original Claude worktrees until integration and checks can finish.

The [fresh inventory](METRICS.md) accounts for **50,928/76,041 APIs
(67.0%)** in the unchanged port scope. These are continuation counts,
not main counts. Done includes exclusions and does not establish full behavioral
parity. This follows the [base frustum checkpoint](../base-frustum-continuation/README.md).

`SelectMgrFrustumCore<const N>` now implements the native template's separating-axis
and quadric tests, with inherited state supplied by `SelectMgrBaseFrustumBase`.
Fixed-length mutable slices expose the protected arrays without allowing callers
to resize them. The implementation preserves native normal-output timing, inside
flag updates, degenerate segment behavior, sphere side-plane tests, hollow-cylinder
checks, circle origin convention and exception paths. All native template API
members have mappings; concrete triangular and rectangular volumes remain open.

The draft archived in the preceding checkpoint was recovered, completed and
validated. The original Claude source remains untouched. Production JSON retains
pointer values; only the differential probe canonicalizes process-local builder
and camera addresses after checking their identities. Numerical values are compared
without tolerance.

Verification passed: 106 TKV3d Rust tests, including ten new behavior tests; Clippy
with warnings denied; 7,139 exact differential comparisons (4,620 new); one native
and one Rust mapped legacy test; corpus-wiring tool tests; formatting and diff checks.
All twenty new declaration mappings resolve uniquely, including paired state and
array accessor targets. Rustdoc inventory capture covers all thirty crates.

[receipt.json](receipt.json) records exact counts and hashes.
[continuation.patch](continuation.patch) preserves all source changes relative to
the parent, including earlier recovered work, and passes reverse-application checking.
Generated inventory binaries are omitted and must be regenerated after integration.
The [source-bound runtime report](../../../runtime-evidence/tkv3d-frustum-core-2026-10-09.json.gz)
was checked against the current source tree. Earlier reports remain historical.

Next is `SelectMgr_TriangularFrustum`: build/cache geometry, transformed copies,
all virtual overlap methods and append-only plane export. Native boundary polygon
queries always return false, unknown triangle sensitivity returns true, and the
sphere inside-pointer overload uses a bounding box while its pick overload uses
the sphere core. These behaviors need explicit comparison coverage. The original
unfinished TKernel linked-slot refactor remains untouched and unvalidated.
