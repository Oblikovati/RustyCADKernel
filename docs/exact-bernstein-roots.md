# Exact Bernstein roots for analytic curve prerequisites

Tracking: [kernel issue #10](https://github.com/Oblikovati/RustyCADKernel/issues/10).
Decision: [ADR 0005](adr/0005-exact-bernstein-root-input.md). Base: local kernel
`d983a7383`, the exact power-coefficient solver from ADR 0004.
The later [Bézier-to-line query](exact-bezier-line.md) constructs projected
scalar controls with exact coordinate and weight products.

`tkmath::predicates::roots::isolate_bernstein_roots` takes finite scalar
Bernstein control values, a finite closed query interval, the existing versioned
root policy, and cancellation. The basis is fixed at [0,1]. A query outside
[0,1] evaluates the same polynomial by extrapolation; bounds do not rescale it.
The returned ordered brackets, exact multiplicities, measured dyadic widths,
subdivision count, and typed failures use the same root implementation as power
coefficients. No new root engine or approximate fallback is added.

Binary64 values are lifted to integers with one exact common scale. Integer
forward differences and binomial factors construct power coefficients without
rounding. True degree admission follows exact conversion and content reduction.
A degree-64 representation of a linear or constant polynomial is therefore
admitted under the corresponding degree-1 or degree-0 limit. Empty and all-zero
control arrays decline as zero polynomials. Cancellation discards both conversion
and isolation state. Temporary storage uses a linear number of integer slots.
Conversion uses quadratically many integer operations in control count.
Degree admission is not a running-time guarantee.

## Independent evidence

`crates/tkmath/tests/exact_roots/bernstein/jacobian.jsonl` retains 20 requests,
the supplied control-value bits, and complete independent Jacobian results:
38 distinct real roots with minimal polynomials, multiplicities, and rational
isolating intervals. Requests have degree at most eight and meet the oracle's
coefficient-size limit. A separate Python `Fraction` check expands every
`choose(n,i) * b[i] * t^i * (1-t)^(n-i)` directly, verifies the submitted rational
coefficients, checks proportionality to the returned primitive source, and
reproduces the flattened `oracle.tsv` byte-for-byte. Production conversion uses
forward differences, not this direct expansion.

Rust tests compare complete counts and exact multiplicities, require ordered
strictly disjoint brackets, and evaluate each independent minimal polynomial
at both returned bounds using the shared direct integer power-sum certificate.
Negating every input preserves the whole result. Generated integer controls
use ten deterministic cases, degrees two through eight, from LCG seed 10 with
multiplier 1664525, increment 1013904223, and modulo 2^32. Values are
state mod 17 minus 8. All reference roots lie inside the tested interval [-16,16].

Two explicit counterexamples establish why ordinary floating conversion cannot
supply a topology certificate. Controls `[1,-1,next_up(1)]` have no real roots,
while `[1,-1,next_down(1)]` have two simple roots. Rounded power conversion turns
both into `[1,-4,4]`, a false double root at 1/2. Exact conversion retains the
independent counts. Other cases cover endpoint roots, degree reduction,
extrapolation, reversal, exact degree elevation, all finite exponent fields,
maximal finite controls, a distinct root below the smallest positive binary64
parameter, and multiplicities 32 and 64 beyond the oracle's degree scope.

Cancellation is injected at every observed checkpoint of conversion plus
isolation. Input bits remain unchanged on success, work exhaustion, and
cancellation. Completed results remain unchanged after source mutation and
clearing. The original power corpus still checks whole-result equality under
exact scales; elevated Bernstein and power representations also produce
identical brackets, multiplicities, and work counts for the same polynomial.

## Local validation

```sh
cargo test -p rustycad-tkmath --all-features --locked
cargo clippy -p rustycad-tkmath --all-targets --all-features --locked -- -D warnings
RUSTDOCFLAGS='-D warnings' cargo doc -p rustycad-tkmath \
  --all-features --no-deps --locked
cargo llvm-cov -p rustycad-tkmath --all-features --locked --test toolkit \
  --json --output-path /tmp/bernstein-coverage.json -- exact_roots
cargo bench -p rustycad-tkmath --bench exact_roots --profile dev \
  --all-features --locked
```

The frozen-source TKMath suite passes 11 unit, 2,756 integration, and 39
documentation tests, with none skipped. The 26 exact-root tests cover 544/550
production lines (98.91%); the new conversion module covers 34/34 lines (100%).
Duplication across the seven root modules is 0%. Clippy, Rustdoc, formatting,
markdownlint, SPDX-header inspection, and `git diff --check` pass. Integer
index arithmetic is bounded by allocated array lengths; results do not depend
on pointer order, hash order, native floating evaluation, or word size.

The benchmark replays the 22 power and 20 Bernstein requests 1000 times each
after warmup, asserting every complete root count. It reports 42,000 requests,
85,000 roots, and 12.332 seconds in the timed loop. `/usr/bin/time` reports
5,356 KiB peak process RSS and 12.34 seconds wall time. Build time is excluded
from the direct executable time and peak-RSS measurement.
This is a local Linux debug-profile baseline, not an optimized performance,
application-latency, or cross-platform claim.

## Remaining application requirements

The new boundary certifies supplied scalar controls. It does not certify
floating point/weight products, a rational denominator with poles, spline knot
extraction, periodic trims, or a full geometric intersection. Exact construction
of those inputs and complete bounded-region validation, achieved tolerances,
source history, and persistent split identity remain required by issue #10.
No geometric recognizer, tolerance constant, fallback site, or geometric type
assertion is added. No mapped OCCT operation is replaced. The application still
requires a tested owner release; its dependency pin and production extrusion
path do not change here.
