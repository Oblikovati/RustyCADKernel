# ADR 0083: Ellipses, elliptic arcs and B-splines in planar regions

Status: implemented on a worktree branch (2026-10-15); integration and release are the
coordinator's.

## Context

ADR 0068 declined `ProfileCurve::Ellipse` and `ProfileCurve::Spline` by name. The region
proof (ADR 0066, now the plane graph proof of ADR 0084) certifies any polynomial or
positive-weight rational Bézier pieces, and ADR 0067 certifies ellipse and spline
incidences; what was missing was turning these curves into proof pieces, a native carrier
and history.

## Decision

Public input (`profile_extrusion`, re-exported by `rustycad_kernel::modeling`):

- `Ellipse { id, center, major_axis, minor_radius }`: a closed curve with no authored
  vertex; its seam (native parameter 0) is the end of the major semi-axis.
- New `EllipticArc { id, start, end, center, major_axis, minor_radius, sense }`: both ends
  within `sew` of the ellipse (measured at the point's affine parameter, an upper bound of
  the distance).
- `Spline { id, start, end, degree, poles, weights, knots }` (fields added): a clamped
  polynomial or rational B-spline with a flat knot vector of `poles + degree + 1` values;
  equal end ids give a closed spline. The end poles must lie within `sew` of the vertices;
  the admitted (and published) end poles are the vertex points.

Proof pieces:

- An elliptic piece is the affine image `c + a x u + b y v` of the circular piece of the
  same parameter interval. Affine maps keep rational Bézier weights, so the quarter form
  (corner on the diagonal, weights `1, 1, 2`) and the general form (weights
  `1, cos h, 1`) of ADR 0068 carry over; arcs are split into at most quarter turns, and the
  authored vertices are the exact piece ends.
- A spline is split at its knots into Bézier pieces by knot insertion (The NURBS Book,
  A5.6) in homogeneous coordinates; adjacent pieces share one computed junction control
  bitwise, and the clamped ends are the vertex points.

Native carriers: `gp_Elips` (parameter increasing in the arc sense, as for circles) and
`Geom_BSplineCurve` (distinct knots and multiplicities). Vertex tolerances cover the sew gap
of elliptic arc ends as they do for circular arcs. History is unchanged: a full ellipse's
seam vertex is attributed to its curve.

Areas: elliptic loops have the closed form `1/2 (c x (end - start) + a b sweep)` per arc,
which cross-checks the certified area; spline loops have none. Every non-line region is
`IntegrationMethod::RepresentationEnclosure` (ADR 0081): the certified enclosure of the
pieces plus `64 eps max(|area|, scale)`, where a spline piece's scale is its largest control
distance times its control polygon length.

Named declines: `UnsupportedCurve` for `"unclamped spline"` and `"spline above degree 16"`
(the certified integrals take degree 16); new `InvalidCurve { curve, reason }` (degree 0,
too few poles, knot count, decreasing or empty knots, multiplicity above the degree,
nonpositive weight, minor radius above the major radius); new
`EndOffCurve { curve, vertex, distance, sew }`; `DegenerateCurve` and `NonFiniteCurve` as
for circles.

## Evidence

`profile_extrusion::curves` (6 tests): a tilted ellipse (`a = 2`, `b = 0.5`) is one region of
area `pi` (ten pair domains) and extrudes to an elliptic cylinder of volume `3 pi`
(`V2 E3 F3`, valid, seam history); a half ellipse closed by its diameter has area `3 pi`; a
tilted elliptic segment between parameters 0.3 and 2.4 has `a b (dt - sin dt) / 2` and its
prism volume; a two-span quadratic spline of the parabola `y = x (4 - x) / 2` closed by its
base has area `16/3` and prism volume 8 (four faces, valid); a rational two-span spline of
the half circle has area `pi/2` and prism volume `pi`; repeated builds are identical;
named declines for each rule. Kernel `modeling_tests::ellipse_and_spline_prisms_match_body_properties`:
`body_properties` (adaptive quadrature on the B-rep, independent of the proof) agrees with
the closed-form volumes and centroids (`(1, -2, 1.5)`, `(2, 0.8, 0.75)`,
`(0, 4/(3 pi), 1)`). Unit tests: ellipse parameter inversion and Green terms; knot insertion
reproduces de Boor's junction `C(0.5) = (2.75, 1.5)` and keeps a rational circle on the unit
circle.

## Limits

Periodic or unclamped splines and splines above degree 16 are declined. The representation
error of rounded pieces is estimated, not proven. A tilted ellipse's quarters are certified
by the generic pair route because their rounded corners leave the exact common conic.
