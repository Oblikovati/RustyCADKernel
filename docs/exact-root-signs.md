# Exact signs at algebraic source roots

Tracking: [kernel issue #10](https://github.com/Oblikovati/RustyCADKernel/issues/10).
Decision: [ADR 0007](adr/0007-signs-at-algebraic-roots.md). Base: kernel
`3d611879f`, following exact Bézier-to-line parameter queries from ADR 0006.

`tkmath::predicates::roots::signs::signs_at_polynomial_roots` accepts ascending
source and query coefficients, a finite closed interval, the existing versioned
root policy, and cancellation. It returns the complete source isolation and an
aligned immutable sign list. `Ordering::Less`, `Equal`, and `Greater` mean
negative, zero, and positive query values at the corresponding source root.
The returned brackets, multiplicities, and subdivision count are unchanged from
the source-only isolation. A zero source declines; a zero query is valid.
A constant source or a source without interval roots returns completed empty.

The implementation uses exact supplied binary64 values throughout. It builds
the signed remainder sequence of P and the remainder of P'Q modulo P. On a
one-root bracket with nonroot endpoints, the variation difference is the exact
query sign. Exact point roots use exact substitution. Integer convolution
supplies the product; shared signed remainders, positive content reduction,
and endpoint signs perform the work; the shared sequence builder replaces the
former embedded builder loop. No second remainder engine or root-approximation
heuristic exists. A variation difference outside -1, 0, or 1 is a typed
invariant failure.

The source and query degrees are admitted separately. Internal product degrees
can exceed those input limits. Cancellation is polled during convolution,
remainder construction, and root classification; no partial sign list succeeds.
Typed failures retain policy/source/query/evaluation ownership and their
original causes. The method requires no extra subdivision, but degree limits
do not guarantee a running-time bound or interrupt individual integer operations.

## Independent evidence

`crates/tkmath/tests/exact_roots/signs/jacobian.jsonl` retains 20 polynomial pairs,
40 complete root-isolation results, and 75 exact algebraic-root comparisons.
The 42 expected source-root signs are derived independently: start with the
query's leading-coefficient sign, then change it for each larger query root of
odd multiplicity. Equality to any query root makes the sign zero. Conjugate
nonreal factors stay positive on the real line. This uses root ordering and
multiplicity, not the production Sturm–Tarski sequence.

A separate Python `Fraction` check verifies the input bits against every exact
request, proportionality to the oracle's primitive sources, the source-bound
identities of every comparison, and byte-for-byte reconstruction of `oracle.tsv`.
Cauchy bounds put every source root inside the tested interval [-16,16]. All
requests respect their declared degree and coefficient-size bounds. Eight
polynomial pairs use deterministic integer coefficients from LCG seed 10,
multiplier 1664525, increment 1013904223, and modulo 2^32. Their nonleading
coefficients are state mod 17 minus 8; leading signs alternate.

Rust assertions compare all signs, retain the whole source isolation, and
check exact positive scaling, source negation, and query-sign reversal. Cases
cover irrational common roots, repeated roots, dyadic endpoint roots, constant
and zero queries, empty results, and a degree-eight query on a quadratic source.
One regression evaluates x-sqrt(2)_rounded at the positive root of x²-2 in the
unchanged bracket [1,2]. The midpoint has the wrong sign. Another has two query
roots inside that bracket and positive query values at both endpoints while
the enclosed source root has a negative query value. Neither width nor endpoint
sign agreement substitutes for the exact algebraic sign.

Extreme input tests separate a source root below the smallest binary64 parameter
and certify a strictly positive subnormal-scaled query there. Cancellation is
injected at every observed checkpoint for four query types, including common
factors and a zero query. Source bits, including negative zero, survive success,
failure, and cancellation; completed results survive source mutation/clearing.
Invalid variation differences are tested in both directions and at usize::MAX.

Four ordinary source polynomials are exact projections of the retained ADR 0006
OCCT fixtures. Their signs relative to parameter 1/4 match native parameters;
these comparisons stay away from query zeros. The OCCT 8.0.0 generator is rebuilt
and its six-row receipt reproduces byte-for-byte. This is a supplementary ordinary
case gate. The exact algebraic oracle owns common-root and near-zero decisions;
native approximate evaluation is not promoted to an exact sign certificate.
All earlier orientation, Bernstein, and Bézier-line corpus tests remain gates.

## Local validation and cost

```sh
cargo test -p rustycad-tkmath --all-features --locked
cargo clippy -p rustycad-tkmath --all-targets --all-features --locked -- -D warnings
cargo llvm-cov -p rustycad-tkmath --all-features --locked --test toolkit \
  --json --output-path /tmp/root-sign-coverage.json -- exact_
RUSTDOCFLAGS='-D warnings' cargo doc -p rustycad-tkmath \
  --all-features --no-deps --locked
cargo bench -p rustycad-tkmath --bench root_signs --profile dev \
  --all-features --locked
```

The frozen-source TKMath suite passes 12 unit, 2,787 integration, and 41
documentation tests, with none skipped. The 65 selected exact-predicate
integration tests cover 911/918 production lines (99.24%). The three new sign
modules cover 113/115 lines (98.26%); the private invalid-variation guard also
has a unit test in the full suite. Duplication across all fifteen predicate
modules is 0%. Clippy, Rustdoc, formatting, markdownlint, SPDX inspection, and
`git diff --check` pass. Integer index arithmetic stays within allocated array
lengths; no floating sign threshold, pointer order, or map order is introduced.

The benchmark replays the same 20 accepted polynomial pairs 1000 times after
warmup, asserting every sign. It includes source isolation,
owned scratch polynomials, remainder construction, and sign evaluation. Build
time and native calls are excluded. The run reports 20,000 queries, 42,000
signs, and 4.401 seconds in the timed loop. `/usr/bin/time` reports 5,272 KiB
peak process RSS and 4.40 seconds wall time. These local Linux debug-profile
measurements are not optimized, application-latency, or worst-case guarantees.

## Remaining application requirements

This numerical operation does not yet perform directed Bézier/ray selection.
That consumer must retain exact projected integer coefficients and denominator
signs, then handle ray origins, endpoint ownership, tangencies, and overlaps.
Periodic spline extraction, curve/curve intersections, complete region
validation, achieved modelling tolerance, source history, and persistent split
naming also remain required under issue #10. No mapped OCCT result changes.
Net changes in geometric recognizers, tolerance constants, fallback sites, and
geometric type assertions are zero. The application dependency pin and production
profiles/extrusion remain unchanged. Cross-platform execution is unverified.
