# Inventory refresh and Claude continuation

Latest continuation: [Viewer selector traversal and public picking with fresh inventory](selector-pick-continuation/README.md).

This is the initial checkpoint. The [evolved continuation](evolved-continuation/README.md)
records subsequent implementation, recovery and validation work.

The refreshed main snapshot is bound to `bf6164bbd`, which includes the TKExpress
and TKFillet merges. Its source, ledger, artifact and tool fingerprints are current.
The pinned OCCT source inventory also matches. The previous published snapshot
predated both merges.

The generated [metrics](../../api-inventory/METRICS.md) account for 48,335 of 76,041
in-scope API declarations in the tool's Done category, 973 partial declarations and
26,733 open gaps. Done includes resolved implementation claims, exclusions and
upstream unimplemented declarations; it does not establish behavioral parity.
TKExpress accounts for 520/570 and TKFillet for 1,617/1,973 declarations. The two
unresolved ledger rows are the protected CellFilter hasher operators. Claude's
unmerged TKernel branch contains their accounting correction.

## Preserved Claude work

All three original worktrees and their uncommitted edits remain in place under
`.claude/worktrees/`. Inspection used the repository commits and source files,
not an imported conversation transcript.

| Worktree / branch suffix | Committed tip | Remaining work |
| --- | --- | --- |
| `agent-af067ae554ada5e1e` / `worktree-agent-af067ae554ada5e1e` | `a208fdaa3` | 24 unmerged TKBool commits through BRepAlgo_Loop; unfinished BRepFill_Evolved in nine untracked modules and a modified module export |
| `agent-aa64fda07fced1190` / `worktree-agent-aa64fda07fced1190` | `a99dc168a` | 11 unmerged TKV3d commits through Select3D/SelectMgr; untracked BaseIntersector and Frustum implementation files |
| `agent-aeed1901b085eca95` / `worktree-agent-aeed1901b085eca95` | `d61890682` | Two unmerged commits for ordered-cursor reset and CellFilter accounting; unfinished linked-slot/list refactor with staged deletions and modified/untracked files |

The unfinished edits were inspected but not compiled or merged. No validation
claim is made for them. Their original commit messages retain Claude session and
co-author provenance.

## Tested TKBool continuation

`porting/resume-claude` at `/tmp/rustycad-resume` starts from current main and merges
the committed TKBool tip as `7fac5b5bf`. Conflicts in CI, coverage replay and toolkit
verification were resolved by retaining each toolkit's registration. The original
dirty TKBool worktree was not used as the build source.

The separately generated [continuation metrics](continuation-metrics.md) account
for 49,468 in-scope declarations, including 1,156/2,704 for TKBool. All 12 new
explorer mappings resolve, with no new ledger defects. Its source, ledger,
artifact and tool fingerprints are current; it includes the uncommitted source
changes and must not be represented as main.

The continuation adds `TopOpeBRepDSCurveExplorer` and 12 exact declaration mappings.
It preserves sparse bindings, Keep filtering, current/indexed fallback distinctions,
live source edits through an exclusive borrow, the initial traversal limit, and
the count operation's restart/refresh/exhaustion behavior. Nine Rust tests cover
these contracts. The deterministic native corpus adds 255 cases.

The final numeric gate passes 1,381 exact comparisons, with no NaN representation
differences. TKBool's existing Rust suite passes 126 tests with eight pre-existing
ignored documentation examples; the new explorer's nine tests pass separately.
Clippy passes for TKBool and all its targets, Cargo formatting passes, and all
164 porting-tool test modules pass.

The full toolkit verifier currently runs zero mapped TKBool GoogleTest declarations.
Three translations already exist in the committed branch but are absent from the
ledger: the two BRepFill_PipeShell cases and
`TopOpeBRepDS_BuildToolTest.CopyReversedPeriodicEdgePreservesRange`. Registering them
also requires splitting the mixed native/port-only Rust test targets because the
verifier requires an exact target-to-ledger match. Fourteen other upstream TKBool
declarations remain unmapped. These gaps are not counted as native GTest passes.

The [runtime report](../../runtime-evidence/tkbool-resume-2026-10-09.json.gz) retains
the input source hashes and library hashes; all source hashes were checked against
the continuation tree after execution. [receipt.json](receipt.json) identifies the
merge commit, uncommitted state and evidence hashes. [curve-explorer.patch](curve-explorer.patch)
preserves the new implementation, tests, oracle and mappings relative to `7fac5b5bf`.

## Integration still required

After the user resumed the session, the filesystem policy made `.git` read-only.
`git add` failed while creating `.git/worktrees/rustycad-resume/index.lock`. Therefore
the new work and refreshed inventories are uncommitted, and the TKBool merge has
not advanced main. The continuation branch and worktree must remain until writable
Git metadata permits committing, merging directly into main, and checking the
integrated tree. No worktrees or branches were deleted.

The next algorithm dependency remains the unfinished TopOpeBRepBuild split/merge/
section/HBuilder work; the committed Builder currently has its constructor,
accessors and initial section/classification state. This still blocks the full 3D
fillet/chamfer builders described in `docs/tkfillet-progress.md`.
