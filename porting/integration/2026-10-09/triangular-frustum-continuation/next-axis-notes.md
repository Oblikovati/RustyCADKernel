# Next selecting volume: native constraints found during triangular validation

Source: pinned OCCT 3d097a0, SelectMgr_AxisIntersector.hxx/.cxx. No AxisIntersector
implementation or mapping was added in the triangular-frustum checkpoint.

- Init sets selection type Point and axis. Build and SetCamera do nothing.
- CopyWithBuilder and ScaleAndTransform assert initialized Point selection type.
  Identity-form transform returns a fresh, uninitialized default axis; a numeric
  identity matrix with Other form must take the transform branch instead.
- GetNearPnt, GetFarPnt and GetViewRayDirection assert initialization too. Current
  SelectMgrBaseIntersector getter signatures are infallible; extend them to carry
  Failure and update callers/tests/probes rather than discarding the native errors.
  Existing base-default bodies can return Ok; check downstream selecting manager
  interfaces as part of that work. Only about 24 getter references exist in TKV3d.
- AxisIntersector declares no RTTI macro, so its native DynamicType and static type
  descriptor are inherited from BaseIntersector. A standalone native probe confirmed this (see axis-contract-probe.cxx and
  axis-contract-probe.log). Rust project_interface currently gets casters exclusively from
  Transient::dynamic_type(), so handle projection and inherited native RTTI may need
  a separate Rust interface-descriptor path. Do not invent observable native RTTI.
  TriangularFrustum DOES have its own RTTI and its current ancestry matches native.
- BVH ray-box intersection is already ported as
  rustycad_tkmath::bvh::ray_box_intersection_by_corners over BvhRay.
- Point at axis origin fails direction construction. Point overlap only accepts
  co-directed points; pick overload writes depth/point before clipping rejection.
- Segment/plane helpers invalidate pick depth/point/normal on parallel, behind-ray
  or outside-segment rejection; center distance is retained by Invalidate.
- Polygon Interior intersects its plane without checking polygon containment.
  Boundary invalidates output, checks wrapped edges, retains nearest successful
  pick, then applies clipping. Unknown sensitivity only applies clipping to old pick.
- Triangle Interior handles degenerate point/segment, parallel-plane nearest edges,
  barycentric interior and nearest infinite-edge fallback; normal writes have exact
  timing relative to miss and clipping. Boundary includes a duplicate closing vertex.
- Sphere and box picks build the native Bnd_Range (which may be void), use output-
  preserving GetMin, then GetNearestDepth. Preserve native std::max argument order.
- Cylinder/cone and circle transform the axis into local coordinates. Depth remains
  in local units despite transformed picked point; do not apply a scale correction.
  A single cylinder ray crossing initializes missing leave to native zero.
- Sphere/cylinder/circle inside flags are ANDed with nonnegative entry only after a
  successful intersection. Filled-circle pick clamps negative time to zero before
  clipping, whereas inside-query reports entry sign. Surface normals follow native
  cap/side branches and transform direction versus vector semantics.

The native probe also confirms uninitialized GetNearPnt raises Standard_ProgramError,
and identity ScaleAndTransform returns selection type -1 while CopyWithBuilder
returns Point (0). These checks used the same pinned native build as the suite.
