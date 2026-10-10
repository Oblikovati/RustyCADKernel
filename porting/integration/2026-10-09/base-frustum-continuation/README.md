# Base frustum continuation

The tested tree is `/tmp/rustycad-resume`, branch `porting/resume-claude`, parent
`7fac5b5bf9ca8b35f7b33b8462dfa1a7943e14d5`. Git metadata remains read-only, so main
has not advanced and these changes remain uncommitted. Preserve this checkout and
the original Claude worktrees until integration and checks can finish.

The [fresh inventory](METRICS.md) accounts for **50,908/76,041 APIs
(66.9%)** in the unchanged port scope. These are continuation counts,
not main counts. Inventory accounting does not establish full behavioral parity.
This follows the [base intersector checkpoint](../intersector-continuation/README.md).

`SelectMgrBaseFrustum` now provides the shared builder/tolerance state and all native
base-frustum bodies. Concrete picking volumes can delegate their inherited virtual
setters to that state. Camera changes invalidate the viewport; replacing a builder
adopts its camera; removing the builder preserves the existing camera. Boundary
intersection retains closed-polygon indexing, projected-direction errors, unchanged
inside outputs after misses or crossings, and overridden sphere-ray behavior.

Native comparison confirmed the apparently tangent edge at y=1 can miss because
OCCT squares a rounded Distance() result. The port keeps that arithmetic. The JSON
probe uses a native readable string stream matching Rust DumpStream; opaque cout
has a different native separator rule. No output normalization or numeric tolerance
was introduced. A shared guarded fixture macro avoids duplicated abstract geometry
stubs in the base-intersector and base-frustum tests.

Verification passed: 96 TKV3d Rust tests, including nine new behavior tests; Clippy
with warnings denied; 2,519 exact differential comparisons (1,032 new); one native
and one Rust mapped legacy test; the corpus-wiring tool tests; cargo formatting and
diff checks. Twelve new declaration mappings cover the base class, constructor,
protected state and concrete methods. They do not claim concrete frustum algorithms.

[receipt.json](receipt.json) records exact counts and hashes.
[continuation.patch](continuation.patch) preserves all source changes relative to
the parent, including earlier recovered work, and passes reverse-application checking.
Generated inventory binaries are omitted and must be regenerated after integration.
The [source-bound runtime report](../../../runtime-evidence/tkv3d-base-frustum-2026-10-09.json.gz)
was checked against the current source tree. Earlier reports remain historical.

Preparation for the remaining SAT and quadric routines is in
`/tmp/rustycad-frustum-core-draft` and [this archive](frustum-core-unvalidated-draft.tar.gz).
These drafts are uncompiled and unvalidated and have no inventory claims. The archive's
STATUS.md lists required compilation, shape invariants, output-normal preservation,
DumpJson implementation and native tests for both triangular and rectangular cases.
The original Claude frustum and unfinished TKernel linked-slot refactor remain
untouched. The port is incomplete.

Follow-up: the archived draft has since been completed and validated in the
[frustum core continuation](../frustum-core-continuation/README.md). The evidence
above remains a historical snapshot of this earlier checkpoint.
