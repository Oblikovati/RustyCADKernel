# BeanFace coincidence evidence

Pinned OCCT: `3d097a0328e71b826377d4814ab05ec3c3d23871`.

`tkbo_bean_coincidence.cxx` observes the unchanged native private
`TestComputeCoinside` implementation through header access exposure. Its 24
rows cover four line/plane configurations, three initial flag patterns and
repeated calls. The Rust range-walk unit test compares the returned boolean,
range endpoints and flags against the captured output.

`tkbo_bean_coincident_curves.cxx` exercises public `Perform` for a reversed
circle and a Bezier curve on a plane, twice each. The Rust integration test
compares all four result snapshots with the native output.

These are supplemental fixtures, not newly mapped legacy declarations.
General exact intersections, extrema, localization and boundary refinement
remain separate parity obligations. Passing these fixtures does not prove
arbitrary curve/surface coincidence or full BeanFace parity.
