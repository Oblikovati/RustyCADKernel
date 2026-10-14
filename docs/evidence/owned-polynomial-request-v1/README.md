# Owned polynomial request V1: preparation checkpoint

This is Martensite Kernel #10 prerequisite evidence for ADR0045. The new private
entry owns scalar input and drives the existing polynomial region/prism pipeline.
It returns a prepared diagnostic stack, **not a completed or published body**.
The semantic correspondence resolver and complete publication adapter are pending.

## Revisions and executed checks

- Dependency merge: `8a891a633`, preserving reviewed publication `3e8bb7dc3` ancestry.
- Behavior source: `21e74305d`.
- Structural follow-up/final source: `06a9f7e7142266295a83084fa75678336c6c80ff`.
  This only boxes the retained census error and adapts its two construction sites;
  the exact diagnostic payload, classification and callback sequence remain intact.
- Rust 1.99.0, Linux x86_64, two Cargo build jobs. Exact compiler/platform details
  and commands are in `receipt.json` and the archive's `platform.json`.
- Focused preparation plus existing naming tests: **34 passed** (20 new, 14 existing).
- Full `sketch_region_prism` library filter: **628 passed** before boxing, then
  **628 passed** after boxing, including existing publication/history/validation,
  geometry, measurement and replay fixtures.
- Strict Clippy for TKBO, all targets and features: passed after boxing.
- Formatting check: passed.
- Separate final cancellation sweep: **3,696 observed checkpoints**, cancellation
  at each exact prefix, unchanged owned request values, and successful fresh reuse.
- All **16,055** tracked crate/manifest files have identical before/after hashes
  for both committed-source prism runs. No source changes occurred during them.

The exact invocation uses `/home/vmiguel/.cargo/bin/rustup run 1.99.0 cargo`;
`receipt.json` uses the equivalent compact `cargo +1.99.0` notation. The gate is
focused on the affected prism module; it is not the full workspace release gate.

## Independent expectations and retained refusals

The owned fixture supplies literal world-space cubic poles, an explicit plane,
seed/extrusion, distinct native endpoint IDs and all policies. It does not call
existing geometry or naming fixture orchestration. Tests retain the constructor's
actual shared vertex and Forward edge: native parameters are exactly 0/1, while
querying against the reversed closed edge gives 1/0. Native caller IDs never swap.
Generated cases reverse stored poles, directed traversal and extrusion separately,
and translate the supporting plane. Constant/tangent cases and outside/boundary
seeds cannot produce a prepared certificate. Empty input declines; multiple input
boundaries explicitly remain outside this entry's admitted domain.

Isolation checks compare actual captured curve coefficients against owned inputs,
retain previous preparation after later source-buffer edits, and re-run changed
geometry even when the caller revision is unchanged. The off-plane edit retains
exact legacy `ValidationRejected(Geometry)` as Failed. Callback failures preserve
the typed callback cause separately from a child cancellation diagnostic. Private
preparation and failure state have compile-time Send assertions; owned request
scalars have Send+Sync assertions. These assertions are not a concurrency proof.

## Failed attempts preserved

The archive retains the following without rewriting them as successful runs:

1. `first-owned-tests.log`: 25 passed, one test incorrectly expected native 0/1
   while querying the reversed closed edge. The corrected test asserts both actual
   contexts explicitly; the constructor now retains the Forward native edge.
2. `expanded-owned-tests.log`: 32 passed, one test incorrectly expected Incomplete
   for an off-plane edit. `same-revision-*.log` and the red source tar show the
   actual exact Geometry rejection. The final test expects that precise failure.
3. `complete-preparation-tests.log`: compile error E0308 from borrowed coordinate
   arrays in the new finite-plane admission check; fixed by copying their values.
4. `preparation-clippy.log`: 11 `result_large_err` diagnostics caused by the retained
   census payload. The separate structural commit resolved them without suppressions.

Earlier development logs are identified as development attempts, not frozen-source
acceptance. The two final committed-source runs have their own hash bookends.

## Artifacts and exclusions

`checks.tar.gz` contains complete logs, platform details, both source manifests,
revision files, and the preserved failed-attempt source snapshot. The separate
`candidate-source.tar.gz` retains the final owned-request module, unchanged naming
fixtures and ADR0045. `artifacts.json` records byte counts and SHA-256 hashes.
These committed artifacts are sufficient to preserve the evidence after worktree
and build-artifact cleanup.

No OCCT execution, new native oracle, tessellation acceptance, policy widening,
retry ladder or host-protection change was used. No new whole-preparation portable
receipt exists yet, so this evidence does not claim two-process equality for that
entry. Existing child replay fixtures ran unchanged. Runtime concurrency stress
and ThreadSanitizer for the eventual complete owned operation remain unexecuted.
Private mutable diagnostics are not the future owned refusal result. Public API,
multiple/hole profiles, application persistence/edit flows and Kernel #10 release
completion remain open.
