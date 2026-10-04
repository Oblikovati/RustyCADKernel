# BeanFace general-stage public replay

Pinned OCCT: `3d097a0328e71b826377d4814ab05ec3c3d23871`.

The native probe executes `Perform` twice for six geometries: a line crossing
a sphere, a line crossing a cylinder, a cylinder tangency, a near-tangent line,
a circle crossing a plane, and a translated near-tangent line/cylinder pair.
Twelve snapshots capture completion, context,
minimum-distance sentinel classification and result ranges at twelve decimal
places. Values rounded by this fixture are not a bitwise floating-point proof.

This fixture is supplemental. Localized Bezier/high-degree B-spline/other
surface searches remain separate obligations; matching these analytic surface
examples does not establish full BeanFace or legacy-test parity.

The translated near-tangent fixture failed before the extrema stage selected
`BRepTool::curve`, matching the native overload which applies the edge location.
Reading only `curve_with_location().curve` discarded that transform, producing
no ranges and retaining the distance sentinel. The regression now matches the
native range for both calls.
