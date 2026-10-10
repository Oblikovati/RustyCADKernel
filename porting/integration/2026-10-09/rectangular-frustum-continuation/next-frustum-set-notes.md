# Next port: SelectMgr_TriangularFrustumSet

Inspected against OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871. No set source has
been implemented in this checkpoint. Continue in `/tmp/rustycad-resume` on
`porting/resume-claude`; its rectangular-frustum checkpoint is validated separately.

## Existing dependency seams

- `SelectMgrTriangularFrustum` and `SelectMgrFrustumCore<3>` provide all child geometry.
  Child `overlaps_point` deliberately returns false, so native
  `isPointInsideAnyFrustum` must invoke the child's core `has_point_overlap` directly.
- `crates/tkmesh/src/brep_mesh/delaun/api.rs` already has `with_indices(mesh,
  &mut indices, (-1,-1))`, and `batch_insertion.rs` implements perform/insertion.
  The comment in `delaun/mod.rs` saying batch insertion is unported is stale;
  inspect the actual bodies and existing differential corpus before relying on it.
- The mesh structure has allocator construction, add_node(vertex,false), add_link,
  elements_of_domain, get_element and element_nodes. Preserve native PackedMap
  iteration order when creating children. Do not replace Delaun by a fan or ear clip.
- TKV3d currently depends only on TKernel, TKMath, TKG3d and TKService. Its native set
  needs TKMesh and TKGeomAlgo (`GeomIntIntSS` for plane/plane intersection). Add the
  corresponding owned crate interfaces and regenerate Cargo.lock when implementing.
- `GeomIntIntSS::with_surfaces` / `perform` already exist under TKGeomAlgo; inspect
  their plane/plane results and line orientation, since circle-boundary arithmetic
  depends on that orientation. GeomPlane, GeomCircle and GeomLine are available.
- Native set has no RTTI declaration and inherits BaseFrustum. Extend the existing
  inherited-intersector macro in `axis_runtime_type.rs` to expose the Rust concrete,
  BaseFrustum and BaseIntersector projections without changing native identity.

## Construction and sharing

Init retains an HArray1 handle and resizes/assigns it in place, then selects Polyline.
Copies share that handle; later Init on one copy can change the other's rebuild
pixels. Preserve array lower bounds. Build's native assertion uses OR:
selection == Polyline OR points != null. It clears children, projects near and far
boundary arrays, then meshes the input polygon with frontier vertices/edges.
Several native next-index expressions use `(index % length) + 1`, implicitly assuming
one-based bounds; test non-one-based arrays and retain actual failures rather than
silently normalizing them. Collinear/short input can leave no triangles.

ScaleAndTransform transforms children and all boundary points, shares input points,
adopts the supplied builder, and does NOT copy myToAllowOverlap (fresh false).
Child triangular transforms ignore the supplied builder while the set adopts it.
CopyWithBuilder requires non-null builder, copies children awaiting reconstruction,
shares points, copies overlap mode, and leaves its boundary array empty. Pixel
tolerance is not copied. IsScalable is false; DetectedPoint always throws.
GetPlanes clears once and then appends each child's five equations. DumpJson writes
explicit set class, inherited BaseFrustum and each child in list order.

## Overlap distinctions

- Most methods assert Polyline and use the first matching child, retaining the
  child's untouched pick fields and ignored clips.
- Box inside-pointer overload first finds any overlap; unless overlap is allowed
  or inside is null, all eight corners must pass SAT in some child or inside is
  ANDed with false. It still returns true for partial overlap.
- Polygon/segment/triangle pick overloads reject crossings of the outer boundary
  when overlap detection is disabled. Boundary triangles inherit the child's false
  result; unknown triangle sensitivity inherits its true behavior before boundaries.
- Sphere pick overload simply delegates to children. Sphere inside overload ignores
  its output pointer, builds a projection plane using the native farthest-point
  selection, computes angle sums and boundary/sphere tests. It has unusual indexing
  (the final near vertex's successor is the first far vertex); preserve it.
- Cylinder and circle inside-pointer overloads access the first child before any
  selection assertion. Empty sets need a deliberate Rust Failure contract for the
  native invalid dereference/list access; avoid crashing the differential fixture.
- Cylinder containment projects six silhouette points and uses child isDotInside;
  the cylinder axis is normalized before using height (transform scaling is not
  applied to height). Then it may fall back to child cylinder overlap.
- Circle inside-pointer overload computes some unused projections before branching.
  Preserve possible failures. Boundary crossing clears inside only in the unfilled
  branch; no matching child clears inside at the end.
- The private circle boundary helper constructs circle and plane objects, intersects
  each boundary face plane using GeomIntIntSS, and computes line-circle crossings.
  Even the unused circle construction can fail for an invalid radius.

## Differential coverage to add

Both camera types and depth conventions; convex, concave, reversed, repeated,
collinear and short polylines; allowed/full containment modes; point SAT versus
child point-only false; copied/rebuilt/shared input mutation; transformed children
and set camera mismatch; all inside flags and seeded pick fields; circle plane
parallel/tangent/crossing cases; empty child lists; array lower-bound failures;
all dump depths and child plane ordering. Private predicates need indirect tests
through the boundary queries. Keep only process-local addresses canonicalized.
