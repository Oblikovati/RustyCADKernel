# ADR 0073: Analytic body and face properties

Status: implemented on a worktree branch (2026-10-15); public in `rustycad-kernel`.

## Decision

`rustycad_kernel::properties::{body_properties, face_properties}` measure any owned
B-rep shape from its own data. Results are owned values (`Send + Sync`, no live shape):
every scalar is a `Measured { value, error_bound }`, every point a `MeasuredPoint`, and
each result names its weakest `IntegrationMethod`.

One formulation covers every face. With the face taken forward, Green's theorem turns
each surface integral over the parameter domain into `sum over trims of integral
F(u(t), v(t)) v'(t) dt`, where `dF/du` is the integrand. Eight integrands are carried
together about one reference point `Q` (the vertex mean): area, area moment, the flux
`(P - Q).(Pu x Pv)` and the flux-weighted moment. The divergence theorem then gives the
volume (`1/3` of the oriented flux sum) and the volume moment (`1/4` of the
flux-weighted sum, from `div((P-Q)((P-Q).a)) = 4 (P-Q).a`). Reversed faces subtract
their flux; internal and external faces bound no volume.

| Surface | `F` | Trims in closed form |
| --- | --- | --- |
| Plane | `c1 u + c2 u^2 + c3 u v` | lines (polynomials), circles (trigonometric polynomials) |
| Cylinder, cone | harmonics of `u` times polynomials of `v` | iso-`u` and iso-`v` lines |
| Sphere, torus | harmonics of `u` times trigonometric polynomials of `v` | iso-`u` and iso-`v` lines |

The revolution surfaces share `P = O + r(v)(cos u X + sin u Y) + h(v) Z` with constant
speed `s = |(r', h')|`, so `|Pu x Pv| = s r` and `Pu x Pv = e r (h' (cos u X + sin u Y)
- r' Z)` with `e = +1` for a direct frame. Two small exact algebras (`Poly`,
`TrigPoly`) form every product and antiderivative. This covers every edge of the box,
cylinder, cone, sphere, torus and of extruded line/arc profiles (seams, generators,
degenerate pole edges, circular caps, annulus holes).

`ClosedForm` bounds cover floating-point rounding only: 64 unit roundoffs times the sum
of the absolute antiderivative terms, a forward error model, not interval arithmetic.
Other trims on those surfaces (ellipses, splines, helical lines) use adaptive
Gauss-Kronrod 7/15 (`AdaptiveQuadrature`); free-form surfaces nest it for `F` from the
surface derivatives, integrating the inner estimates along the trim with a factor of
two. Those bounds are estimates, named as such.

A face whose wires do not close in its parameter plane (a periodic face without seam
edges) cannot use Green's theorem; it falls back to the ported fixed-order
`BRepGProp` integration with an infinite bound (`NativeUnbounded`), and so does the
whole body containing it.

Face results also give the surface kind, the face orientation and one `LoopSummary`
per wire: edge uses, signed parameter area (closed form for line and circle trims) and
the outer/inner role from its sign.

## Native quirk kept out

`BRepGProp::*_properties_eps` is not used as evidence. Natively (DRAWEXE 8.1.0,
`vprops p 1e-9`) and in the port it reports a relative error of 7.4e-17 for an
elliptic prism (`a = 2`, `b = 1`, `h = 3`) while missing the volume by 4e-7 and the
area by 8.5e-3; its centroid is off by 7e-4. The fixed-order variant is close
(18.8496, 41.6327) but returns no error. Both remain ported unchanged.

ADR 0063's certified annular volume is not reused: it is conditional on ADR 0061's
nested polynomial loops and their orientation certificate. This module gives the
general path; the annular prism test checks the same closed form.

## Verification

Independent textbook oracles, each asserting `|value - exact| <= error_bound` and a
tight bound (`<= 1e-11` relative): box, cylinder, sphere, pointed cone and frustum
(volume, area, centroid `h (r1^2 + 2 r1 r2 + 3 r2^2) / (4 (r1^2 + r1 r2 + r2^2))`),
torus (Pappus), annulus prism; face areas and centroids of the cylinder lateral face
and caps, sphere, cone lateral face (centroid `h/3`) and torus; annulus cap loops.
A rotated and translated torus keeps its volume and its centroid follows a moved
sphere's. The elliptic prism checks the adaptive and nested paths against `pi a b h`
and against the native fixed-order area. Results are bit-identical across four
threads. Unit tests check the algebras pointwise, the product-to-sum table, the
quadrature estimate and its refusal of a singular integrand, and the centroid bound.

## Limits

Seamless periodic faces use the unbounded native path. The closed-form bound is a
model bound, not an interval enclosure; certified enclosures in the sense of ADR 0063
for general profiles remain future work. Second moments (inertia) are not exposed.
