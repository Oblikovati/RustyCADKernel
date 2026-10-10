# TKBool continuation: explorers, interference construction and evolved sweeps

This checkpoint follows the [initial inventory and Claude audit](../README.md).
The tested tree is `/tmp/rustycad-resume`, branch `porting/resume-claude`, based on
merge commit `7fac5b5bf9ca8b35f7b33b8462dfa1a7943e14d5`. Main has not advanced.
The changes remain uncommitted because the session filesystem policy makes `.git`
read-only. Preserve this worktree and all original Claude worktrees until integration
and its checks can finish.

## Refreshed inventory

The [fresh metrics](METRICS.md) account for 49,502 of 76,041 port-scope API
declarations (65.1%), with 973 partial and 25,566 open. TKBool accounts for
1,190/2,704 (44.0%); all 34 new mappings resolve. The two pre-existing unresolved
CellFilter ledger rows remain. Source, ledger, artifact and tool fingerprints are
current. The scope and denominator are unchanged. Done includes reasoned exclusions
and upstream unimplemented declarations as well as resolved implementations; it is
an inventory measure, not full behavioral proof. These counts belong to the
continuation checkout, not main.

## Implemented and recovered work

- `TopOpeBRepDSPointExplorer`: twelve declaration mappings, six behavior tests and
  109 OCCT comparisons. Shares cursor traversal with the earlier CurveExplorer
  while preserving the point-specific failure after a live source is cleared.
- `TopOpeBRepDSInterferenceTool`: ten mappings, five behavior tests and 288 OCCT
  comparisons. Covers typed construction, retained handles, independent complemented
  copies, parameters, bound flags and configurations.
- `BRepFillEvolved`: twelve mappings, five behavior/regression tests and 67 OCCT
  comparisons. Recovers the nine untracked implementation files left by Claude in
  `agent-af067ae554ada5e1e` at `a208fdaa325fe755185783840457cc1039672aad`.
  The original files are untouched; [recovery-provenance.json](recovery-provenance.json)
  records their hashes. The recovered module is formatted and split into ten files.
  Coverage includes wire/face spines, default and performing constructors, caps,
  generated history, planar/vertical/general profiles, square/concave/circular
  spines, transformed profile axes and unsupported joins.
- Three existing GoogleTest translations now have exact ledger registrations and
  dedicated test targets: the two PipeShell solid cases and BuildTool's reversed
  periodic-edge copy case. Shared pipe-shell fixtures avoid duplicated test bodies.

## Diagnosed parity difference

Repeating the same inward square sweep produced four different OCCT traversal
sequences in six executions; Rust varied too. The evolved oracle therefore sorts
hash-dependent siblings and p-curve representations. It retains exact sampled
geometry, tolerances, flags, child incidence and unique topology counts. It accounts
for a free shell's arbitrary overall orientation and consumes a solid shell's
orientation into its face incidences. No floating-point tolerance was introduced.
After stabilization, the same input had one output in six runs on each side.

This exposed a real signed-zero difference in outward sweeps. `BRepFill_MultiLine`
used Rust floating-point min/max for UV bounds, losing the first operand on ties.
OCCT uses `std::min/max`. Reusing the project's existing C++-compatible helpers
preserves the negative-zero bound and resulting p-curve coordinate. The regression
failed before the fix and passes afterwards; its failing log is retained here.

## Verification and recovery

- Full TKBool Rust suite: 151 passed, zero failed; eight pre-existing ignored doctests.
- All TKBool Clippy targets: passed with warnings denied.
- Full toolkit gate: 1,845 numeric comparisons, zero NaN representation differences;
  three native and three Rust legacy runtime cases passed.
- All 164 porting-tool test modules passed. Cargo formatting and diff checks passed.
- Current inventory and verification results are recorded in [receipt.json](receipt.json).

The [runtime report](../../../runtime-evidence/tkbool-evolved-2026-10-09.json.gz)
binds the evidence to source and native-library hashes. Every source hash matched
the continuation tree after execution. [continuation.patch](continuation.patch)
preserves all new source, tests, probes and ledger changes relative to the merge
commit, including the earlier CurveExplorer. Applying it in reverse with `--check`
succeeded. It excludes generated inventory binaries, which must be regenerated
against the final integrated source tree.

This checkpoint does not establish full toolkit or kernel parity. The earlier
checkpoint's inventory and runtime evidence are historical and must not be used as
proof for this changed source tree. The original TKV3d and TKernel work remains
preserved and unintegrated. TopOpeBRepBuild split/merge/section/HBuilder remains a
major dependency for the unfinished 3D fillet/chamfer work.
