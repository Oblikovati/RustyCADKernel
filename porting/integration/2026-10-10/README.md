# Main integration and fresh inventory

Follow-up: the [selector camera-slot correction](../../../docs/viewer-selector-traversal.md)
is integrated at `a9a4d67de`, with 275 TKV3d tests and 132,815 native comparisons
passing. Its branch and worktree were deleted. The [compact receipt](../../runtime-evidence/selector-camera-slot-a9a4d67de.json)
records the source revision and refreshed inventory. The combined workspace
integration results below retain their original tested revision.

The restarted session permits Git writes. Completed Claude branches and the validated
continuation are integrated directly into `main`. The tested source revision is
`9cb1b33fa94c5b2ab618528ab04582cbd22c4858`; the later evidence commit changes only inventory and documentation.

The fresh [metrics](../../api-inventory/METRICS.md) account for **51,412 / 76,041**
in-scope API declarations as Done, with **975 partial**, **23,654 open**,
and zero unresolved ledger rows. Done includes resolved implementation claims,
reasoned exclusions and upstream unimplemented declarations; it is not a behavioral
parity percentage. Scope remains 56 toolkits. Full port completion remains false.

## Integration and cleanup

- `82e5c9436` integrates the validated continuation, including the recovered TKBool
  and TKV3d work and subsequent AIS context, traversal and picking ports.
- `66aab4589` integrates original Claude TKV3d history; all 1,333 original ledger
  keys remain. [Resolution audit](tkv3d-merge-resolutions.json) records the decisions.
- `a8bca4f3d` integrates the original TKernel cursor and CellFilter accounting fixes.
- `9cb1b33fa` integrates the CheckTool test-isolation regression fix.

All three original Claude committed tips and continuation tip `b9aef7146` are
ancestors of main. Their worktrees and merged branches are removed, as is the
completed CheckTool fix worktree. Original local edits remain in the hash-checked
[archives](claude-original-edits/manifest.json). Historical checkpoints retain
their original execution state; their read-only and unmerged notes are superseded
by this receipt.

## Validation

`cargo test --workspace --all-targets -j 2` passed **16,611 tests** across
**1,653 targets**, with 0 failures and 0 ignored tests.
TKernel passed 1,521 native legacy instances, 1,522 Rust runtime cases and
201,892 differential cases, covering 1,316 mapped declarations with one explicit
upstream skip and its separate Rust compatibility regression. TKV3d passed
132,707 differential cases and its one mapped native/Rust legacy pair.
Both runtime reports' input hashes match the tested main source.

The first workspace run exposed a race between CheckTool tests that change the
native process-wide error mode. Test subprocess isolation preserves production
semantics; the fix passed 1,000 parallel-pair repetitions, 20 whole-library
repetitions and the final workspace run. The affected packages passed formatting;
CheckTool library/tests also passed Clippy with warnings denied.

[receipt.json](receipt.json) binds report/log hashes, exact commits, cleanup and
inventory counts. Runtime evidence is retained under `porting/runtime-evidence/`.
The compiler inventory captures all 30 configured Rust crates at the tested revision.

## Preserved unfinished work

`porting/linked-slot-recovery` at `7f9efad84a7237773ee1856c4ca96a2222268f89` remains in
`/tmp/rustycad-linked-slot-recovery`. It preserves all 25 original TKernel changed
or deleted files byte-for-byte and still has **39 compile errors**. Deleted base
list/link-node modules remain imported, linked slots are not registered, and
removed exports and changed List interfaces require deliberate recovery. It is
not integrated. Main and this recovery checkout are the only remaining worktrees.
The receipt also lists pre-existing WIP branches without worktrees; those are
preserved and are not claimed as completed integrations.

## Next porting work

The selector class remains partial. `ToPixMap` requires SelectionImageFiller and
its color/depth/normal modes; DisplaySensitive requires sensitive presentation
construction. Before claiming exhaustive callback parity, add a native regression
for replacing the original intersector camera handle during Matches: native
traversal retains a reference to that slot, while the current Rust frame captures
a handle. See [traversal details](../../../docs/viewer-selector-traversal.md).
