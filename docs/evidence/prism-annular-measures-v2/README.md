# Annular conditional measures: candidate evidence

Frozen matrix source: `50a287e78` (implementation/test source `179270636`).
Evidence-runner correction: `9fe4653e6`; no crate or Cargo input changed.
This branch is a private prerequisite under ADR0055, pending coordinating-agent
integration. It does not complete an annular body, public operation or release.

## Executed gate

Rust 1.99.0, `CARGO_BUILD_JOBS=2`, worktree-local target, platform/compiler details
and exact commands in [frozen/receipt.json](frozen/receipt.json).

- TKBO: 1,449 library + 646 integration + 2 example tests = **2,097**.
- TKMath: 243 library + 2,954 integration tests = **3,197**.
- Combined: **5,294 tests**, **89 doctests**, all passing.
- Strict all-target/all-feature Clippy, default formatter, and docs with warnings
  denied passed for both affected toolkits.
- All **50,919** original tracked input hashes matched after the complete matrix.
- The 19 focused annular measure tests are included in that matrix. They cover
  independent cap/side/total/signed-sweep bounds; 13 actual stored/traversal/
  reflection/extrusion variants; source-row reordering; requested/effective
  policy distinction; zero/exact/one-short work; terminal-child work; all 1,279
  cancellation checkpoints and all six phases; fresh getter mutation/drop
  isolation; immutable readers and concurrent operations.

## Complete process payloads

Two independent processes compared the entire payload for each row:

| Payload | Bytes | Additional comparison |
| --- | ---: | --- |
| Annular measures V2 | 537,713 | Exact initial V2 baseline, before orchestration extraction |
| Joined measures V1 | 193,495 | Exact prior V1 baseline, before shared codec/policy extraction |
| Named annular domain V2 | 536,690 | Independent process equality |
| Annular orientation V2 | 546,491 | Independent process equality |

The [initial](initial/receipt.json) and [codec](codec/structural.json) records
separate baseline pins, unchanged structural work and new behavior. Independent
scalar expectations and the exact Jacobian computation receipt are described in
[independent/README.md](independent/README.md). No OCCT computation was added.

## Preserved runner correction

The original full matrix passed, then the first replay **test** passed but its
parser refused the payload because Rust's single-thread harness prefixed the
line with the test name. The original runner, refusal, successful commands and
exact source bookends are retained in `frozen/`. Only the evidence parser was
corrected. `frozen/replay_only.py` reran the eight replay probes and compared
bookends, explicitly admitting only the runner edit and added supplement.
It did not repeat or replace the completed matrix. All crate hashes were unchanged.

## Limits and ownership

The scalar report consumes one sealed `AnnularFrozenDomain`, owns every named
face report and full input binding, and computes total area from exactly those
four enclosures. Its signed material sweep is **not physical nonnegative volume**.
Failure retains typed causes, original policy, completed partial reports and all
completed child work; partial evidence is not a successful body. TSan acceptance
remains incomplete due to the previously recorded host startup limitation; these
ordinary thread tests do not establish TSan acceptance. No storage redesign or
previous-release workload benchmark is claimed.

The coordinator must preserve already integrated module registrations and other
workers' newer code when merging this development dependency branch. The explicit
candidate crate manifest is `frozen/candidate-crate-files.json`; the complete
matrix input manifests retain all inherited dependency revisions and hashes.
