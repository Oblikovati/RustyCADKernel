# `Poly_Triangulation::HasGeometry` virtual dispatch probe

Pinned OCCT revision: `3d097a0328e71b826377d4814ab05ec3c3d23871`.
The Rust gap was observable because `RWMesh_TriangulationSource::HasGeometry()`
counts either triangles or its edge array after requiring nodes. The native probe
derives an edge-backed triangulation with that exact predicate, checks its base
and override results, then confirms `MinMax(..., accurate=true)` bypasses a
cached box for a rotated query. It verifies only the virtual geometry predicate
and resulting cache decision; the fixture does not model edge topology.

Source SHA-256 values:

* `Poly_Triangulation.hxx`: `885f46dcacaf7f3260c347f87ff9de1a4929362671229fcf0080bf8e89d8f802`
* `RWMesh_TriangulationSource.hxx`: `27912dd0f6c5e76346a574d56773254a97678e351b58ac7f38cca733cc84203b`

The installed OCCT header matches the pinned `Poly_Triangulation.hxx`; the probe
links `libTKMath.so.8.0.0` and `libTKernel.so.8.0.0` with hashes recorded in
`porting/audits/poly-triangulation-has-geometry.json`.

Reproduce with:

```sh
c++ -std=c++17 -O0 -g0 \
  -I/home/vmiguel/git/oblikovati-workspace/occt-install/include/opencascade \
  tools/oracle/poly_triangulation_has_geometry.cxx \
  -L/home/vmiguel/git/oblikovati-workspace/occt-install/lib \
  -Wl,-rpath,/home/vmiguel/git/oblikovati-workspace/occt-install/lib \
  -lTKMath -lTKernel -o /tmp/poly-triangulation-has-geometry
/tmp/poly-triangulation-has-geometry
```

`TriangulationHooks::has_geometry` overrides the base rule, while
`has_geometry_base` provides explicit base-qualified evaluation. The Rust replay
checks that an edge-only geometry override changes the accurate rotated MinMax
result from cached bounds to transformed node bounds.
