# Exact directed Bézier wire classification

Tracking: [kernel issue #10](https://github.com/Oblikovati/RustyCADKernel/issues/10).
Decision: [ADR 0010](adr/0010-exact-directed-bezier-wire-classification.md).
Base: `f9164a8ad`, following exact segment winding contributions.

`tkmath::predicates::roots::bezier_line::classify_bezier_wire` accepts a nonempty
ordered list of borrowed polynomial or rational Bézier occurrences. Each has a
unique caller-owned ID, finite controls, optional weights, a closed parameter
interval, and an explicit reversal flag. One ray frame supplies the seed and
positive direction. One root policy supplies per-segment degree/width limits
and the subdivision budget for the whole wire.

The query proves every consecutive join and the closing join exactly. It returns
outside for zero winding, inside with the complete signed nonzero winding, or
boundary with every incident occurrence ID in input order. Reversal swaps exact
endpoints and negates the segment contribution without forming a rounded `1-t`.
Repeated traversal retains winding multiplicity. Self-crossing and zero-area
closed paths remain mathematically admissible; this result does not claim that
they form valid B-rep faces.

Every occurrence must finish before a boundary result can succeed. A boundary
on an earlier occurrence does not hide a later pole, open join, cancelled query,
or exhausted isolation budget. Nested errors retain the affected occurrence ID
and original stage/cause. No partial winding or boundary list succeeds. IDs are
occurrence labels, not persistent geometric names or complete source history.

## Exact closure

The shared projection supplies unreduced normal, longitudinal, and denominator
Bernstein controls. Homogeneous de Casteljau evaluates all three together at the
exact dyadic trim bounds. Each level multiplies by the same parameter denominator,
so its scale cancels in both coordinate ratios. Equality of the two projected
ratios is equivalent to point equality because the common nondegenerate ray
frame is invertible. Cross multiplication handles different degrees, weights,
parameterizations, and denominator signs without rounded division.

Separate primitive polynomial reductions would destroy the relative endpoint
scales. The capture therefore runs on the unreduced controls, and the denominator
gate must finish before any endpoint incidence is used. Existing scalar, ray,
and winding APIs use a no-op capture and keep their failure order. The winding
engine reports its already measured subdivision count to the wire; no second
root or winding implementation is introduced.

Input slices contain plain numeric values and are never retained or mutated.
Returned boundary IDs are owned. Cancellation is polled during ID admission,
projection, endpoint evaluation, root work, and winding accumulation. It does
not interrupt individual arbitrary-precision operations or allocator work.
The subdivision budget is not a wall-time or total-allocation bound.

## Independent equations and native evidence

The retained 48-case corpus contains squares, diamonds, a quadratic lens, and
a single closed cubic. It asserts full locations, signed winding, and exact
boundary occurrence lists. Each case is repeated with four further ray directions.
For the lens, x=4t-2 and y=±4t(1-t), hence x²±4y-4=0. For the cubic,
x=12t(1-t)² and y=12t²(1-t), hence (x+y)³-12xy=0. In the positive quadrant,
t=y/(x+y) identifies its boundary parameter; the bounded interior has negative
implicit value. The quadrant restriction is essential: a negative value at
(0,-1) is outside, not inside the loop.

`tests/exact_roots/bezier_line/wire/jacobian.jsonl` retains 17 exact evaluations
of those implicit equations. A separate rational replay checks each source-bit
seed, canonical polynomial, exact value, domain restriction, and expected corpus
location. Polygon expectations follow exact coordinate and half-plane relations.
This evidence does not use the production projection or winding algorithm.

The native generator builds and validates an OCCT 8.0.0 face for every corpus
row. Source checkout: `3d097a0328`. Its raw classifier agrees on 45 locations.
It reports IN at both lens endpoints and at the cubic's closing point, despite
exact native vertex-coordinate equality. The retained vertex probe reproduces
the cubic discrepancy at zero surface distance and UV=(0,0), with both 2D and
3D native classifiers, for tolerances from 1e-12 through 1e-5. Native enum value
0 in that receipt is IN. An inferred plane gave the same three disagreements.

The new exact query returns boundary for these three cases. Tests retain the
raw IN values, require the exact three case names and native vertex evidence,
and separately assert the equation-derived boundary IDs. No full native
classifier parity is claimed, and the mapped classifier remains unchanged.
This is an intentional extension-policy difference recorded in ADR 0010.
The diagnostic source and JSONL receipt are retained beside the native corpus.
All earlier line, ray, and segment-winding native receipts reproduce byte-for-byte.

Further regressions cover exact non-binary rational endpoint equality, a rounded
2/3 endpoint that must not close, a one-subnormal gap, out-of-basis trims, reversed
subnormal trims, degenerate occurrences, duplicate IDs, signed weights, complete
boundary lists, and self-crossing paths. Generated checks cover all finite
binary64 exponent fields and both scale signs. Shared-budget tests pass at the
measured total and decline one subdivision below it. Cancellation is injected
at every observed checkpoint for interior and boundary queries. Source bits and
returned results remain isolated after success, failure, and source clearing.

## Local commands

```sh
cargo test -p rustycad-tkmath --all-features --locked
cargo clippy -p rustycad-tkmath --all-targets --all-features --locked -- -D warnings
cargo llvm-cov -p rustycad-tkmath --all-features --locked --test toolkit \
  --json --output-path /tmp/wire-coverage-final.json -- exact_
RUSTDOCFLAGS='-D warnings' cargo doc -p rustycad-tkmath \
  --all-features --no-deps --locked
cargo bench -p rustycad-tkmath --bench bezier_wire --profile dev \
  --all-features --locked
```

The retained C++ generators use the sibling `occt-install` headers and libraries.
Build `wire/occt_oracle.cpp` or `wire/occt_vertex_probe.cpp` with TKTopAlgo, TKBRep,
TKGeomAlgo, TKGeomBase, TKG3d, TKG2d, TKMath, and TKernel. Pass the corpus path;
the probe also takes `cubic_2`. Native libraries and Jacobian generate fixtures
only; Rust tests have no dependency on either runtime.

## Validation and cost

The full TKMath suite passes 14 unit, 2,838 integration, and 44 documentation
tests, with none skipped. The 116 exact-predicate integration tests cover
1,516/1,534 predicate lines (98.83%). The five new wire modules cover 207/213
lines (97.18%); endpoint evaluation/comparison covers 40/40. The guarded winding
overflow also has a unit test in the full suite. Duplication across all 25
predicate modules is 0%. Clippy, strict Rustdoc, scoped formatting, Markdown
lint, SPDX inspection, and whitespace checks pass on Linux x86_64, rustc 1.99.0.

The benchmark asserts all 48 expected complete locations after one corpus warmup.
Three runs each perform 1,000 repetitions, or 48,000 requests, in 5.273, 5.180,
and 5.201 seconds in the timed loop. Peak process RSS is 5,152, 5,320, and
5,308 KiB respectively. Work includes borrowed-occurrence view allocation,
endpoint certification, ID/closure checks, complete winding, and owned results.
Builds and native/math oracle calls are excluded. These are sequential local
debug-profile measurements, not optimized application or worst-case guarantees.

## Remaining region work

This query certifies closure and seed location for supplied planar Bézier paths.
It does not extract analytic curves, arrange intersections, construct overlap
intervals, validate a B-rep, measure achieved modelling tolerance, or supply
persistent split history/naming. Disconnected multi-wire region assembly and
the end-to-end named region/prism gate remain under issue #10. The application
pin and production profile/extrusion behavior are unchanged. No sampled
classifier is promoted into production acceptance.
Net changes in geometric recognizers, tolerance constants, fallback sites, and
geometric type assertions are zero. Cross-platform execution remains unverified.

The [constructed cubic prism probe](exact-region-prism.md) now reads actual
region pcurves into this classifier and checks the resulting prism against
native face reports. It also records the shared-input mutation in the diagnostic
construction path and the tested geometry-copy boundary. This does not complete
the production region contract.
