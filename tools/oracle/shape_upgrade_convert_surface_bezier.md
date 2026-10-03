# `ShapeUpgrade_ConvertSurfaceToBezierBasis` oracle

The pinned native probe is `shape_upgrade_convert_surface_bezier.cxx`. Build it against
the local OCCT install from the repository root:

```sh
g++ -std=c++17 \
  -I/home/vmiguel/git/oblikovati-workspace/RustyCADKernel/target/occt-modeling-full/include/opencascade \
  tools/oracle/shape_upgrade_convert_surface_bezier.cxx \
  -L/home/vmiguel/git/oblikovati-workspace/RustyCADKernel/target/occt-modeling-full/lin64/gcc/libi \
  -Wl,-rpath,/home/vmiguel/git/oblikovati-workspace/RustyCADKernel/target/occt-modeling-full/lin64/gcc/libi \
  -lTKShHealing -lTKGeomBase -lTKG3d -lTKMath -lTKernel \
  -o /tmp/rustycad-surface-bezier-oracle
LD_LIBRARY_PATH=/home/vmiguel/git/oblikovati-workspace/RustyCADKernel/target/occt-modeling-full/lin64/gcc/libi \
  /tmp/rustycad-surface-bezier-oracle
```

Pinned output:

```text
plane status=1,0 segments=1x1 patch=Geom_BezierSurface patch_bounds=0,1,0,1 result=1x1 result_patch=Geom_BezierSurface
bezier status=0,1 segments=1x1 patch=Geom_BezierSurface patch_bounds=0,1,0,1 result=1x1 result_patch=Geom_BezierSurface
trimmed status=1,0 segments=1x1 patch=Geom_BezierSurface patch_bounds=0,1,0,1 result=1x1 result_patch=Geom_BezierSurface
offset status=1,0 segments=1x1 patch=Geom_BezierSurface patch_bounds=0,1,0,1 result=1x1 result_patch=Geom_OffsetSurface
bspline status=1,0 segments=2x1 patch=Geom_BezierSurface patch_bounds=0,1,0,1 result=4x1 result_patch=Geom_BezierSurface
revolution status=1,0 segments=1x8 patch=Geom_SurfaceOfRevolution patch_bounds=0,6.28319,0,1 result=1x8 result_patch=Geom_SurfaceOfRevolution
extrusion status=1,0 segments=1x1 patch=Geom_BezierSurface patch_bounds=0,1,0,1 result=1x1 result_patch=Geom_BezierSurface
```

The Rust regressions compare status, patch type/count, full-range handle retention,
offset wrapping, and split-cell insertion for these cases. Revolution and extrusion use the
sibling `ShapeUpgrade_ConvertCurve3dToBezier` port to preserve the native curve breakpoints.

Rebuilt and checked against OCCT `3d097a0328e71b826377d4814ab05ec3c3d23871` in the pinned modeling build. The surface converter also preserves concrete parent downcasts, shared split state, and virtual Compute dispatch.
