# Private polynomial prism embedding, behavior version 1

Frozen implementation: `8b8153074` on base `83eccdbcc`. This is candidate-revision
verification, not an assertion that later main revisions were tested. Structural
fixture access commits `8ba42e411` and `569c2b922` precede the separate additive
behavior commit. ADR0029 records the supported domain and theorem.

## Scope and independent expectations

The private opt-in inspector composes the existing fresh Jordan/current-chart
binding, exact nonzero affine determinant, and freshly computed actual boundary
regularity. It owns three trimmed domains and all six self/pair incidence rows.
The cap domains are the captured closed Jordan component at sweep 0 and 1; the
side is the full native boundary interval and normalized sweep [0,1], modulo only
the authored native 0/1 seam. Actual stored sweep is [-h,0], with retained h and
the shared exact E=-h*placed-direction proof. Every rim/seam use retains source
and output identity, all parents and intervals. Full coefficient, pcurve,
placement, closure-context and trim-expectation operands remain in the owned
shared binding. A final callback-free rebind checks them after all callbacks.

The independent cubic expectation is J=2 and
`||C'||²=16+32(9t²-9t+2)²>=16`. Its endpoint tangents differ, so successful closure
is a corner. The retained algebra receipt is
`crates/tkmath/tests/exact_roots/bezier_regularity/cubic-jacobian.json`.
The quartic derivative gcd receipt remains
`crates/tkbo/src/sketch_region_prism/region_certificate/embedding_quartic_gcd.json`.
The actual region pipeline still refuses that stationary quartic; no fabricated
Jordan admission replaces it. A separate guard fixture x=(3t-1)² has exact squared
speed 36(3t-1)², so a zero bisection budget at its non-dyadic 1/3 zero is incomplete.
These independent identities do not verify the Rust implementation themselves.

Version 1 requires a stored polynomial (`weights=None`). Rational, uniform
explicit, negative and sign-changing weight representations are explicit typed
refusals. Existing shared Jordan/chart admission of some rational inputs does not
widen this theorem. The previously integrated four owned baseline tests remain
unchanged. No historical fixture or expected baseline was replaced.

## Reproduction and evidence

`runner.py` records the exact frozen matrix commands, compiler/platform, two-job
policy and all tracked crate/configuration input hashes before and after the run.
The test matrix covers TKMath and TKBO with all features/all targets, separate
doctests, Clippy with warnings denied, and workspace formatting. `receipt.json`
records exits, elapsed times and counts: **4,358 tests and 88 doctests passed**,
Clippy and formatting passed, and all **15,716** input hashes were unchanged.
The 28 new embedding tests and four unchanged baseline tests passed in that matrix. Compressed logs and manifests are
lossless; SHA256SUMS covers the retained artifacts. All input bookends must match.

The new tests exercise source placement/reversal/scaling, negative extrusion,
actual regularity source binding, changed source/copy/side/cap trims, missing or
extra actual faces, altered role/history/context/seed, mismatched physical rim or
seam correspondence, budget/version refusal, all cancellation polls and mutations
at the last callback. Old report operands remain isolated after source mutation.
Compile-time assertions cover immutable evidence/views as Send+Sync. Eight
threads each replay four operations; two fresh processes compare the same
mathematical receipt byte for byte against serial. Operation-local topology
addresses are deliberately excluded from this portable mathematical projection;
that projection is not a durable authored topology identity scheme.

Development logs are explicitly historical unfrozen evidence. The first Clippy
preflight rejected large typed errors; boxing preserves their complete typed
causes and the new layout test guards the size. `boxing-comparison.json` records
the identical 2,095-byte before/after mathematical receipt. Earlier failed tests
exposed mistaken test assumptions (closure physical-group count, and a no-op role
mutation of a non-face binding); tests now check all authored stage contexts and
mutate an actual face binding. These failures are retained rather than presented
as frozen successful evidence.

## Exclusions

The inspector does not modify PrismReceipt, clear mapped gaps, set OrderedCoverage
or make body success claims. It does not prove global C1 closure, untrimmed
support-surface injectivity, generated output vertex metric incidence, sewing,
outward orientation, ordered body validation or tolerance consistency. Scalar
measurements, application acceptance and release performance are separate gates.
No new OCCT execution, GPU run, alternate intersection engine or shape-specific
fallback is included.

ThreadSanitizer remains incomplete: retained ordinary-host startup controls fail
as documented in `docs/evidence/native-display-lifecycle-v1-2026-10-14/TSAN.md`.
No TSan bypass or new TSan claim was used. Ordinary Send/Sync and replay tests do
not establish complete thread-safety acceptance. Published evidence is immutable;
future checks require a new revision-specific receipt.
