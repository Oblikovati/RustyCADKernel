# Exact polynomial root isolation — prerequisite for sketch regions

Tracking: [kernel issue #10](https://github.com/Oblikovati/RustyCADKernel/issues/10),
application port R3/R5 and K02/K11. Decision:
[ADR 0004](adr/0004-exact-polynomial-root-isolation.md).
Base: kernel `90df47a70`, following ADR 0003's exact orientation primitive.

`tkmath::predicates::roots::isolate_polynomial_roots` isolates the distinct real
roots of a nonzero polynomial on a finite closed interval. Inputs are ascending
binary64 coefficients, bounds, a versioned work policy, and a cancellation
callback. Each successful bracket contains exactly one root and records its
exact multiplicity. Brackets are closed, strictly disjoint, and ordered. A
point bracket is an exact dyadic root. Width is returned as an exact dyadic
quantity; it never underflows to floating zero.

Version 1 defaults to maximum degree 32 and 4096 subdivisions. The caller must
supply a positive approximation width and can select other degree/work limits.
Width is not a modelling or incidence tolerance. Constant nonzero polynomials
return a completed empty list. A zero polynomial, invalid scalar, unsupported
policy version, exhausted limit, or cancellation returns a typed error with no
successful partial list. Degree admission ignores leading zero coefficients.

All finite coefficients reuse ADR 0003's exact binary64 conversion. Positive
content normalization and signed integer pseudo-remainders preserve ordinary
Sturm signs. Endpoint right limits include repeated endpoint roots correctly;
GCD towers determine multiplicity. Exact dyadic midpoints avoid overflow and
can separate roots below binary64's smallest positive value. The working stack
retains only nonempty children. Shared subdivision endpoints retain one cached
classification. No coordinate perturbation, epsilon, floating filter, or
ordered fallback is used.

## Independent evidence

The 22 retained requests in
`crates/tkmath/tests/exact_roots/jacobian.jsonl` contain the canonical rational
input and complete result from Jacobian `polynomial.roots.isolate`. Its declared
scope is univariate QQ, degree at most eight, and bounded coefficient size.
Each request meets that scope. The backend supplies 47 distinct real-root
identities, multiplicities, and rational isolating intervals. `oracle.tsv` is the
flattened test input, not an independently edited expectation.

A separate Python `Fraction` check confirms that every stored binary64 bit
pattern is exactly the requested rational coefficient, and reproduces all 22
TSV rows from the retained results. Rust tests compare root counts and
multiplicities, require strict interval separation and the requested width,
and evaluate each independent minimal polynomial at the returned endpoints by
a direct integer power sum. Opposite endpoint signs, or exact zero at a point,
certify the root's presence independently of production Horner evaluation and
the signed remainder sequence. The complete independent root count and disjoint
brackets exclude missing or duplicated roots.

The corpus includes repeated roots, mixed multiplicities, irrational roots,
near-tangent quadratics below/at/above contact, closely spaced roots, and twelve
deterministically generated integer polynomials. Negating or exactly scaling
every corpus polynomial by powers of two preserves the entire result. Additional
cases cover every finite binary64 exponent field, degree-32 repeated roots,
a sparse degree-64 polynomial, maximal finite interval bounds, roots below the
smallest subnormal, closed point intervals, malformed input, and work limits.
Cancellation is injected at every observed checkpoint of a complete mixed-root
request. Input bits and completed results are retained across failure,
cancellation, and later source mutation.

## Local validation

This receipt describes commit `d983a7383`. The later
[Bernstein extension](exact-bernstein-roots.md) shares this solver and expands
the retained benchmark corpus. The measurements below retain their original
22-input scope.

```sh
cargo test -p rustycad-tkmath --all-features --locked
cargo clippy -p rustycad-tkmath --all-targets --all-features --locked -- -D warnings
RUSTDOCFLAGS='-D warnings' cargo doc -p rustycad-tkmath \
  --all-features --no-deps --locked
cargo llvm-cov -p rustycad-tkmath --all-features --locked --test toolkit \
  --json --output-path /tmp/exact-roots-coverage.json -- exact_roots
```

The complete TKMath suite passes 11 unit, 2,744 integration, and 38 documentation
tests, with none skipped. New production root-isolation modules have 479/484
covered lines (98.97%); duplication across those six files is 0%. Rustfmt checks
the changed
module trees. Rustdoc, markdownlint, SPDX-header inspection, and `git diff
--check` are included in the local checks. Validation is local Linux only.

The retained `exact_roots` Cargo benchmark replays the same 22 accepted corpus
requests 1000 times after warmup, checking every root count. Run it with
`cargo bench -p rustycad-tkmath --bench exact_roots --profile dev --all-features`.
The debug-profile run reports 22,000 requests,
47,000 roots, 6.056 seconds inside the timed loop, and 5,156 KiB peak process RSS
(`/usr/bin/time`; wall time 6.06 seconds). This is one local baseline, not an
optimized-build, comparative, application-latency, or worst-case claim.
Big-integer calls are synchronous; cancellation does not interrupt an individual
integer operation. Degree admission bounds the polynomial degree, not elapsed time.

## Remaining contract

This is a numerical primitive, not a sketch-region operation or a replacement
for mapped OCCT solvers. It certifies the supplied coefficients. Exact
coefficient construction from analytic curves, rational-curve denominators,
periodic trims, incidence ownership, complete geometry validation, achieved
modelling tolerance, and persistent split naming remain open under issue #10.
The primitive does not certify a rounded curve-to-polynomial conversion.

No mapped operation, recognizer, tolerance constant, fallback site, or geometric
type assertion changes. There is no new dependency; `num-bigint` remains private
behind the project-owned predicate interface. No application dependency pin or
production application path changes. A tested kernel owner release and the
complete region acceptance corpus are still required before the application
can replace its diagnostic sampled classifier and expose general extrusion.
