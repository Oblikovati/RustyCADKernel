# ShapeFix_EdgeProjAux native oracle

The probe links against the pinned OCCT build corresponding to source commit
`3d097a0328e71b826377d4814ab05ec3c3d23871`. It calls the protected `Init2d`,
`Init3d`, and `UpdateParam2d` members through a derived `Probe` class; this keeps
the subclass-visible API under test alongside `Compute`.

Build and run with the workspace's OCCT installation:

```sh
OCCT_PREFIX=/home/vmiguel/git/oblikovati-workspace/occt-install
g++ -std=c++17 tools/oracle/shape_fix_edge_proj_aux.cxx \
  -I"$OCCT_PREFIX/include/opencascade" -L"$OCCT_PREFIX/lib" \
  -Wl,-rpath,"$OCCT_PREFIX/lib" \
  -lTKShHealing -lTKTopAlgo -lTKBRep -lTKGeomBase -lTKG3d -lTKG2d -lTKMath -lTKernel \
  -o /tmp/shape_fix_edge_proj_aux_oracle
LD_LIBRARY_PATH="$OCCT_PREFIX/lib:/home/vmiguel/git/oblikovati-workspace/occt-build/lin64/gcc/lib" \
  /tmp/shape_fix_edge_proj_aux_oracle
```

Rows ending in four values record done flags and parameters for `Init2d` or
`Init3d`. `UpdateParam2d` rows record normalized parameters and curve direction
or endpoint coordinates. Extra extrema rows expose the solver status beside
the final `Init3d` flags. Together they check reversed-edge vertex selection,
failed extrema behavior, the null-p-curve state, period detection through a
trimmed basis curve, and the nonclosed B-spline branch.
