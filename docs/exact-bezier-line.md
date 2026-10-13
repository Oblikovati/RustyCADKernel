# Exact Bézier-to-line parameter contacts

Tracking: [kernel issue #10](https://github.com/Oblikovati/RustyCADKernel/issues/10).
Decision: [ADR 0006](adr/0006-exact-bezier-line-parameters.md). Base: kernel
`8c764eb99`, following the exact Bernstein conversion from ADR 0005.
The later [algebraic root-sign operation](exact-root-signs.md) supplies the
next numerical prerequisite for directed ray-side classification.

`tkmath::predicates::roots::bezier_line::intersect_bezier_line` accepts immutable
planar control points, optional weights, two distinct line endpoints, a closed
parameter interval, the existing versioned root policy, and cancellation. It
returns either all isolated parameter contacts or a distinct coincident result.
An isolated result with no roots is completed empty. The line is infinite; the
query interval does not rescale the fixed [0,1] Bernstein basis.

All coordinates and weights are converted to exact integers before differences,
determinants, or products. The projected numerator has controls
`w_i * det(B-A, P_i-A)`; the denominator has controls `w_i`. Shared exact Bernstein
conversion and Sturm isolation classify these polynomials. The determinant is
also shared with `orient2d`; its existing independent corpus remains a gate.
No floating projection, numerical retry, perturbation, or geometry construction
is introduced.

The denominator must be nonzero on the complete closed interval. Signed and
individual zero weights are admitted under that condition. A denominator root,
including an endpoint, repeated root, or simultaneous numerator zero, causes a
typed failure containing the certified root parameters. An identically zero
denominator is a separate failure. No common factor is cancelled silently.
Only after denominator regularity passes can numerator zeros become contacts.
Their multiplicities are exact; an identically zero numerator is coincidence.
Both root stages share the subdivision limit. Errors retain their input owner
or root stage and original cause. No failure exposes a successful partial list.

## Independent mathematical and native evidence

`crates/tkmath/tests/exact_roots/bezier_line/jacobian.jsonl` retains the exact
input bits and 17 nonconstant Jacobian root requests across 12 curves. The
complete results contain 21 global numerator/denominator roots; 13 numerator
contacts lie in [0,1], and every tested denominator is regular there. Constant
denominators are checked directly. A separate Python `Fraction` pass recomputes
coordinate differences and weighted determinants, expands the Bernstein basis
by direct binomial sums, verifies every submitted coefficient and proportional
primitive source, excludes denominator zeros, and reproduces `oracle.tsv`.
Oracle degree and coefficient-size bounds are respected, including the exact
products of three smallest-subnormal values.

Rust tests assert complete contact counts and multiplicities, strict bracket
order, width, and independent minimal-polynomial signs at each endpoint. The
same corpus is invariant under line reversal and global weight negation.
Cases include tangency, endpoints, weighted quadratics, an oblique line,
negative/zero weights, near-contact rounding counterexamples, and projection
products outside binary64's finite nonzero range. A separate near-collinear
constant curve has a nonzero exact determinant while the rounded determinant
is zero; it must return empty, not coincidence.

Generated scale checks span every finite binary64 exponent field, both signs,
and maximal finite values with simultaneous coordinate and weight scaling.
Additional tests cover exact affine frame changes, point intervals, parameter
trims, coincidence, pole refusal before coincidence, invalid inputs, staged
work/degree/version errors, cancellation at every observed checkpoint, and
input/result ownership across success, failure, cancellation, and source edits.
The cancellation corpus includes successful contacts, coincidence, and a
singular-denominator failure. Source-bit checks preserve negative zero too.

The retained `occt-8.0.0.tsv` contains six independently executed ordinary-input
queries. Every count agrees and each native parameter lies inside its matching
exact bracket without a comparison epsilon. All native segment counts are zero.
The native computation uses tolerance 1e-10; it does not certify exact signs,
singular weights, or extreme arithmetic. Jacobian and the exact certificates
own those checks. Reproduce the native receipt with:

```sh
g++ -std=c++17 -I../occt-install/include/opencascade \
  crates/tkmath/tests/exact_roots/bezier_line/occt_oracle.cpp \
  -L../occt-install/lib -Wl,-rpath-link,../occt-install/lib \
  -lTKGeomAlgo -lTKGeomBase -lTKG2d -lTKMath -lTKernel \
  -o /tmp/bezier-line-occt-oracle
LD_LIBRARY_PATH=../occt-install/lib /tmp/bezier-line-occt-oracle \
  crates/tkmath/tests/exact_roots/bezier_line/oracle.tsv
```

The repeated output matches the retained receipt byte-for-byte. Native OCCT is
not a Cargo dependency and is not used by the production query or Rust tests.

## Local validation and cost

```sh
cargo test -p rustycad-tkmath --all-features --locked
cargo clippy -p rustycad-tkmath --all-targets --all-features --locked -- -D warnings
cargo llvm-cov -p rustycad-tkmath --all-features --locked --test toolkit \
  --json --output-path /tmp/bezier-coverage.json -- exact_
RUSTDOCFLAGS='-D warnings' cargo doc -p rustycad-tkmath \
  --all-features --no-deps --locked
cargo bench -p rustycad-tkmath --bench bezier_line --profile dev \
  --all-features --locked
```

The frozen-source TKMath suite passes 11 unit, 2,772 integration, and 40
documentation tests, with none skipped. Fifty exact-predicate tests cover
770/775 production lines (99.35%); the four new query modules cover 175/176
lines (99.43%). Duplication across all twelve predicate modules is 0%.
Clippy, Rustdoc, formatting, markdownlint, SPDX-header checks, and
`git diff --check` pass. Exact integer arithmetic and bounded array indices
avoid dependence on native word size, map order, or fused floating operations.
The retained benchmark runs the same 12 accepted input curves 1000 times after
warmup, asserting all contact counts. It includes exact projection, owned scratch
arrays, denominator checks, conversion, and numerator isolation. Native calls
and compilation are excluded. The run reports 12,000 queries, 13,000 contacts,
and 0.872 seconds in the timed loop. `/usr/bin/time` reports 5,152 KiB peak RSS
and 0.87 seconds wall time. These local debug-profile measurements are not
optimized-build, application-latency, or worst-case guarantees.

## Remaining region requirements

This is a parameter predicate, not a built edge, region, or B-rep. It does not
provide finite-line clipping, ray-side selection, periodic spline extraction,
curve/curve intersections, achieved modelling tolerance, full B-rep
validation, source history, or persistent split naming. Those requirements
remain under issue #10 before the application can use general sketch profiles
and extrusion. No mapped operation changes, and no production engine is
removed. Net changes in geometric recognizers, tolerance constants, fallback
sites, and geometric type assertions are zero. The application dependency pin
and production paths are unchanged. Validation is local Linux; cross-platform
execution remains unverified.
