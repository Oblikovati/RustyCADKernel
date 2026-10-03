# BRepOffset_MakeSimpleOffset state oracle

The native probe exercises default construction, `Perform()` on a null input,
successful shell generation from a bounded plane, `Generated`/`Modified`,
reinitialization after success, a bounded plane offset into a solid, and a closed-box
offset that reports multiple shells. It also covers safe-offset values, protected
`Clear`/`ComputeMaxAngle`, and simple offsets of cylinder, sphere, and cone solids. The
checked-in output is
`crates/tkoffset/tests/fixtures/make_simple_offset_state.txt`; it was generated
with OCCT revision `3d097a0328e71b826377d4814ab05ec3c3d23871`.

`Clear()` resets `IsDone`, error status, cached max angle, wall-edge sharing, and reshape
history. It preserves the result shape, input settings, build-solid flag, modifier, and
tolerance. The Rust class follows those state transitions. The plane case produces a
valid solid with signed volume `-3`; the closed-box case retains the offset result and
reports `ErrorInvalidNbShells`, matching the native probe.

The exposed protected-method probe verifies `ComputeMaxAngle` feeds the cached
`GetSafeOffset` path. `Clear` after both success and `ErrorInvalidNbShells` resets done/error
while preserving the result and the build flag, offset, and tolerance settings.

`BRepOffset_MakeSimpleOffset` calls the one-argument
`ShapeAnalysis_FreeBounds(shape)` overload. In the pinned header, that resolves to the
three-argument actual-boundary overload with defaults `(splitclosed=false,
splitopen=true, checkinternaledges=false)`, which the Rust code calls explicitly.
Sphere output exercises native degeneracy restoration: both pole edges remain
degenerated after modification. The cylinder, sphere, and cone simple-offset results
are valid solids with the expected edge-degeneracy counts.

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
