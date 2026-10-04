# Existing pcurve attachment

Oracle: OCCT `3d097a0328e71b826377d4814ab05ec3c3d23871`.
Compile `tkbo_attach_existing.cxx` with the installed OCCT headers and link
TKBO, TKTopAlgo, TKBRep, TKGeomAlgo, TKGeomBase, TKG3d, TKG2d, TKMath and TKernel.
The default output is `tkbo_attach_existing.expected`; argument `seam` emits
`tkbo_attach_seam.expected`.

One hundred cases compare status, pcurve presence, edge tolerance, SameParameter,
SameRange, ranges, UV values and maximum vertex tolerance for five supports, four orientations, and
original/reversed/displaced/missing-pcurve/small-offset inputs. Eight seam cases cover
cylinder and torus in every orientation. Numeric comparison tolerance is 1e-10.

The native static `IsClosed` uses `More() || count == 2`. After two occurrences,
`TopExp_Explorer::Next` restarts traversal; these seam fixtures finish with four
occurrences and return false. Rust deliberately preserves this control flow.
`tkbo_attach_closed.cxx` includes the pinned implementation translation unit to
probe both static helpers directly (add the source BOPTools directory to the
compiler include paths). Its output `0 0 1` means predicate false, direct update
status zero, and a closed seam pair after the update. A Rust unit test checks
those same results. This is distinct from the public attachment path.

The API remains partial: located/general spline geometry, failure statuses
2/3/5/6, exceptional mutation states, general vertex mutation ordering, and direct
closed-helper branch coverage beyond the cylindrical fixture remain unproven.
These supplemental fixtures do not map additional legacy declarations.
