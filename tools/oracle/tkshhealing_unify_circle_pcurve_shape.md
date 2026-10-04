# Circular p-curve union on a nonplanar spline face

The probe constructs a valid face on the quadratic surface `z = x² + y² − 0.25`.
Its boundary is one 3D circle represented by two edge arcs. Each edge stores the
same 2D circle handle over its own half-range. `ShapeUpgrade_UnifySameDomain::Build`
merges the arcs, then `UnionPCurves` detects that the merged p-curve range has the
wrong span, projects the merged 3D circle to the spline surface, and stores a
single `Geom2d_BSplineCurve` over `[0, 2π]`.

The output verifies the resulting p-curve class, range, endpoint values, topology
counts, and BRep validity against the Rust integration fixture
`pcurve_circle::circular_uv_arcs_merge_on_nonplanar_bspline_surface`.

Run against the pinned OCCT build:

```sh
g++ -std=c++17 -I target/occt-modeling-full/include/opencascade \
  tools/oracle/tkshhealing_unify_circle_pcurve_shape.cxx \
  -L target/occt-modeling-full/lin64/gcc/libi \
  -Wl,-rpath,target/occt-modeling-full/lin64/gcc/libi \
  -lTKShHealing -lTKPrim -lTKBO -lTKTopAlgo -lTKBRep -lTKGeomBase \
  -lTKG2d -lTKG3d -lTKMath -lTKernel \
  -o /tmp/tkshhealing-unify-circle-pcurve-shape
/tmp/tkshhealing-unify-circle-pcurve-shape
```
