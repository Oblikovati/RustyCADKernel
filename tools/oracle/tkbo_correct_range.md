# Edge/edge and edge/face range correction

Pinned OCCT: `3d097a0328e71b826377d4814ab05ec3c3d23871`.
Compile `tkbo_correct_range.cxx` against the installed OCCT headers and libraries
TKBO, TKTopAlgo, TKBRep, TKGeomAlgo, TKGeomBase, TKG3d, TKG2d, TKMath and TKernel.
Its output is `tkbo_correct_range.expected`.

Ninety-six cases cover lines, circles and two quadratic Bezier curves (one with
zero derivative at the first endpoint), four input ranges including reversed
and tiny ranges, three tolerance scales, and both overloads. Rust compares
both output endpoints within 1e-12.

The edge/edge overload leaves lines unchanged and uses twice the sum of edge
tolerances for other curves. The edge/face overload uses the face tolerance,
including lines. Each endpoint adjustment separately resets the entire output
to the source range if the remaining span falls below PConfusion; preserving
that order matters when only the second adjustment survives.

Rows remain partial for BSpline/offset/other curves, locations, exceptional
states and further numerical boundary cases. No legacy declaration is mapped
by these supplemental fixtures.
