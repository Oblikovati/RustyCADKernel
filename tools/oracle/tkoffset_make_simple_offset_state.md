# BRepOffset_MakeSimpleOffset state oracle

The native probe exercises default construction, `Perform()` on a null input,
successful shell generation from a bounded plane, `Generated`/`Modified`, and
reinitialization after success. The checked-in output is
`crates/tkoffset/tests/fixtures/make_simple_offset_state.txt`; it was generated
with OCCT revision `3d097a0328e71b826377d4814ab05ec3c3d23871`.

`Clear()` resets `IsDone`, error status, cached max angle, and reshape history. It
preserves the result shape, input settings, build-solid flag, modifier, and tolerance.
The Rust class follows those state transitions. Its build-solid path returns
`Standard_NotImplemented` after the offset shell is computed because wall
reconstruction is still unported; it leaves native error status `OK` and does not
mark the algorithm done.

Build and run against the OCCT build used by the port:

```sh
c++ -std=c++17 -I<occt-build>/include/opencascade \
  tools/oracle/tkoffset_make_simple_offset_state.cxx \
  -L<occt-build>/lin64/gcc/libi \
  -Wl,-rpath,<occt-build>/lin64/gcc/libi \
  -lTKOffset -lTKShHealing -lTKPrim -lTKBO -lTKBRep -lTKTopAlgo \
  -lTKGeomAlgo -lTKGeomBase -lTKG3d -lTKG2d -lTKMath -lTKernel \
  -o tkoffset_make_simple_offset_state
./tkoffset_make_simple_offset_state
```
