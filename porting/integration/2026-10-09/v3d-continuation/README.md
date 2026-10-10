# TKV3d and TKernel recovery checkpoint

Follow-up: [base intersector recovery and refreshed inventory](../intersector-continuation/README.md).

The tested continuation is `/tmp/rustycad-resume`, branch `porting/resume-claude`,
parent `7fac5b5bf9ca8b35f7b33b8462dfa1a7943e14d5`. Main has not advanced.
Git metadata is read-only in this session, so these changes cannot yet be committed
or merged. Keep the continuation and original Claude worktrees until integration
and checks finish.

The [fresh inventory](METRICS.md) accounts for **50,857/76,041 APIs (66.9%)**,
with 972 partial and 24,212 open. That is 1,355 more done declarations than the
[previous checkpoint](../evolved-continuation/README.md). Scope is unchanged.
These are continuation counts, including resolved implementation claims and reasoned
exclusions; they do not establish complete behavioral parity or describe main.

Recovered work includes Claude's eleven committed TKV3d changes and two committed
TKernel fixes. The TKV3d code adds V3d views/viewers, Prs3d presentation tools,
PrsMgr objects and selection foundations. The TKernel changes make foreign-map
cursor reset return a failure and correct the CellFilter ledger. The oracle caller
now handles the cursor-reset result. Shared configuration and ledger changes retain
the TKBool recovery as well as the visualization additions.

Two previously unexported selection helpers are now compiled, tested and mapped:
FrustumBuilder and ViewClipRange. Their 274 additional OCCT comparisons cover camera
projection modes, viewport invalidation, clipping-plane chains, range insertion
order and failure outputs. ViewClipRange exposes the native output mutation even
when a lookup fails. A pre-existing primitive-array test now has an exact legacy
GoogleTest registration and a dedicated Rust test target.

Verification passed:

- Combined TKBool, TKService and TKV3d Rust suites; four TKernel cursor test targets.
- Clippy with warnings denied for every TKernel, TKService and TKV3d target.
- TKV3d: 554 numeric comparisons and one native/Rust legacy case.
- TKService: 3,142 numeric comparisons and 72 native/Rust legacy cases.
- TKBool: 1,845 numeric comparisons and three native/Rust legacy cases.
- All 165 porting-tool test modules; fresh 30-crate rustdoc capture and inventory.

[receipt.json](receipt.json) records hashes and counts. Its three compressed runtime
reports bind all evidence to source hashes, verified against this checkout before
archival. [continuation.patch](continuation.patch) preserves the complete source,
test, probe and ledger delta from the parent commit; reverse application passed.
Generated inventory binaries are omitted from the patch and must be regenerated
when integrated. Prior runtime reports are historical for their own source trees.

The two untracked Claude base-intersector/frustum files and the unfinished TKernel
linked-slot refactor remain preserved in their original worktrees. Their provenance
is recorded in [recovery-provenance.json](recovery-provenance.json). The next work
continues the selecting-volume algorithms; this checkpoint is not kernel completion.
