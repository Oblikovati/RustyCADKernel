# ShapeFix_Edge parameter and vertex edge cases

The native probe checks missing 3D-curve handling, status resets, the face-limited
vertex tolerance overload, `FixSameParameter` default tolerance calls, and context
copy/rejection behavior. `tkshhealing_fix_edge_api_context.expected` records OCCT output at
revision `3d097a0328e71b826377d4814ab05ec3c3d23871`.

```sh
c++ -std=c++17 -I<occt-build>/include/opencascade \
  tools/oracle/tkshhealing_fix_edge_api_context.cxx \
  -L<occt-build>/lin64/gcc/libi \
  -Wl,-rpath,<occt-build>/lin64/gcc/libi \
  -lTKShHealing -lTKPrim -lTKBO -lTKBRep -lTKTopAlgo \
  -lTKGeomAlgo -lTKGeomBase -lTKG3d -lTKG2d -lTKMath -lTKernel \
  -o tkshhealing_fix_edge_api_context
./tkshhealing_fix_edge_api_context
```

The checker defaults to 23 control samples, so the Rust translation passes 23
explicitly. With no 3D curve, OCCT leaves the `ShapeFix_Edge` status at OK and
returns false while setting the same-parameter flag. A selected face limits
vertex tolerance repair to its own surface; the context overload repairs copied
vertices and leaves source vertices untouched.
