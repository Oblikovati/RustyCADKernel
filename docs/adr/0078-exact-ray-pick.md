# ADR 0078: Exact ray picking with stable face indices

Status: accepted (2026-10-15). Consumer: the application's "first cube" slice
(face selection under the cursor).

## Decision

`rustycad_kernel::pick::ray_pick(&Shape, &PickRay) -> Result<PickResult,
BodyQueryError>` intersects a half-line with every face of the body and returns
exact B-rep hits.

- `PickRay::new(origin, direction)` requires finite coordinates and a non-zero
  direction, which it normalizes. Anything else is an `InvalidRay` refusal.
  `distance` is measured from the origin along the unit direction over
  `[0, Precision::Infinite()]`.
- Each face, in stable face order (ADR 0076), is intersected by the ported
  `IntCurvesFace_Intersector` with `Precision::Confusion()`, restricted to the face
  (`aRestr = true`) and classified with the edge tolerances. Planes, cylinders,
  cones, spheres and tori are solved in closed form (IntAna). A hit carries the
  stable `FaceIndex`, `distance`, the surface parameters `uv`, the `point` and
  `on_boundary` (state ON).
- Hits are sorted by `PickHit::pick_order`: distance, then face index, then u,
  then v, all as total-order floats. Hits tied at a shared edge or vertex are
  reported once per face, in face-index order. A seam crossing is reported once.
- `coplanar_faces` lists planar faces whose plane contains the ray (parallel and
  within confusion). Such an overlap is a segment, so it gives no point hits.
- `completeness` is `Analytic` when every face is one of the closed-form kinds.
  Otherwise it is `SampledCandidates`, because IntCurvesFace seeds free-form faces
  from a sampled polyhedron and a crossing can be missed. That result is reported,
  never presented as complete.

### Why the presentation mesh does not seed the pick

The request suggested refining from a mesh hit. We did not: for the closed-form
surface kinds, the exact intersection is complete without any seed. A mesh seed
can only lose hits. Chord sag means a ray near a silhouette or a grazing tangent
can meet the exact surface but miss every triangle, or the reverse. Picking
therefore reads only the exact B-rep and never depends on display tolerances.
Faces are keyed by the same stable indices as `tessellate`, so a pick selects the
matching presentation group. For free-form faces, seeding from a certified
enclosure (not the display mesh) is the open follow-up recorded by
`SampledCandidates`.

## Evidence

`crates/kernel/tests/pick.rs` covers the same fixtures as ADR 0077. Each hit is
checked three ways: it lies on the ray (1e-12), the face surface evaluated at its
`uv` equals the point (1e-9), and it lies on an independent closed-form fixture
surface. The tests are:

- Closed-form distances on box faces (three axes), sphere chords, a sphere ray from
  inside (forward root only), cylinder chords, a seam crossing (once), torus
  (4 walls, and 2 through the tube) and annulus prism walls and caps.
- Misses: just outside a sphere, a ray pointing away, and down the annulus hole.
- Grazing: a sphere tangent and a cylinder tangent give one hit each.
- A ray in the plane of a box face gives that face as coplanar and two boundary
  hits on the crossed faces.
- A corner hit gives three equal-distance boundary hits in face-index order, with
  identical results on repeat.
- Pick face keys match the presentation face groups.
- Invalid-ray and null-body refusals.
- 8 threads on one shared body give identical results.

## Limits

Free-form faces are `SampledCandidates`. Hits behind the origin are excluded, apart
from those within the confusion tolerance that IntCurvesFace admits. Picking edges
and vertices directly (rather than faces) is not part of this decision.
