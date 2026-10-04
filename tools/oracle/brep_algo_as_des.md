# BRepAlgo_AsDes association graph

The offset-intersection pipeline uses `BRepAlgo_AsDes` to associate each face
with intersection edges and track already processed face pairs. This fixture
pins duplicate insertion, common-descendant output multiplicity, direct edits
to a bound descendant list, replacement list filtering, leaf removal, and
`Clear` against OCCT `3d097a0328e71b826377d4814ab05ec3c3d23871`.
Pinned source hashes are `BRepAlgo_AsDes.hxx`
`480bf992317e327b90d24ba8a6dc35aa8afbbbf47dfdfb94b686ce6803889097` and
`BRepAlgo_AsDes.cxx`
`5463330df8be6ee6cf8f90e2fd1efe40c822aa6ce98a77c926aff0f5d7999ee1`.

The native class returns a shared static empty list from `ChangeDescendant` for
an unbound source. Rust returns `None` in that case so an absent association
cannot be mutated through process-global state. This capability difference is
explicit; bound-list edits and the operations used by the offset pipeline are
compared to the native fixture.

Reproduce with the pinned OCCT modeling libraries:

```sh
occt=/home/vmiguel/git/oblikovati-workspace/RustyCADKernel/target/occt-modeling-full
lib="$occt/lin64/gcc/libi"
g++ -std=c++17 -I"$occt/include/opencascade" \
  tools/oracle/brep_algo_as_des.cxx -L"$lib" -Wl,-rpath,"$lib" \
  -lTKBool -lTKTopAlgo -lTKBRep -lTKG3d -lTKG2d -lTKMath -lTKernel \
  -o /tmp/brep-algo-as-des
/tmp/brep-algo-as-des
```
