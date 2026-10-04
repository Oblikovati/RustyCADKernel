# ShapeFix_Wire 2D range intersection

OCCT oracle: `3d097a0328e71b826377d4814ab05ec3c3d23871`, optimized native modeling build.

`line_cross` puts two planar line edges in one wire with a 0.2-unit p-curve gap.
`FixGap2d(2, false)` intersects their extended line domains at `(0.8, 0)`,
extends the first p-curve to parameter `0.8`, and extends the second backward to
parameter `-0.28284271247461901`. `reversed_line_cross` confirms that a reversed
first edge keeps its native range and endpoint ordering. `parallel_projection` covers parallel edges
whose closest endpoints are diagonally offset; the two p-curves extend to
parameters `1.05` and `-0.05`. Both rows record the return, last-fix status,
context state, ranges and endpoint coordinates. `trimmed_surface_clip` uses a
plane trimmed to `[0,1]×[0,1]`. The proposed nearest point is outside U, but its
surface-bound intersection lies beyond the candidate-to-original endpoint
segment; OCCT keeps that candidate range, matching its clipping loop's finite
parameter domain. `surface_boundary_crossing` places the p-curve intersection
beyond U=1 while the corresponding trim-boundary hit lies on the candidate
segment. OCCT extends the first p-curve to U=1 and starts the second at
parameter `0.5656854249`, preserving its native untrimmed line parameterization.

`single_circular_arc` and `single_elliptical_arc` cover OCCT's same-edge conic
branch, which expands both parameter ends by `PI - gap/2`; the circular case's
separate WireData and Wire rows agree. `intersecting_circles` exercises the full
`[0,2π]` conic search domains and the asymmetric nearest-endpoint selection.
`intersecting_ellipses` records OCCT's chosen ellipse event: the first range stays
unchanged while the second p-curve extends backward by about `0.1`.
`disjoint_circles_extrema` exercises the fallback path after the conics have no
intersection: `Geom2dAPI_ExtremaCurveCurve` finds their nearest points, then the
periodic parameter adjustment chooses OCCT's wrapped ranges.
`trimmed_line_pcurves` confirms that range replacement unwraps trimmed line
bases for intersection, then retains the trimmed wrappers on both updated
p-curves. `offset_line_pcurves` checks the corresponding offset-curve path with
nonzero offsets; the selected intersection is interior to both parameter
ranges, so both existing ranges move inward while the wrappers remain offsets.
`trimmed_offset_line_pcurves` nests those wrappers and confirms replacement
preserves both the trimmed outer curve and its offset basis.
`bezier_line_pcurves` covers OCCT's bounded Bezier domains and verifies that
the candidate intersection changes both range endpoints without replacing the
Bezier dynamic type.
`bspline_line_pcurves` provides the matching degree-one B-spline case, including
the source knot interval `[0,1]` and preserved B-spline representation.
`multispan_bspline_line_pcurves` adds a quadratic spline with an internal knot
at `0.5`; its selected crossing lies before the knot at `0.4436491673`.
`tangent_parabola_line` checks a repeated/tangent intersection from a quadratic
Bezier curve against a line; OCCT selects the common parameter `0.5` and trims
both p-curves to that point.
`tangent_bspline_line` repeats the tangency through a degree-two B-spline so
the spline intersection path is exercised independently of Bezier dispatch.
`parabola_line_extension_exception` records an upstream `StdFail_NotDone` from
the analytic parabola fallback. OCCT has already created its reshape context,
but leaves both p-curve ranges and DONE/FAIL status unchanged; the Rust test
compares this failure class and retained state instead of counting the attempt
as a successful repair.

These fixtures cover the direct intersection, parallel-curve extrema/projection
fallback, finite-surface bounds handling, trimmed and offset wrappers, and
copied-edge/context replacement paths. Higher-degree spline cases remain
uncovered.

Rebuild with:

```sh
g++ -std=c++17 -I target/occt-modeling-full/include/opencascade \
  tools/oracle/tkshhealing_wire_gap2d_ranges.cxx \
  -L target/occt-modeling-full/lin64/gcc/libi \
  -Wl,-rpath,target/occt-modeling-full/lin64/gcc/libi \
  -lTKShHealing -lTKBO -lTKBool -lTKPrim -lTKTopAlgo -lTKBRep \
  -lTKGeomAlgo -lTKGeomBase -lTKG2d -lTKG3d -lTKMath -lTKernel \
  -o /tmp/tkshhealing-wire-gap2d-ranges
/tmp/tkshhealing-wire-gap2d-ranges
```
