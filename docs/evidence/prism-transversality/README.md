# Exact affine prism support-map evidence, version 1

This bounded prerequisite belongs to issue #10 / ADR0014. Code revision and
source hashes are in [receipt.json](receipt.json). The existing diagnostic prism
pipeline now captures its actual copied planar support and full placement, then
retains `det(dF/du, dF/dv, extrusion)` using exact dyadic arithmetic. This is an
affine-map injectivity theorem, not a completed analytic region or prism.

The combined `PrismInjectivityAndSurfaceCharts` gap stays open. Actual generated
cap/side chart correspondence, complete source/region evidence, and ordered body
validation are still required. No successful body, volume/error certificate,
new replacement pipeline, tolerance change, or old-behavior removal is claimed.

The final affected all-target/all-feature run passed 4,093 tests; all 84 doctests,
clippy with warnings denied, formatting, and the seven-test focused replay of
the final thread-worker helper passed. The earlier 4,090-test run is retained
separately. Ordinary-host ThreadSanitizer acceptance remains incomplete below.

## Independent checks

- [FLINT rational determinant](exact-oblique-determinant.json): exact value 501
  for the independently composed oblique matrix.
- Algebraic cancellation: with `n=2^52`, exact `n²-(n-1)(n+1)=1`, although the
  corresponding binary64 expression is zero.
- Generated exponent corpus: all 2,098 finite positive binary64 powers of two,
  with exact expected Jacobian `s^5`, including subnormal and extreme scales.
- Tangent/zero sweeps, dependent support columns and collapsed placements;
  finite translations, handedness reversal and actual face placement.
- Typed invalid coefficients/supports, all three numerical cancellation polls,
  operation failure/cancellation prefixes and subsequent request reuse.
- Input support mutation cannot affect the retained copy or scalar. Mutating
  the copied support changes a fresh observation while the old scalar remains
  historical evidence; it cannot certify the changed shape.

`runs.tar.gz` retains logs, the exact standalone probe sources/lockfile, output
bytes, process comparison and invocation history. `SHA256SUMS` covers that
archive and the individual receipt files. Run the affected suites with Rust
1.99.0:

```sh
cargo test -p rustycad-tkbo -p rustycad-tkmath --all-targets --all-features --locked
cargo test -p rustycad-tkbo -p rustycad-tkmath --doc --all-features --locked
cargo clippy -p rustycad-tkbo -p rustycad-tkmath --all-targets --all-features --locked -- -D warnings
cargo fmt -p rustycad-tkbo -p rustycad-tkmath --check
```

To replay the ordinary-host standalone probe, extract its `probe` directory
into `target/prism-transversality`, preserving that depth for the path dependency.
Run `cargo run --manifest-path target/prism-transversality/probe/Cargo.toml
--offline` twice and compare stdout. Each invocation compares eight threads ×
128 repetitions × four exact cases against its serial result. The captured
ordinary-host process outputs match byte for byte. This checks the pure scalar
calculation; the private compatibility occurrence adapter remains thread-confined.

## ThreadSanitizer remains incomplete on the ordinary host

The nightly `2026-09-25` TSan build succeeded, but execution exited 139 before
probe output. Its cause was not established. One later exploratory invocation
masked `/etc/ld.so.preload` inside `bwrap`; it exited zero without a race report
and produced matching stdout. Its log also records sanitizer address-space
reexecution. This altered environment is **excluded from ordinary-host TSan
acceptance**. The recorded invocation is historical evidence, not a replay
instruction. Do not repeat masking host protection. Completion requires a
supported instrumented runner without bypassing host protections.

Compile-time `Send`/`Sync` assertions cover the owned scalar evidence and its
dyadic result. No unsafe trait implementation was added. Neither these
assertions nor the stress checks establish full prism-result thread safety.

Existing native fixture replay is historical only; no OCCT execution or new
capture occurred. There is no application acceptance, benchmark against a
previous accepted prism operation, or completed operation claim in this record.
