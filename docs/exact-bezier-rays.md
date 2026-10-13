# Exact directed sides at Bézier contacts

Tracking: [kernel issue #10](https://github.com/Oblikovati/RustyCADKernel/issues/10).
Decision: [ADR 0008](adr/0008-directed-bezier-contact-sides.md). Base: kernel
`488a7a4e4`, after exact line parameters and algebraic-root signs.

`tkmath::predicates::roots::bezier_line::classify_bezier_ray_contacts` accepts
the existing borrowed Bézier query. The first line point is the ray origin;
the second defines its direction. Every isolated supporting-line contact is
returned with its original bracket and multiplicity and one aligned side:
`Less` behind the origin, `Equal` at the origin, or `Greater` ahead. Behind
contacts remain in the result. Distinct parameters at the same position remain
distinct contacts. A nonzero normal polynomial with no interval roots completes
empty. Supporting-line coincidence has a named decline because isolated roots
cannot describe its continuous overlap.

The shared exact coordinate traversal produces normal numerator N, denominator
D, and longitudinal numerator L. After certifying D has no interval roots,
the side at each N root is the sign of L*D. This equals the sign of L/D because
D² is positive. The existing Sturm–Tarski operation consumes these integer
polynomials directly. No rounded projection, approximate parameter, sampled
sign, second root engine, or geometric tolerance enters the decision. Input
polynomials obey the degree policy separately; internal products can exceed
that degree. A zero L is valid and makes all isolated contacts origin contacts.
The original infinite-line query uses the same preparation and isolation code.

Typed causes retain their input, denominator, normal, direction, or sign stage.
The denominator gate precedes coincidence, even at removable common zeros.
Cancellation returns no partial contact list. Subdivision work remains shared;
sign certification adds no subdivision. Degree and subdivision limits do not
bound the cost of each arbitrary-precision integer operation.

## Independent and native evidence

The retained `tests/exact_roots/bezier_line/rays/jacobian.jsonl` corpus contains
16 curves, 36 exact root-isolation calls, and 79 algebraic comparisons. It
certifies 18 contact sides. Python `Fraction` independently expands the normal,
denominator, and dot-product Bernstein expressions from the exact input bits.
For each source root, exact comparisons with query roots establish the sign:
start with the query's leading sign and flip for every larger odd-multiplicity
query root; a common root gives zero. Comparisons with both interval bounds
establish inclusion. This calculation does not use the production sign sequence.

A separate replay checks every input projection, oracle source normalization,
comparison identity, and reconstructed TSV row. The overflow case's directed
polynomial has a 1,233-digit coefficient. Removing its positive integer content
before the oracle call preserves its sign and meets the oracle's 996-digit
input bound. Both forms and the scale remain in the receipt. All accepted
requests meet the degree-eight and coefficient-size bounds.

The corpus includes signed and zero individual weights, endpoint and tangent
contacts, exact zero longitudinal projection, and two parameters ±sqrt(2) at
the same origin. The two adjacent binary64 values around sqrt(2) give opposite
sides for x(t)=t-c at y(t)=t²-2. A dedicated regression uses the unchanged root
bracket [1,2], whose midpoint gives the wrong side for the larger c.

Tests retain the whole legacy line isolation, reverse direction at a fixed
origin, negate all weights, and scale through all finite binary64 exponent
fields and both signs. They check retained input bits and owned output after
source mutation. Cancellation is injected at every observed checkpoint for
isolated, empty, coincident, and denominator-pole queries. Policy, degree,
subdivision, and coincidence failures remain typed.

The OCCT 8.0.0 generator adds an optional `--sides` mode. Its six ordinary cases
match native sides, including an exact endpoint at the origin; default parameter
receipts reproduce byte-for-byte. Native approximate arithmetic is supplementary
and does not certify the irrational/common-root cases. The native library and
Jacobian are fixture generators only, with no Rust runtime dependency.

## Local checks and cost

```sh
cargo test -p rustycad-tkmath --all-features --locked
cargo clippy -p rustycad-tkmath --all-targets --all-features --locked -- -D warnings
cargo llvm-cov -p rustycad-tkmath --all-features --locked --test toolkit \
  --json --output-path /tmp/bezier-ray-coverage-final.json -- exact_
RUSTDOCFLAGS='-D warnings' cargo doc -p rustycad-tkmath \
  --all-features --no-deps --locked
cargo bench -p rustycad-tkmath --bench bezier_line --profile dev \
  --all-features --locked -- --rays
```

Validation uses Linux x86_64 and rustc 1.99.0. The full TKMath suite, Clippy,
strict Rustdoc, scoped rustfmt, markdownlint, SPDX inspection, and whitespace
checks pass. The suite has 12 unit, 2,803 integration, and 42 documentation
tests, with none skipped. The 81 exact-predicate integration tests cover
1,066/1,073 predicate lines (99.35%). The two new ray modules cover 89/89 lines.
All 17 predicate modules
have 0% measured duplication. Exact binary64 input conversion and integer
arithmetic introduce no floating threshold, platform-sized constant, pointer
order, or map order. Cross-platform execution remains unverified.

The benchmark runs 1,000 repetitions after one corpus warmup and asserts all
expected sides. Its 16,000 directed queries produce 18,000 sides in 2.193 seconds;
peak process RSS is 5,144 KiB and wall time is 2.19 seconds. Default line mode
runs its unchanged 12-case workload: 12,000 requests, 13,000 contacts, 0.885
seconds, and 5,356 KiB peak RSS. These are different workloads, not a speedup
comparison. Measurements exclude builds and native calls and include owned
scratch polynomials and exact certification. They are local debug-profile
measurements, not optimized application or worst-case guarantees.

## Remaining application work

This operation supplies isolated directed contacts. Overlap intervals, wire
endpoint ownership, winding, and full point-in-region classification remain.
Periodic spline extraction, curve/curve intersections, ordered full B-rep
validation, achieved modelling tolerance, source history, and persistent split
naming also remain under issue #10. No mapped OCCT operation is replaced.
Net changes in geometric recognizers, tolerance constants, fallback sites, and
geometric type assertions are zero. The application dependency pin, production
profiles, and extrusion are unchanged.
