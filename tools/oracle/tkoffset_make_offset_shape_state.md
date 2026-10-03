# `BRepOffsetAPI_MakeOffsetShape` simple wrapper state oracle

This probe checks the public wrapper behavior added with `PerformBySimple`: the
default algorithm selector, `Build`, `Generated`, `Modified`, `IsDeleted`, a null
input, a successful bounded-plane offset, and a failed run after success. The
expected output was recorded against OCCT revision
`3d097a0328e71b826377d4814ab05ec3c3d23871`.
The captured output is in `crates/tkoffset/tests/fixtures/make_offset_shape_state.txt`.

```sh
c++ -std=c++17 -I<occt-build>/include/opencascade \
  tools/oracle/tkoffset_make_offset_shape_state.cxx \
  -L<occt-build>/lin64/gcc/libi \
  -Wl,-rpath,<occt-build>/lin64/gcc/libi \
  -lTKOffset -lTKShHealing -lTKPrim -lTKBO -lTKBRep -lTKTopAlgo \
  -lTKGeomAlgo -lTKGeomBase -lTKG3d -lTKG2d -lTKMath -lTKernel \
  -o tkoffset_make_offset_shape_state
./tkoffset_make_offset_shape_state
```

The wrapper's `Build()` is intentionally a no-op. Before any simple algorithm
run, history lists are empty. After a null `PerformBySimple`, `Generated(null)`
raises `Standard_NoSuchObject`, `Modified(null)` is empty, and `IsDeleted` is
false. A successful bounded plane run has one generated result and no modified
result. A later null run clears the done flag while retaining the previous
result internally; `Shape()` then raises `StdFail_NotDone`.
