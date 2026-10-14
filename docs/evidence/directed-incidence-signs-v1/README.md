# Directed incidence signs v1: candidate verification

The executed source revision is **ac5f2db3aaa2ff371f2461ff41979d32547286f4**,
based on 38a5edf6633ff3683d536a191f241497f8e3077d. This evidence does not
claim execution on a later integration revision. The evidence follow-up clarifies
only the subtraction wording in [ADR 0022](../../adr/0022-source-bound-directed-incidence-signs.md).
No Rust, fixture, manifest or behavior changed after these checks.

## Executed checks

Pinned Rust 1.99.0, Linux x86_64, `CARGO_BUILD_JOBS=2`:

- Full TKMath all-targets/all-features: **174 unit + 2,926 integration tests passed**.
  Existing bench workload gates and the example target also completed. Debug
  timing output is not release performance acceptance.
- **51 doctests passed**.
- Clippy for all targets/features with `-D warnings`: passed.
- Documentation with `RUSTDOCFLAGS='-D warnings'`: passed.
- TKMath formatting check: passed.

[receipt.json](receipt.json) records exact commands, toolchain, platform, source
revision, per-command unchanged-source checks and artifact hashes. The compressed
source fingerprint manifest is identical before and after the complete run.
Logs are gzip files; use `gzip -cd FILE.log.gz` to read them.

The pre-refactor owned fixture was captured from the original implementation,
then checked before extraction (1 pass) and after extraction (17 passes).
These working contents were committed separately as 2c5773909 and 533a5786c.
The full frozen run includes the unchanged fixture. This baseline is evidence
of retained behavior, not an independent numerical oracle.

## Independent expectations and replay

The exact rational fixture proves t=1/2, world x=1/3. The retained Jacobian
[receipt](../../../crates/tkmath/tests/exact_roots/bezier_line/incidence_assets/directed-third-jacobian.json)
records exact normal-zero and neighboring-binary64 subtraction evaluations.
Other expectations use dyadic closed forms, the inequality 1/3 > 1/4 for two
irrational roots, exact endpoint equality, reversal, signed/zero weights and
repeated-root multiplicity. Tests retain complete root-index alignment even
when contacts are filtered or parameter order opposes distance order.

Owned success, failure and cancellation inputs remain isolated after mutation.
All new public types have compile-time Send/Sync assertions. Eight threads each
perform 16 deterministic query/cancellation replays; two independent child
processes compare complete report bytes with each other and the serial result.
The encoding is Rust Debug output for same-revision test replay, not a persistent
wire-format contract. Cancellation is exhaustive over the rational fixture's
observed checkpoints; the irrational refinement fixture checks early, middle
and final checkpoints. Neither failure publishes a successful partial result.

## Incomplete and excluded acceptance

Ordinary-host ThreadSanitizer remains **incomplete**: the retained
[startup-control failures](../native-display-lifecycle-v1-2026-10-14/TSAN.md)
prevent sanitizer acceptance. This target was not executed under TSan, and no
host-protection or instrumentation bypass was used. Compile-time checks and
ordinary thread tests do not close that requirement.

This slice adds explicit opt-in endpoint evidence. It preserves legacy Support
skip semantics and does not claim a complete certified trim/extension operation,
nearest-hit ordering, TKBO/region success, broad API mapping, application
acceptance, GPU results, release performance, peak memory or a new native oracle.
