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
parameter domain.

`single_circular_arc` covers OCCT's same-edge conic branch, which expands both
parameter ends by `PI - gap/2`; its separate WireData and Wire rows agree.

These fixtures cover the direct intersection, parallel-curve extrema/projection
fallback, finite-surface bounds handling and copied-edge/context replacement
paths. Actual boundary crossing and non-line domain choices remain uncovered.

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
