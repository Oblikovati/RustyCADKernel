# Local application acceptance handoff

Frozen failed candidate: `bb1e00115cd8855da4a40e2092b570a2120ab657`, tree
`30e3bf46f1d49d1aa4f42f8e11d5e8b941da6a9e`. Checkout remained tracked-clean.
Artifact export contains evidence and existing package/source assets only;
it is not an archived checkout. No tag, version change or publication occurred.
Names ending in `0.2.5` are unpublished local candidate assets, not immutable
assets of the already-existing release tag.

## Completed gates on bb1e

| Gate | Actual result | Receipt |
|---|---|---|
| Extracted packages, pinned VVL | 31 archives / 64 cargo commands / 230 executable or doc groups / 22,626 pass / 0 fail / 111 declared ignored | `application-prerequisites/extracted-root-audit.json` |
| All-feature instrumented coverage | 18,078 pass / 0 fail / 6 skipped | `application-prerequisites/coverage-root-audit.json` |
| Workspace Clippy all-target/all-feature | exit 0 | `application-owner-rust/gates/source-receipt.json` |
| Workspace docs all-feature | exit 0, RUSTDOCFLAGS=-D warnings | same source receipt |
| Tooling | 225 modules pass / 0 failed; no invented individual count | `application-prerequisites/tools-owner/receipt.json` |
| Foundation porting and dependencies | both exit 0 | `application-prerequisites/owner-foundation-receipt.json` |
| Lavapipe Vulkan all-target | 644 pass / 0 fail / 5 ignored / 71 groups | `application-owner-rust/gates/gpu/gpu-owner-failure-receipt.json` |

Extracted scope was default all-target plus doctests for all 31 packages, and
all-feature all-target plus doctests for TKMath. It was not an all-feature
extracted run for every package. Coverage excludes the existing CI case
`bvh_bug30655::BVH_PairDistanceTest__Bug30655_SphereDistance`; the functional
package run retained it. LCOV is preserved verbatim at
`application-prerequisites/lcov.info`, SHA256
`8fbd02c1b6d8fb08688d5d9fcc5faad0dde0d73f9d8bcee995572ee8f585efff`.
The initial unpinned package failure and later pinned success are both retained;
`receipt.json` must not be replaced by `receipt-pinned.json`.

## Actual bb1e failure

Original lavapipe `offscreen_acceptance` exited 101 at point_alpha_goldens.rs:319:
`received empty camera-persistent points; expected visible depth`.
Lighting and every host GPU gate stopped and were not run on bb1e.
No accepted renderer performance run belongs to this candidate.
Original binary is preserved at
`application-owner-rust/debug/examples/offscreen_acceptance`, SHA256
`e7c56b6ea95432d32b71d1092c4aba77a329f35deb8d544e4b19c5c9ede1a5e6`.
Original device, source snapshots, library/validation snapshots and logs are in
`application-owner-rust/gates/gpu/`. Do not relabel this candidate green.

## Corrected source evidence, separate revision

Feature commit `248978e42` integrated as `81ec965e809e94c8f31b2cb33be57329986b9405`;
completion receipt followed at local main `2e577787cba2c30c97c7e7cc99989d8b02a78e81`.
`corrected-source-81ec965e8/` preserves the root integration logs/receipt.
Both device all-target runs passed 648 tests (5 ignored); both actual offscreen
examples passed 20 runtime checks and both lighting examples passed 42 scenes /
172,032 analytic pixels, with zero validation errors. No performance was run.
The root receipt's result array records the five sequential follow-up commands;
its separate lavapipe-tests.log records the preceding software test command.

Committed independent native attribution, original red/diagnostic clear/restored
red, strict new context fixtures, two-run native repeatability on both devices,
and source bookends live in
`crates/vulkan/tests/fixtures/point-alpha-context/` on main. Production alpha
behavior and all original 21 assets are unchanged: the acceptance assertion now
checks legitimate retained alpha rejection, then resets it through a real opaque
sprite before strictly comparing camera-persistence color and depth.
This is not a production bypass, fixture tolerance change or full-class claim.

## Remaining final-source acceptance

Freeze a new exact local source revision and bind every new receipt/archive to it.
The old bb1e results cannot become owner results for this new revision.

- Exact CI workspace `cargo lint`, `cargo test-all --no-fail-fast`, and
  `RUSTDOCFLAGS=-D warnings cargo doc --workspace --all-features --no-deps --locked`.
- Exact instrumented workspace coverage command/profile and declared CI exclusion;
  preserve its own LCOV and source fingerprint.
- Final tooling, canonical inventory/current source selector checks and dependencies.
- Generate and validate new source-bound local package/test-support archives,
  checksums and extracted package feature matrix; never overwrite immutable
  published release assets or relabel old bb1e archive receipts.
- Quiet optimized renderer benchmark on the same accepted workloads, software
  SSE2/128 and native host separately, with time, peak RSS and source/binary
  bookends. Root must reserve an actually idle slot; no timing GO was issued.
- Both-profile functional GPU evidence already exists on corrected81ec; rerun
  only if the chosen final source changes the executed inputs/context.

Root alone decides any later final version/tag/publication after authorization.
This handoff starts no build, timing, tag or remote operation.

## Cleanup dependencies

`preservation-manifest.json` and `SHA256SUMS` bind the 62 original exported files
(201,237,429 bytes), including 31 verified .crate assets, existing test-support
source archive, full LCOV, old failed binary, logs and receipts. The corrected
source export is separately named and does not alter those bb1e checksums.
All owned candidate/diagnostic processes are terminal. No pending task needs the
old candidate build cache. Original unfinished controller work remains deferred
in `../Martensite-controller-frame-events` at clean `c72b9b6ab`; do not delete it.
