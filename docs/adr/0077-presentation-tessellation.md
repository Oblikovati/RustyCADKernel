# ADR 0077: Generic presentation tessellation of a body

Status: accepted (2026-10-15). Consumer: the application's "first cube" slice.
The application meshes only its prism path today
(`oblikovati-rust/crates/martensite/src/prism/mesh.rs`, keyed by prism roles).

## Decision

`rustycad_kernel::presentation::tessellate(&Shape, &DisplayOptions) ->
Result<PresentationMesh, BodyQueryError>` meshes any body for display.

- `DisplayOptions::new(linear, angular)` takes absolute display deflections (model
  units, radians). They are separate from modelling tolerances. The linear value
  must be finite and at least `Precision::Confusion()`. The angular value must be
  in `[Precision::Angular(), PI]`. Anything else is an `InvalidOptions` refusal.
- The body is copied with `BRepBuilderAPI_Copy(copyGeom = true, copyMesh = false)`
  and the copy is meshed by the ported `BRepMesh_IncrementalMesh`, sequentially
  (`in_parallel = false`). The body, its history and any mesh it already carries
  stay unchanged. The result does not depend on earlier meshing of the body.
- Faces and edges of the copy are found through the copy history
  (`Modified`, exactly one image), never by position. The history image carries no
  context orientation, so each image is replaced by its occurrence in the copy's
  own enumeration (ADR 0076). A FORWARD/REVERSED face orientation then sets the
  triangle winding and the normal side.
- `PresentationMesh` is plain owned data (`Send + Sync`):
  - `faces`: one `FaceTriangles` per stable `FaceIndex`, with face-local
    positions, unit normals (`BRepLib_ToolTriangulatedShape::ComputeNormals`,
    location applied, flipped for REVERSED), counter-clockwise `[u32; 3]` triangles,
    and the mesher's `sampled_deflection_estimate`.
  - `edges`: one `EdgePolyline` per non-degenerated `EdgeIndex`. It comes from the
    polygon on the triangulation of the first face (stable order) bounding the
    edge, so its points are vertices of that face mesh.
  - `degenerated_edges` (poles, apexes) and `vertices` (stable order).
  - `requested` options and `effective` settings (the completed mesher
    parameters: interior deflections, minimum size). These are separate from
    measured error.
- `canonical_bytes()` encodes the mesh as little-endian bytes, so results can be
  compared byte for byte.
- These are refusals, never partial results: mesher status flags other than 0, a
  face without triangles, a non-degenerated edge without a polygon, a face
  orientation other than FORWARD/REVERSED, non-finite nodes, or an incomplete copy
  history.

Tessellation is presentation only. Following AGENTS.md, it is never modelling or
acceptance evidence. Area and volume evidence stays with error-bounded integration.

## Evidence

`crates/kernel/tests/presentation.rs` and `presentation_determinism.rs` cover the
tkprim box 2x3x4, cylinder r1 h2, sphere r1.5, torus 3/1 and an annulus prism
(radii 1 and 2, height 1.5). The oracles are independent closed forms
(`tests/support/analytic.rs`) that use no kernel geometry:

- Each face matches exactly one fixture surface: every vertex lies within 1e-7 of
  it and every vertex normal is within 0.99 of its outward normal.
- Every triangle centroid lies within the requested deflection (0.1 and 0.01) of
  its matched surface.
- Welded by exact coordinates, every directed edge is used once and its opposite
  once, so each closed body is watertight, manifold and consistently wound.
  Triangle winding agrees with the vertex normals, and the signed volume is
  positive.
- Mesh area and volume converge to the closed forms within `2 d / R` (volume
  `3 d / R`) for d = 0.1, 0.01 and 0.001. The error decreases on every curved
  fixture.
- Edge polylines lie on face-mesh vertices. The sphere lists 2 degenerated edges
  and the torus 2 seams and 1 vertex.
- Bytes are identical across repeated calls, separately built bodies, 8 threads
  sharing one body, and two child processes.
- The body has no triangulation after tessellation. An existing coarse body mesh
  is kept and does not change the result.

## Limits

Normals at singular points (sphere poles) fall back to averaged facet normals,
as in OCCT. The deflection check samples centroids only. It is not a certified
bound, and `sampled_deflection_estimate` is the mesher's own estimate.
Free-form faces are meshed by the same pipeline, but this ADR has evidence only
for planes, cylinders, spheres and tori.
