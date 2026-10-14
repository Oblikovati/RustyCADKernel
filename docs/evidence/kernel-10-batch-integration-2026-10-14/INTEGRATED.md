# Kernel #10 paused-batch integration

Tested source: `0bae73e4958710a5b7d5f66d8fa57372d7ced610` on local `main`.
Candidate commits: conformance `482e6fbc2`, measures `d2909a9e0`, history helpers
`59348feee`, ordered validation `a41b53c61`, projection refusal `cf196e9fe`.
The prior complete-owned operation was separately integrated and checked at
`5b7366168`; its evidence and cleanup are retained in the adjacent integration folder.

## Executed combined checks

- **5,384 tests and 89 doctests passed** across TKMath and TKBO, with all
  targets/features and locked dependencies on Rust 1.99.0.
- Strict all-target Clippy, workspace formatting and warning-denied documentation passed.
- **16,451 source/configuration hashes** match across the complete matrix
  and exact-binary replay checks. All 238 candidate-file comparisons
  are accounted for: exact candidate bytes, seven reviewed registration unions,
  or four unchanged-codec visibility/reexport facades from the projection candidate.
- Every prior-main and candidate test remains, qualified by package, target source
  path and full Rust test name: prior_main 2,112; conformance 2,106; measures 5,294; ordered_validation 2,121; history_helpers 2,065; owned_projection_refusal 2,006. These are overlapping revision totals,
  not additive coverage or distinct capability credit.
- **70 fresh processes reproduced 35 full replay records** byte for byte against
  their retained candidate/earlier baselines, using the exact TKBO binary from the
  combined TKMath/TKBO feature graph. Its hash is in `root-replays.json`.
  The additional TKBO-only run is retained separately in `tkbo-only-replays/`.

The full matrix includes the exhaustive 16,536 complete-operation cancellation/
native-failure paths, resolver publication checkpoints, 1,279 annular measure
cutoffs, and the new conformance/ordered-validation failure corpora. Complete
payload comparisons include 26,813,998-byte projection refusals, 9,583,467-byte
owned result, 803,816-byte outer trace, 4,264,018-byte annular validator,
1,208,718-byte annular conformance/tolerance and 537,713-byte annular measures.

`integrated-main.json` records commands/platform/results; `test-retention.json`
records target-qualified preservation. `source-origin-audit.json`, retained diffs
and `merge-resolution-notes.json` record integration decisions. Candidate artifact
seals are independently checked in `candidate-artifact-integrity.json`.

## Scope and remaining gates

This batch supplies private bounded conformance, conditional measures, ordered
validation and owned projection-refusal evidence. **Complete annular history is
unimplemented**: its branch contains shared V1 helpers and actual native census
fixtures only. The future proof is detailed in its retained `HANDOFF.md`.

No annular successful publication, physical-volume certificate, complete owned
request/outcome/error boundary, general analytic profile acceptance, persistent
consumer edit/undo/reopen/display workflow, or release approval is claimed.
Ordinary thread/process tests do not close ThreadSanitizer: the prior host startup
failure remains incomplete and was not bypassed or retried unchanged. Accepted-
workload performance and exact owner package/release gates remain open.

The user requested finishing this batch and a handoff; broader work remains paused.
`cleanup-*.json` records deletion of each completed worktree and merged branch
following clean/ancestor/active-task process checks. Protected long-lived login,
credential and memory services are explicitly recorded as audit exclusions, with
no access-policy change. Four older unfinished trees and the user's local files
remain preserved. No new porting task, PR, push or release was performed.
