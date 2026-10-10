# Axis intersector continuation

The tested source is `/tmp/rustycad-resume`, branch `porting/resume-claude`, parent
`7fac5b5bf9ca8b35f7b33b8462dfa1a7943e14d5`. Git metadata is read-only in this
session; main has not advanced and the source changes remain uncommitted. Preserve
this checkout and the original Claude worktrees for later integration.

The [fresh inventory](METRICS.md) accounts for **50,985/76,041 APIs
(67.0%)** in the unchanged port scope. These are continuation
counts, not main counts. Done includes exclusions and does not establish full
behavioral parity. This follows the [triangular checkpoint](../triangular-frustum-continuation/README.md).

`SelectMgrAxisIntersector` now implements its selecting ray, protected linear
helpers, every overlap, clipping, copy/transform and JSON dump. It preserves native
initialization failures, identity-form reset, no-op configuration, output mutation
on misses and clipped hits, polygon plane-only interior, triangle fallbacks and
local-coordinate quadric depths. Reversed sphere root ranges retain construction
failure. The native inherited BaseIntersector runtime identity remains observable;
Rust handle projection uses a separate caster descriptor. DumpJson retains the
explicit AxisIntersector class name. Shared geometric getters now return Result
to propagate native axis initialization failures.

Verification passed: 5,219 TKernel and 125 TKV3d Rust tests, including three new
handle regression tests and ten direct axis tests; Clippy with warnings denied;
41,704 native comparisons (22,213 new); one native and one Rust mapped legacy case;
corpus wiring, formatting and diff checks. All 31 new mappings resolve uniquely.
The inventory captures all thirty Rust crates and has zero unresolved ledger rows.

[receipt.json](receipt.json) records counts and hashes.
[continuation.patch](continuation.patch) preserves all source changes relative to
the parent, including earlier recovered work, and passes reverse-application
checking. Generated inventory binaries are omitted and must be regenerated after
integration. The [runtime report](../../../runtime-evidence/tkv3d-axis-intersector-2026-10-09.json.gz)
is source-bound and was verified against the current tree. Historical reports are
preserved.

Next are RectangularFrustum, TriangularFrustumSet and the selecting-volume manager.
The original unfinished TKernel linked-slot refactor is untouched and unvalidated.
The port remains incomplete.
