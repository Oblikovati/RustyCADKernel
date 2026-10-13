# Exact Bézier winding contributions

Tracking: [kernel issue #10](https://github.com/Oblikovati/RustyCADKernel/issues/10).
Decision: [ADR 0009](adr/0009-exact-bezier-winding-contribution.md).
Base: `6b0c27bd9`, following exact directed-contact classification.

`tkmath::predicates::roots::bezier_line::bezier_winding_contribution` returns
`Boundary` if the supplied segment contains the ray origin, or `Clear(i32)`
with its complete signed crossing contribution. The curve runs from the lower
to the upper parameter bound. The first line point is the origin and the second
defines the positive ray. The result owns no curve or input storage.

A positive-normal crossing contributes +1; a negative-normal crossing contributes
-1. Behind-origin contacts contribute zero. Interior even-order tangencies also
contribute zero. Lower endpoints count only departure into the positive normal
half-plane; upper endpoints count only arrival from that half-plane, as -1.
This half-open rule counts a shared crossing once and cancels a shared touch.
It also handles stationary endpoint tangencies. A zero-length parameter interval
has no crossing contribution, but can still be boundary.

Shared exact projection and the regular-denominator gate run before normal
classification. The original ray query keeps its coincidence decline. For the
new contribution query, an identically zero normal means the entire curve lies
on the supporting line. Exact longitudinal root isolation then decides origin
incidence; otherwise its contribution is zero. A zero longitudinal polynomial
means boundary throughout. A denominator pole still declines, even when numerator
factors could cancel it. Degree, subdivision, cancellation, and typed input
failures retain their existing policy. No partial sum succeeds.

At an isolated normal root, the upper nonroot bracket bound certifies the local
normal sign. A point bracket instead uses the derivative at the root's certified
multiplicity. Multiplying by the denominator sign gives the rational normal side.
Parity determines the other side. No approximate root position or metric epsilon
enters these decisions. A zero local sign contrary to the root certificate is a
typed invariant failure, covered by a private unit test. Contributions fit i32:
the admitted polynomial degree is bounded by u16, and each root contributes at
most one unit.

## Independent evidence

The 16 retained ADR 0008 curve receipts provide exact normal roots, multiplicities,
longitudinal signs, and comparisons with both parameter bounds. The new
`tests/exact_roots/bezier_line/winding/derivation.jsonl` derives local signs from
the global root order: start with the normal's leading sign and reverse it for
every greater odd-multiplicity normal root. The already certified regular
denominator has the exact sign of its rational value at the lower interval
bound. This independent method uses neither production derivatives nor local
bracket evaluation. The compact TSV contains one boundary-or-contribution result
per curve. Reversed controls, weights, and parameter intervals must negate each
non-boundary result while preserving boundary.

The OCCT 8.0.0 generator adds `--winding` for its six ordinary curves. Native
contact parameters, native curve derivatives, and native endpoint positions
produce matching contributions. Its earlier default parameter and `--sides`
receipts remain byte-identical. Native approximate derivatives are supplementary;
they are not used to certify higher-order or near-zero cases. Retained rational
and algebraic receipts remain the independent exact evidence.

Further regressions cover square and diamond loops, a reversed inner square,
a self-crossing bow shape, edge and vertex seeds, cubic stationary crossings,
fourth-order contacts, horizontal origin crossings, horizontal tangency at the
origin, and a horizontal control hull that crosses the origin while its curve
does not. Generated checks cover every finite binary64 exponent field, both
scale signs, and globally negative weights. Cancellation is injected at every
observed checkpoint for isolated, tangent, coincident, and empty contributions.
Source bits survive success, failure, and cancellation. Coincident origin searches
retain degree and subdivision limits.

## Local validation and measured cost

```sh
cargo test -p rustycad-tkmath --all-features --locked
cargo clippy -p rustycad-tkmath --all-targets --all-features --locked -- -D warnings
cargo llvm-cov -p rustycad-tkmath --all-features --locked --test toolkit \
  --json --output-path /tmp/winding-coverage.json -- exact_
RUSTDOCFLAGS='-D warnings' cargo doc -p rustycad-tkmath \
  --all-features --no-deps --locked
cargo bench -p rustycad-tkmath --bench bezier_line --profile dev \
  --all-features --locked -- --winding
```

The full TKMath suite passes 13 unit, 2,817 integration, and 43 documentation
tests, with none skipped. The 95 selected exact-predicate integration tests
cover 1,244/1,256 predicate lines (99.04%). The winding modules cover 158/161
lines (98.14%); their invariant-decline branch also has a unit test in the full
suite. Shared ray preparation covers 22/22 lines. Duplication across all 20
predicate modules is 0%. Clippy, strict Rustdoc, scoped formatting, Markdown
lint, SPDX inspection, and whitespace checks pass.

The benchmark asserts all 16 accepted results on each of 1,000 repetitions
after warmup. Three runs each complete 16,000 contributions in 2.257, 2.152,
and 2.163 seconds in the timed loop. Peak process RSS is 5,096, 5,348, and
5,300 KiB respectively. These are local debug-profile measurements, excluding
build and oracle time. They are not optimized application or worst-case bounds.
The measured workload is fixed across the three runs; no faster result is
obtained by dropping difficult inputs.

Validation uses Linux x86_64 and rustc 1.99.0. Cross-platform execution remains
unverified. Exact input lifting, integer signs, bounded crossing sums, and ordered
root traversal introduce no floating threshold or pointer/map order. The new
primitive has no native OCCT or Jacobian runtime dependency.

## Remaining region work

This is a contribution for one supplied directed segment, not a closed-wire
validator or a face classifier. Production assembly still must certify endpoint
closure, connected orientation, curve extraction, and source identity. Explicit
overlap intervals are not constructed. Full analytic-region validation, measured
tolerances, split naming/history, and an end-to-end named region/prism remain under
issue #10. The application pin and production profile/extrusion behavior remain
unchanged. No sampled classifier is promoted into production acceptance.
Net changes in geometric recognizers, tolerance constants, fallback sites, and
geometric type assertions are zero. The existing sampled diagnostic classifier's
deletion gate remains the full region contract.
