# Base intersector continuation

Follow-up: [base frustum state and boundary intersection](../base-frustum-continuation/README.md).

The tested tree is `/tmp/rustycad-resume`, branch `porting/resume-claude`, parent
`7fac5b5bf9ca8b35f7b33b8462dfa1a7943e14d5`. Git metadata remains read-only, so main
has not advanced and this work is uncommitted. Preserve this checkout and the three
original Claude worktrees until integration and checks can finish.

The [fresh inventory](METRICS.md) accounts for **50,896/76,041 APIs
(66.9%)** in the unchanged port scope. These are continuation counts,
not main counts. Resolved declarations and reasoned exclusions are inventory
accounting; they are not a claim of complete behavioral parity.

This follows the [TKV3d/TKernel recovery](../v3d-continuation/README.md). Claude's
untracked `SelectMgr_BaseIntersector` is now compiled and exported as a trait over
embedded state, with required abstract geometry methods and concrete native default
bodies. The sphere, cylinder/cone and circle ray kernels live in a separate module.
The unfinished frustum and TKernel linked-slot refactor remain preserved in their
original worktrees. [Provenance](recovery-provenance.json) verifies the original
files are unchanged.

Native comparison found and fixed the default view-ray direction: OCCT 3d097a0
uses `gp_Dir()` (+X), while the unfinished port assumed +Z. The failing native and
Rust regression logs are retained here. Native `DynamicArray::Clear()` also retains
allocation by default; the recovered code now supplies its required Rust flag.

The 39 new mappings cover the base record, constructor, state and concrete/default
members, plus required abstract trait signatures. They do not claim implementations
of concrete selecting volumes. The 933 new differential cases cover defaults,
retained camera identity, sentinels, no-op setters, JSON, tangent/missed/inside/behind
rays, cap and hollow modes, cone taper direction, degenerate height, circle outlines
and output arguments left untouched after failure or a single crossing.

Verification passed: 87 TKV3d Rust tests, including eight new behavior tests;
Clippy with warnings denied; the full TKV3d gate with 1,487 numeric comparisons and
one native/Rust legacy case; the updated corpus wiring tests; cargo formatting and
diff checks. The complete 165-module tooling suite passed at the preceding
checkpoint; the corpus-specific module was rerun after these additions.

[receipt.json](receipt.json) records exact counts and hashes.
[continuation.patch](continuation.patch) preserves all source changes relative to
the parent, including earlier TKBool/TKernel/TKV3d recovery, and passes reverse
application checking. It excludes generated inventory binaries, which must be
regenerated after integration. The new source-bound runtime report is
[archived here](../../../runtime-evidence/tkv3d-intersector-2026-10-09.json.gz).
TKBool and TKService were validated at the preceding checkpoint; those reports
remain evidence for that earlier tree. This checkpoint does not complete the port.
