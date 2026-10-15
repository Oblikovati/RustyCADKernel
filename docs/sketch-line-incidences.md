# Certified sketch line incidences

[ADR 0060](adr/0060-certified-sketch-line-incidences.md) adds
`rustycad_tkmath::predicates::roots::sketch_incidence` for application blocker
"Certify the incidences consumed by trim and extension". It certifies complete
bounded and supporting-line incidences, then selects trim and extension targets
from them. It is an extension; mapped OCCT intersectors are unchanged.

## Certified and open curve pairs

The source is always a line (two distinct points), as in the application's
`intersect_sketch_line`. Both the segment (trim) and support (extension) domains
are certified for every admitted target.

| Target representation | Status | Notes |
| --- | --- | --- |
| Segment, infinite line | Certified | Crossing, endpoint, parallel, collinear overlap |
| Circle, circular arc | Certified | Sector by boundary rays; ccw, cw, reflex, half turn |
| Ellipse, elliptical sector | Certified | Any nonzero major-axis vector; sector by rays |
| Control/fit B-spline | Certified | Open, closed (wrapped knots), handled triple knots |
| Rational B-spline | Certified | Strictly positive weights only |
| Arc or ellipse with angle bounds | Certified (ADR 0066) | Exact real angles; only a zero bound is an exact end |
| Circle, arc, ellipse, spline source | Certified (ADR 0066) | Against every target above; see ADR 0066 |
| Offset spline or curve | Open | Not admitted; needs its own exact equation |
| Native periodic knot vector | Open | Supply the unwrapped flat knots |
| Spline-source extension | Open | No application rule; support domain refused by name |

The application's arcs come from authored center and end points, so their rays
are exact. Its `elclib` arc span in radians is not an exact sector description.

## Results

Each contact reports: owning target key and index; outward bounds of the exact
line parameter and, for lines and splines, the target parameter; exact line and
target positions; crossing, touching or target-end transition; multiplicity on
each side; the constructed point, its coordinate bounds and its certified error.
Overlaps report exact target ranges, line ranges, end positions and orientation.
`ordered()` lists in-domain contacts in exact line order, with exact ties grouped.
`max_achieved_world_error()` and `refinements()` report the measured error bound
and the work used; requested options are retained unchanged.

## Evidence

- Hand-derived closed forms in `crates/tkmath/tests/sketch_line_incidences/`:
  dyadic and Pythagorean chords, irrational chords, tangency, sectors, ellipses,
  segments, overlaps, triple inflection, knot vertices, trimmed bounds, a rational
  quarter circle, irrational ties across circles and a spline, a closed seam,
  selection refusals and every failure class.
- 195 generated near-degenerate cases against the independent Python `Fraction`
  oracle (ulp-level tangency at scales 1e-9 to 1e9, endpoints, sector rays,
  circles 1–5 ulps apart, segment ends, near-parallel lines, spline extrema and
  knot points). The oracle replays byte for byte. See the
  [oracle README](../crates/tkmath/tests/sketch_line_incidences/README.md).
- 38 native OCCT 8.0.0 `Geom2dAPI_InterCurveCurve` cases and the mapped Rust
  intersector on the same corpus: equal counts, and every native point within the
  certified error plus 1e-8 of a distinct contact. 40 more cases use splines from
  the owned constructor. See the
  [native README](../crates/tkgeomalgo/tests/sketch_line_incidences/README.md).
- Internal checks: Cox–de Boor pieces equal independent exact de Boor points,
  resultant roots equal line parameters, Cauchy bounds contain roots, and common
  factors prove equality while nearby distinct roots separate.
- Eight concurrent threads produce reports equal to the single-thread report;
  reports keep their inputs after caller mutation; two separate test processes
  printed identical report text (8,927 bytes, SHA-256
  `8a505d8310cfae8107fce37e0a9ca64a19709c532f449fcedec93c96105e8485`).

## Executed checks

Local Linux checks on 2026-10-15, Rust 1.99.0, locked dependencies:

- `cargo test --locked -p rustycad-tkmath --all-features`: 249 unit, 3,010
  integration and 55 documentation tests passed; none ignored.
- `cargo test --locked -p rustycad-tkgeomalgo --all-features`: 7 unit, 509
  integration and 154 documentation tests passed; 49 declared ignored doctests.
- Clippy for both crates, all targets and features, with warnings denied: passed.
- TKMath rustdoc with warnings denied: passed. New and touched Rust files pass
  `rustfmt --check`. The workspace format script reports existing differences in
  untouched TKBRep, TKService and Vulkan test files.
- LLVM line coverage of the 22 new module files under the incidence tests:
  96.73% (1,777/1,837), regions 2,746/2,888, functions 252/255. This is
  new-module coverage, not toolkit coverage.

A release run of the 195 generated cases took 0.60 s wall time and 11,568 KiB
peak RSS; all 56 TKMath incidence tests took 0.94 s and 11,864 KiB. This is a
local diagnostic, not a portable timing gate or a comparison with a release.

## Remaining acceptance

No owner release, application intake or native edit test is claimed. The
application must still adopt the operation through its atomic endpoint ownership,
validate the final solved edit, and pass undo/reload and native tests. A seed
point can move during recompute; the final endpoint's incidence must be certified
against the solved geometry. The open pairs above remain blocked.

## Curve sources and angle bounds (ADR 0066)

[ADR 0066](adr/0066-certified-sketch-curve-incidences.md) extends the operation
to circle, ellipse, ray- and angle-bounded arc and spline sources against every
admitted target, with cyclic trims of closed sources, arc extension through the
full conic, parameter picks for splines and exact angle-bounded arcs. Evidence:
closed-form tests (`curve_sources`, `curve_selection`, `conic_pairs`,
`spline_pairs`, `angle_arcs`), 244 generated cases against `oracle_curves.py`
(see the [oracle README](../crates/tkmath/tests/sketch_line_incidences/README.md)),
76 native OCCT 8.0.0 cases and the mapped intersector (see the
[native README](../crates/tkgeomalgo/tests/sketch_line_incidences/README.md)), and
`curve_isolation` for eight-thread equality, input ownership and a process digest.
Named incomplete results: `UncertifiedOverlap`, `ExceptionalFiber`,
`KnotTangency`, `ExceptionalTangency`. No owner release or application intake is
claimed.
